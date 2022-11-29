use super::buffer::{Buffer, MutBuffer};
use core::mem;
use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader, StreamWriter,
};
use num_enum::FromPrimitive;

#[derive(FromPrimitive)]
#[repr(u8)]
pub enum BufferRights {
    #[num_enum(default)]
    Read = 2,
    Write = 4,
    ReadWrite = 6,
}

#[inline(always)]
fn make_header(len: usize, rights: BufferRights) -> u32 {
    ((len as u32) << 0x4) | 0x8 | rights as u32
}

#[inline(always)]
fn read_header(header: u32) -> (usize, BufferRights) {
    let len = (header >> 0x4) as usize;
    let rights = BufferRights::from((header & 7) as u8);
    (len, rights)
}

pub enum PermissionBuffer {
    Read(Buffer),
    Write(MutBuffer),
    ReadWrite(MutBuffer),
}

impl PermissionBuffer {
    pub fn new_read(data: &[u8]) -> Self {
        Self::new(data.as_ptr() as usize, data.len(), BufferRights::Read)
    }

    pub fn new_write(data: &mut [u8]) -> Self {
        Self::new(data.as_ptr() as usize, data.len(), BufferRights::Write)
    }

    pub fn new(ptr: usize, len: usize, rights: BufferRights) -> Self {
        match rights {
            BufferRights::Read => Self::Read(Buffer::new(ptr as *const u8, len)),
            BufferRights::Write => Self::Write(MutBuffer::new(ptr as *mut u8, len)),
            BufferRights::ReadWrite => Self::ReadWrite(MutBuffer::new(ptr as *mut u8, len)),
        }
    }

    /// # Safety
    /// Behavior is undefined if the buffer did not come from kernel translation.
    pub unsafe fn as_slice(&self) -> &[u8] {
        match self {
            Self::Read(buffer) => buffer.as_slice(),
            Self::Write(_) => &[],
            Self::ReadWrite(buffer) => buffer.as_slice(),
        }
    }

    /// # Safety
    /// Behavior is undefined if the buffer did not come from kernel translation.
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        match self {
            Self::Read(_) => &mut [],
            Self::Write(buffer) => buffer.as_mut_slice(),
            Self::ReadWrite(buffer) => buffer.as_mut_slice(),
        }
    }

    /// # Safety
    /// Behavior is undefined if the buffer did not come from kernel translation.
    pub unsafe fn as_read_stream(&self) -> StreamContainer<&[u8]> {
        StreamContainer::new(self.as_slice())
    }

    /// # Safety
    /// Behavior is undefined if the buffer did not come from kernel translation.
    pub unsafe fn as_write_stream(&mut self) -> StreamContainer<&mut [u8]> {
        StreamContainer::new(self.as_mut_slice())
    }

    #[inline(always)]
    fn header(&self) -> u32 {
        match self {
            Self::Read(buf) => make_header(buf.len(), BufferRights::Read),
            Self::Write(buf) => make_header(buf.len(), BufferRights::Write),
            Self::ReadWrite(buf) => make_header(buf.len(), BufferRights::ReadWrite),
        }
    }

    pub fn ptr(&self) -> usize {
        match self {
            Self::Read(buf) => buf.ptr() as usize,
            Self::Write(buf) => buf.ptr() as usize,
            Self::ReadWrite(buf) => buf.ptr() as usize,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Read(buf) => buf.len(),
            Self::Write(buf) => buf.len(),
            Self::ReadWrite(buf) => buf.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl EndianRead for PermissionBuffer {
    #[cfg(target_pointer_width = "32")]
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let mut stream = StreamContainer::new(bytes);

        let header = stream.read_stream_le()?;
        let ptr = stream.read_stream_le::<u32>()? as usize;
        let (len, rights) = read_header(header);

        Ok(ReadOutput::new(
            Self::new(ptr, len, rights),
            stream.get_index(),
        ))
    }

    #[cfg(target_pointer_width = "64")]
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let mut stream = StreamContainer::new(bytes);

        let header = stream.read_stream_le()?;
        let ptr = stream.read_stream_le::<u64>()? as usize;
        let (len, rights) = read_header(header);

        Ok(ReadOutput::new(
            Self::new(ptr, len, rights),
            stream.get_index(),
        ))
    }

    fn try_read_be(_bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl EndianWrite for PermissionBuffer {
    fn get_size(&self) -> usize {
        mem::size_of::<*const u8>() + mem::size_of::<u32>()
    }

    #[cfg(target_pointer_width = "32")]
    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let mut stream = StreamContainer::new(dst);
        stream.write_stream_le(&self.header())?;
        stream.write_stream_le(&(self.ptr() as u32))?;
        Ok(stream.get_index())
    }

    #[cfg(target_pointer_width = "64")]
    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let mut stream = StreamContainer::new(dst);
        stream.write_stream_le(&self.header())?;
        stream.write_stream_le(&(self.ptr() as u64))?;
        Ok(stream.get_index())
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}
