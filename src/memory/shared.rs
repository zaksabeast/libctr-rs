use crate::{res::CtrResult, svc, Handle};
use num_enum::IntoPrimitive;

#[derive(IntoPrimitive)]
#[repr(u32)]
pub enum MemoryPermission {
    None = 0,
    Read = 1,
    Write = 2,
    ReadWrite = 3,
    Execute = 4,
    ReadExecute = 5,
    DontCare = 0x10000000,
}

pub struct MemoryBlock<'a> {
    handle: Handle,
    slice: &'a mut [u8],
}

/// An error will be returned if it's given a slice not aligned to 0x1000.
#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
impl<'a> MemoryBlock<'a> {
    pub fn new(
        slice: &'a mut [u8],
        my_permission: MemoryPermission,
        other_process_permission: MemoryPermission,
    ) -> CtrResult<Self> {
        let handle = svc::create_memory_block(slice, my_permission, other_process_permission)?;

        Ok(Self { handle, slice })
    }

    pub fn get_size(&self) -> usize {
        self.slice.len()
    }

    pub fn get_handle(&self) -> &Handle {
        &self.handle
    }
}

impl<'a> Drop for MemoryBlock<'a> {
    // If this doesn't close, there's not much to recover from
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        svc::unmap_memory_block(&self.handle, self.slice);
    }
}
