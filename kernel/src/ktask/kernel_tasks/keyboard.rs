use crate::println;
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{stream::Stream, task::AtomicWaker};

/// Instead of the OnceCell primitive,
/// we could also use the lazy_static macro here.
/// However, the OnceCell type has the advantage that
/// we can ensure that the initialization does not happen
/// in the interrupt handler, thus preventing that
/// the interrupt handler performs a heap allocation.
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

/// The size of the SCANCODE_QUEUE.
const SCANCODE_QUEUE_SIZE: usize = 128;

/// To implement the Waker notification for our ScancodeStream,
/// we need a place where we can store the Waker between poll calls.
/// We can't store it as a field in the ScancodeStream itself
/// because it needs to be accessible from the add_scancode function.
/// The solution for this is to use a static variable of
/// the AtomicWaker type provided by the futures-util crate.
/// Like the ArrayQueue type, this type is based on atomic instructions
/// and can be safely stored in a static and modified concurrently.
static WAKER: AtomicWaker = AtomicWaker::new();

/// This is a singleton class
pub struct ScancodeStream {
    _singleton: (),
}

impl ScancodeStream {
    /// The only way to create a ScancodeStream is use Self::new()
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(SCANCODE_QUEUE_SIZE))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _singleton: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        // fast path
        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
    // if not initialized, just do nothing.
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        // The ArrayQueue performs all necessary synchronization itself,
        // so we don't need a mutex wrapper here.
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    }
}

pub async fn print_keyevents() {
    use crate::print;
    use futures_util::stream::StreamExt;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

    let mut stream = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    while let Some(scancode) = stream.next().await {
        match keyboard.add_byte(scancode) {
            Ok(Some(event)) => match keyboard.process_keyevent(event) {
                Some(DecodedKey::RawKey(key)) => print!("{:?}", key),
                Some(DecodedKey::Unicode(char)) => print!("{}", char),
                _ => (),
            },
            _ => (),
        }
    }
}
