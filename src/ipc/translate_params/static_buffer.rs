use super::buffer::Buffer;
use core::mem;
use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, LeIter, ReadOutput, StreamContainer, StreamReader,
    StreamWriter,
};

#[inline(always)]
pub fn make_header(len: usize, buffer_id: u16) -> u32 {
    ((len as u32) << 0xE) | (((buffer_id as u32) & 0xF) << 0xA) | 0x2
}

#[inline(always)]
fn read_header(header: u32) -> (usize, u16) {
    ((header >> 0xE) as usize, ((header >> 0xA) & 0xF) as u16)
}

#[derive(Debug)]
pub struct StaticBuffer {
    id: u16,
    buf: Buffer,
}

impl StaticBuffer {
    pub fn new(slice: &[u8], buffer_id: u16) -> Self {
        Self::new_raw(slice.as_ptr(), slice.len(), buffer_id)
    }

    pub fn new_mut(slice: &mut [u8], buffer_id: u16) -> Self {
        Self::new_mut_raw(slice.as_mut_ptr(), slice.len(), buffer_id)
    }

    fn new_raw(ptr: *const u8, len: usize, buffer_id: u16) -> Self {
        Self {
            id: buffer_id,
            buf: Buffer::new(ptr, len),
        }
    }

    fn new_mut_raw(ptr: *mut u8, len: usize, buffer_id: u16) -> Self {
        Self::new_raw(ptr, len, buffer_id)
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    #[inline(always)]
    fn header(&self) -> u32 {
        make_header(self.buf.len(), self.id)
    }

    /// # Safety
    /// Behavior is undefined if the buffer did not come from kernel translation.
    pub unsafe fn as_slice(&self) -> &[u8] {
        self.buf.as_slice()
    }

    /// # Safety
    /// Behavior is undefined if the buffer did not come from kernel translation.
    pub unsafe fn as_stream(&self) -> StreamContainer<&[u8]> {
        StreamContainer::new(self.as_slice())
    }

    /// # Safety
    /// Behavior is undefined if the buffer did not come from kernel translation.
    pub unsafe fn iter<T: EndianRead>(&self) -> LeIter<T, StreamContainer<&[u8]>> {
        self.as_stream().into_le_iter()
    }
}

impl EndianRead for StaticBuffer {
    #[inline(always)]
    #[cfg(target_pointer_width = "32")]
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let mut stream = StreamContainer::new(bytes);

        let header = stream.read_stream_le()?;
        let ptr = stream.read_stream_le::<u32>()? as *const u8;
        let (len, buffer_id) = read_header(header);

        Ok(ReadOutput::new(
            Self::new_raw(ptr, len, buffer_id),
            stream.get_index(),
        ))
    }

    #[inline(always)]
    #[cfg(target_pointer_width = "64")]
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let mut stream = StreamContainer::new(bytes);

        let header = stream.read_stream_le()?;
        let ptr = stream.read_stream_le::<u64>()? as *const u8;
        let (len, buffer_id) = read_header(header);

        Ok(ReadOutput::new(
            Self::new_raw(ptr, len, buffer_id),
            stream.get_index(),
        ))
    }

    #[inline(always)]
    fn try_read_be(_bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl EndianWrite for StaticBuffer {
    #[inline(always)]
    fn get_size(&self) -> usize {
        mem::size_of::<*const u8>() + mem::size_of::<u32>()
    }

    #[inline(always)]
    #[cfg(target_pointer_width = "32")]
    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let mut stream = StreamContainer::new(dst);
        stream.write_stream_le(&self.header())?;
        stream.write_stream_le(&(self.buf.ptr() as u32))?;
        Ok(stream.get_index())
    }

    #[inline(always)]
    #[cfg(target_pointer_width = "64")]
    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let mut stream = StreamContainer::new(dst);
        stream.write_stream_le(&self.header())?;
        stream.write_stream_le(&(self.buf.ptr() as u64))?;
        Ok(stream.get_index())
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}
