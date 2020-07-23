use alloc::collections::VecDeque;
use crate::ktask::Task;
use core::task::{Waker, RawWaker};
use core::task::RawWakerVTable;
use core::task::{Context, Poll};

pub struct SimpleExecutor {
    tasks: VecDeque<Task>,
}

impl SimpleExecutor {
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            tasks: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.tasks.push_back(task)
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.tasks.pop_front() {
            let waker = no_op_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => self.tasks.push_back(task),
            }
        }
    }
}

fn no_op_waker_raw() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        no_op_waker_raw()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn no_op_waker() -> Waker {
    unsafe { Waker::from_raw(no_op_waker_raw()) }
}
