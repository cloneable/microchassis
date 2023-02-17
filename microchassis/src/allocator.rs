use core::{
    alloc::{GlobalAlloc, Layout},
    cell::Cell,
};

pub struct OomPanicAllocator<T: GlobalAlloc>(pub T);

thread_local! {
    static PANICKING: Cell<bool> = Cell::new(false);
}

#[allow(clippy::panic)]
#[inline]
fn panic_alloc(size: usize) -> ! {
    PANICKING.with(|v| v.set(true));
    panic!("memory allocation of {size} bytes failed");
}

#[allow(clippy::panic)]
#[inline]
fn panic_realloc(old_size: usize, new_size: usize) -> ! {
    PANICKING.with(|v| v.set(true));
    panic!("memory reallocation from {old_size} to {new_size} bytes failed");
}

#[allow(unsafe_code)]
unsafe impl<T: GlobalAlloc> GlobalAlloc for OomPanicAllocator<T> {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.0.alloc(layout);
        if ptr.is_null() && !PANICKING.with(Cell::get) {
            panic_alloc(layout.size());
        }
        ptr
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = self.0.alloc_zeroed(layout);
        if ptr.is_null() && !PANICKING.with(Cell::get) {
            panic_alloc(layout.size());
        }
        ptr
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let ptr = self.0.realloc(ptr, layout, new_size);
        if ptr.is_null() && !PANICKING.with(Cell::get) && new_size > layout.size() {
            panic_realloc(layout.size(), new_size);
        }
        ptr
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.dealloc(ptr, layout);
    }
}
