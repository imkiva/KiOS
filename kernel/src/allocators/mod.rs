pub mod bump;
pub mod fixed;
pub mod linked;

#[allow(dead_code)]
pub static KERNEL_ALLOCATOR1: Locked<bump::BumpAllocator> = Locked::new(bump::BumpAllocator::new());

#[allow(dead_code)]
pub static KERNEL_ALLOCATOR2: Locked<linked::LinkedListAllocator> =
    Locked::new(linked::LinkedListAllocator::new());

#[global_allocator]
pub static KERNEL_ALLOCATOR3: Locked<fixed::FixedSizeBlockAllocator> =
    Locked::new(fixed::FixedSizeBlockAllocator::new());

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
