use crate::Handle;
use alloc::vec::Vec;
use core::mem;
use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader, StreamWriter,
};

#[inline]
fn make_header(len: u32) -> u32 {
    (len - 1) << 26
}

#[inline]
fn read_header(header: u32) -> u32 {
    (header >> 26) + 1
}

pub struct Handles {
    raw_handles: Vec<u32>,
}

impl Handles {
    pub fn new(raw_handles: Vec<u32>) -> Self {
        Self { raw_handles }
    }

    fn header(&self) -> u32 {
        make_header(self.raw_handles.len() as u32)
    }

    pub fn into_raw(self) -> Vec<u32> {
        self.raw_handles
    }

    pub fn into_handles(self) -> Vec<Handle> {
        self.raw_handles.into_iter().map(|raw| raw.into()).collect()
    }

    pub fn into_handle(self) -> Option<Handle> {
        self.raw_handles.first().map(|raw| (*raw).into())
    }
}

impl EndianRead for Handles {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let mut stream = StreamContainer::new(bytes);
        let header: u32 = stream.read_stream_le()?;
        let len = read_header(header) as usize;

        let mut raw_handles = Vec::with_capacity(len);

        for _ in 0..len {
            let handle: u32 = stream.read_stream_le()?;
            raw_handles.push(handle);
        }

        Ok(ReadOutput::new(Self { raw_handles }, stream.get_index()))
    }

    fn try_read_be(_bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl EndianWrite for Handles {
    fn get_size(&self) -> usize {
        mem::size_of::<u32>() * (self.raw_handles.len() + 1)
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let mut stream = StreamContainer::new(dst);
        stream.write_stream_le(&self.header())?;

        for raw_handle in self.raw_handles.iter() {
            stream.checked_write_stream_le(raw_handle);
        }

        Ok(stream.get_index())
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}
