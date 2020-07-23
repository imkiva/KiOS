use crate::ktask::{KernelTask, TaskId};
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;

/// We assume that our kernel will have less than 128 kernel tasks
/// currently. Maybe we need a flexible size in the future.
const KERNEL_TASK_EXECUTOR_POOL_SIZE: usize = 128;

pub struct Executor {
    /// Where all kernel tasks store
    tasks: BTreeMap<TaskId, KernelTask>,

    /// We use this Arc<ArrayQueue> type for the task_queue
    /// because it will be shared between the executor and wakers.
    /// The idea is that the wakers push the ID of the woken task to the queue.
    /// The executor sits on the receiving end of the queue,
    /// retrieves the woken tasks by their ID from the tasks map,
    /// and then runs them. The reason for using a fixed-size queue
    /// instead of an unbounded queue such as SegQueue is that
    /// interrupt handlers that should not allocate will push to this queue.
    task_queue: Arc<ArrayQueue<TaskId>>,

    /// This map caches the Waker of a task after its creation.
    /// This has two reasons:
    /// First, it improves performance by reusing the same waker
    /// for multiple wake-ups of the same task instead of
    /// creating a new waker each time.
    /// Second, it ensures that reference-counted wakers are
    /// not deallocated inside interrupt handlers
    /// because it could lead to deadlocks.
    waker_cache: BTreeMap<TaskId, Waker>,
}

struct TaskWaker {
    /// The woken task id.
    task_id: TaskId,

    /// the ownership of the task_queue is shared
    /// between the executor and wakers,
    /// we use the Arc wrapper type to implement
    /// shared reference-counted ownership.
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(KERNEL_TASK_EXECUTOR_POOL_SIZE)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: KernelTask) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same TID already exists");
        }
        self.task_queue.push(task_id).expect("kernel task full");
    }

    fn run_ready_tasks(&mut self) {
        // destructure `self` to avoid borrow checker errors
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        while let Ok(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done -> remove it and its cached waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.idle();
        }
    }

    fn idle(&self) {
        if self.task_queue.is_empty() {
            crate::cpu::hlt();
        }
    }
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue
            .push(self.task_id)
            .expect("kernel task full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
