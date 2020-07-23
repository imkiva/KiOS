pub mod bump;
pub mod linked;

pub static KERNEL_ALLOCATOR1: Locked<bump::BumpAllocator> = Locked::new(bump::BumpAllocator::new());

#[global_allocator]
pub static KERNEL_ALLOCATOR2: Locked<linked::LinkedListAllocator> =
    Locked::new(linked::LinkedListAllocator::new());

pub struct Locked<A> {
    lock: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            lock: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.lock.lock()
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    match addr % align {
        0 => addr,
        align => addr - align + align,
    }
}
