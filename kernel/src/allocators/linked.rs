use crate::allocators::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};

struct Node {
    size: usize,
    next: Option<&'static mut Node>,
}

impl Node {
    const fn new(size: usize) -> Self {
        Node { size, next: None }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    head: Node,
}

impl LinkedListAllocator {
    /// Creates an empty LinkedListAllocator.
    pub const fn new() -> Self {
        Self { head: Node::new(0) }
    }

    /// Initialize the allocator with the given heap bounds.
    ///
    /// This function is unsafe because the caller must guarantee that the given
    /// heap bounds are valid and that the heap is unused. This method must be
    /// called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_space(heap_start, heap_size);
    }

    /// Adjust the given layout so that the resulting allocated memory
    /// region is also capable of storing a `Node`.
    ///
    /// Returns the adjusted size and alignment as a (size, align) tuple.
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<Node>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<Node>());
        (size, layout.align())
    }

    /// Adds the given memory region to the front of the free list.
    unsafe fn add_free_space(&mut self, addr: usize, size: usize) {
        // ensure that the freed region is capable of holding Node
        assert_eq!(align_up(addr, mem::align_of::<Node>()), addr);
        assert!(size >= mem::size_of::<Node>());

        // create a new list node and append it at the start of the list
        let mut node = Node::new(size);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut Node;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr)
    }

    /// Looks for a free region with the given size and alignment and removes
    /// it from the free list.
    ///
    /// Returns a tuple of the list node and the start address of the
    /// allocation.
    fn find_free_space(&mut self, size: usize, align: usize) -> Option<(&'static mut Node, usize)> {
        // reference to current list node, updated for each iteration
        let mut current = &mut self.head;
        // look for a large enough memory region in linked list
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from(&region, size, align) {
                // region suitable for allocation -> remove node from list
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            } else {
                // region not suitable -> continue with next region
                current = current.next.as_mut().unwrap();
            }
        }

        // no suitable region found
        None
    }

    /// Try to use the given region for an allocation with given size and
    /// alignment.
    ///
    /// Returns the allocation start address on success.
    fn alloc_from(region: &Node, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start + size;

        if alloc_end > region.end_addr() {
            // region too small
            return Err(());
        }

        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<Node>() {
            // rest of region too small to hold a Node (required because the
            // allocation splits the region in a used and a free part)
            return Err(());
        }

        // region suitable for allocation
        Ok(alloc_start)
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // perform layout adjustments
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_free_space(size, align) {
            let alloc_end = alloc_start + size;
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_space(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // perform layout adjustments
        let (size, _) = LinkedListAllocator::size_align(layout);

        self.lock().add_free_space(ptr as usize, size)
    }
}
