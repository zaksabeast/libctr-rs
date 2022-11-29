use crate::{res::CtrResult, svc, Handle};
use alloc::{alloc::Layout, boxed::Box};
use core::{cmp, ffi, mem};

unsafe extern "C" fn run_thread(func: *mut ffi::c_void) {
    let func = Box::from_raw(func as *mut Box<dyn FnOnce()>);
    func();
    svc::exit_thread();
}

pub struct Thread {
    handle: Handle,
    stack: *mut u8,
    layout: Layout,
}

impl Thread {
    pub fn new(
        stack_size: usize,
        priority: i32,
        processor_id: i32,
        func: impl FnOnce(),
    ) -> CtrResult<Self> {
        let stack_size = cmp::max(stack_size - (stack_size % 8), 0x1000);

        unsafe {
            // This is safe because:
            // - align is greater than 0
            // - align is a multiple of 2
            // - stack_size doesn't overflow align when rounded up
            let layout = Layout::from_size_align_unchecked(stack_size, 8);
            let stack = alloc::alloc::alloc(layout);

            // We need to double box so run_thread knows the size of the inner box.
            // Eventually we'll need to cast the pointer from c_void to something else,
            // and `dyn FnOnce` won't have a known size.
            let boxed_func: Box<Box<dyn FnOnce()>> = Box::new(Box::new(func));
            let handle = svc::create_thread(
                run_thread,
                &*boxed_func as *const _ as u32,
                stack as *mut u32,
                priority,
                processor_id,
            )?;

            // This memory will be dropped in the thread, so it needs to forget it here.
            // If it isn't forgotten, it will be dropped twice.
            mem::forget(boxed_func);
            Ok(Self {
                handle,
                stack,
                layout,
            })
        }
    }

    pub fn join(self) {
        #[allow(unused_must_use)]
        svc::wait_synchronization(&self.handle, i64::max_value());
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        unsafe { alloc::alloc::dealloc(self.stack, self.layout) }
    }
}

/// Spawns a new thread.
/// Panics if thread creation fails.
pub fn spawn(func: impl FnOnce()) -> Thread {
    let current_thread = Handle::get_current_thread_handle();
    let priority = svc::get_thread_priority(&current_thread).unwrap_or(0x3F);
    let capped_priority = cmp::max(priority, 0x18);
    Thread::new(0x4000, capped_priority, -2, func).unwrap()
}
