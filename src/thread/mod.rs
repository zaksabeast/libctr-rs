use crate::Handle;
use alloc::{alloc::Layout, boxed::Box};
use core::{cmp, ffi, mem};
use ctru_sys::{svcCreateThread, svcExitThread, svcGetThreadPriority, svcWaitSynchronization};

unsafe extern "C" fn run_thread(func: *mut ffi::c_void) {
    let func = Box::from_raw(func as *mut Box<dyn FnOnce()>);
    func();
    svcExitThread();
}

pub struct Thread {
    handle: Handle,
    stack: *mut u8,
    layout: Layout,
}

impl Thread {
    pub fn new(stack_size: usize, priority: i32, processor_id: i32, func: impl FnOnce()) -> Self {
        let mut handle = 0u32;
        let stack_size = stack_size + 8 - (stack_size % 8);

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
            svcCreateThread(
                &mut handle,
                Some(run_thread),
                &*boxed_func as *const _ as u32,
                stack as *mut u32,
                priority,
                processor_id,
            );

            // This memory will be dropped in the thread, so it needs to forget it here.
            // If it isn't forgotten, it will be dropped twice.
            mem::forget(boxed_func);
            Self {
                handle: Handle::from(handle),
                stack,
                layout,
            }
        }
    }

    pub fn join(self) {
        unsafe { svcWaitSynchronization(self.handle.get_raw(), i64::max_value()) };
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        unsafe { alloc::alloc::dealloc(self.stack, self.layout) }
    }
}

const CUR_THREAD_HANDLE: u32 = 0xFFFF8000;

pub fn spawn(func: impl FnOnce()) -> Thread {
    let mut priority = 0i32;
    unsafe { svcGetThreadPriority(&mut priority, CUR_THREAD_HANDLE) };
    let capped_priority = cmp::max(priority - 1, 0x18);
    Thread::new(0x4000, capped_priority, -2, func)
}
