use super::svc;
use core::{cmp::PartialEq, convert::From, ops::Drop};

#[derive(PartialEq, Debug)]
pub struct Handle(u32);

/// An abstraction on top of resource handles to enforce type safety.
/// A Handle should only be made from a resource that is guaranteed to be a unique copy.
/// When a Handle is dropped, the underlying resource handle is closed.
/// Handles are intentionally non-copyable to avoid using Handles that have already been closed.
impl Handle {
    /// Returns the raw u32 handle
    /// # Safety
    /// Because a Handle closes itself when it's dropped, a raw handle might have been previously closed.
    /// The user must guarantee the handle will outlive the raw handle (and all copies/clones of the raw handle)
    ///
    /// Admittedly this is less of memory safety and more of logical safety, but since that's the purpose of this abstraction
    /// unsafe will be used in this way.
    pub unsafe fn get_raw(&self) -> u32 {
        self.0
    }

    /// Returns a pseudo handle for the current process
    pub fn get_current_process_handle() -> Self {
        0xFFFF8001.into()
    }
}

impl From<u32> for Handle {
    fn from(raw_handle: u32) -> Self {
        Self(raw_handle)
    }
}

impl Drop for Handle {
    // If this doesn't close, there's not much to recover from
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        svc::close_handle(self.0);
    }
}
