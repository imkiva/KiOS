pub mod bump;

#[global_allocator]
pub static KERNEL_ALLOCATOR: Locked<bump::BumpAllocator> = Locked::new(bump::BumpAllocator::new());

struct Locked<A> {
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
