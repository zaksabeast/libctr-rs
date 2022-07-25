use std::slice;

#[derive(Debug)]
pub struct Buffer {
    ptr: *const u8,
    len: usize,
}

impl Buffer {
    pub(super) fn new(ptr: *const u8, len: usize) -> Self {
        Self { ptr, len }
    }

    pub(super) fn ptr(&self) -> *const u8 {
        self.ptr
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }

    pub unsafe fn as_slice(&self) -> &[u8] {
        slice::from_raw_parts(self.ptr, self.len)
    }
}

pub struct MutBuffer {
    ptr: *mut u8,
    len: usize,
}

impl MutBuffer {
    pub(super) fn new(ptr: *mut u8, len: usize) -> Self {
        Self { ptr, len }
    }

    pub(super) fn ptr(&self) -> *const u8 {
        self.ptr
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }

    pub(super) unsafe fn as_slice(&self) -> &[u8] {
        slice::from_raw_parts(self.ptr, self.len)
    }

    pub(super) unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        slice::from_raw_parts_mut(self.ptr, self.len)
    }
}
