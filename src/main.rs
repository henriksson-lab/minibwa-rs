struct MiMalloc;

unsafe extern "C" {
    fn mi_malloc_aligned(size: usize, alignment: usize) -> *mut std::ffi::c_void;
    fn mi_realloc_aligned(
        p: *mut std::ffi::c_void,
        newsize: usize,
        alignment: usize,
    ) -> *mut std::ffi::c_void;
    fn mi_free(p: *mut std::ffi::c_void);
}

unsafe impl std::alloc::GlobalAlloc for MiMalloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        mi_malloc_aligned(layout.size().max(1), layout.align()).cast()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: std::alloc::Layout) {
        mi_free(ptr.cast());
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: std::alloc::Layout, new_size: usize) -> *mut u8 {
        mi_realloc_aligned(ptr.cast(), new_size.max(1), layout.align()).cast()
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: MiMalloc = MiMalloc;

fn main() {
    std::process::exit(minibwa_rs::cli::run_from_env());
}
