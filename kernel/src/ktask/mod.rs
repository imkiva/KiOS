use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub mod kernel_tasks;
pub mod simple_executor;

pub struct KernelTask {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl KernelTask {
    pub fn new(future: impl Future<Output = ()> + 'static) -> KernelTask {
        KernelTask {
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
