use crate::{
    ipc::{Command, CurrentProcessId, StaticBuffer},
    res::{error, CtrResult},
    service_session::create_session_manager,
    srv::get_service_handle_direct,
    Handle,
};
use alloc::{str, vec, vec::Vec};
use core::{convert::Into, mem};
use cstr_core::CString;
use no_std_io::{EndianRead, EndianWrite, ReadOutput, Writer};
use num_enum::IntoPrimitive;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, EndianRead, EndianWrite)]
pub struct ProductInfo {
    pub product_code: [u8; 0x10],
    pub company_code: [u8; 2],
    pub remaster_version: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MediaType {
    Nand = 0,
    Sd = 1,
    GameCard = 2,
}

impl From<u8> for MediaType {
    fn from(raw_media_type: u8) -> Self {
        match raw_media_type {
            0 => MediaType::Nand,
            1 => MediaType::Sd,
            2 => MediaType::GameCard,
            _ => MediaType::Nand,
        }
    }
}

impl EndianRead for MediaType {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, no_std_io::Error> {
        Ok(u8::try_read_le(bytes)?.into_other())
    }

    fn try_read_be(_bytes: &[u8]) -> Result<ReadOutput<Self>, no_std_io::Error> {
        unimplemented!()
    }
}

impl EndianWrite for MediaType {
    fn get_size(&self) -> usize {
        mem::size_of::<u8>()
    }

    fn try_write_le(&self, mut dst: &mut [u8]) -> Result<usize, no_std_io::Error> {
        dst.write_le(0, &(*self as u8))
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, no_std_io::Error> {
        unimplemented!()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EndianRead, EndianWrite)]
#[repr(C)]
pub struct ProgramInfo {
    pub program_id: u64,
    pub media_type: MediaType,
    pub padding: [u8; 7],
}

impl Default for ProgramInfo {
    fn default() -> Self {
        Self {
            program_id: 0,
            media_type: MediaType::Nand,
            padding: [0; 7],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum ArchiveId {
    SystemSaveData = 8,
    Sdmc = 9,
}

#[derive(IntoPrimitive)]
#[repr(u32)]
pub enum WriteFlags {
    Flush = 1,
}

#[derive(IntoPrimitive)]
#[repr(u32)]
pub enum OpenFlags {
    Read = 1,
    Write = 2,
    Create = 4,
    ReadWriteCreate = 7,
}

/// A path used with the fs module to open archives and files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsPath {
    Empty(Vec<u8>),
    Binary(Vec<u8>),
    Ascii(Vec<u8>),
}

impl FsPath {
    pub fn new_empty_path() -> Self {
        Self::Empty(vec![])
    }

    pub fn new_binary(path: [u32; 2]) -> Self {
        let inner = path.iter().flat_map(|word| word.to_le_bytes()).collect();
        Self::Binary(inner)
    }

    pub fn get_raw_type(&self) -> u32 {
        match self {
            Self::Empty(_) => 1,
            Self::Binary(_) => 2,
            Self::Ascii(_) => 3,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Empty(_) => 1,
            Self::Binary(path) => path.len(),
            Self::Ascii(path) => path.len() + 1,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_inner(&self) -> &[u8] {
        match self {
            Self::Empty(path) => path,
            Self::Binary(path) => path,
            Self::Ascii(path) => path,
        }
    }
}

impl From<&str> for FsPath {
    fn from(path: &str) -> Self {
        if path.is_empty() {
            Self::new_empty_path()
        } else {
            let c_path = CString::new(path).unwrap_or_default();
            Self::Ascii(c_path.into_bytes_with_nul())
        }
    }
}

create_session_manager!({
    let handle = get_service_handle_direct("fs:USER")?;
    user::initialize_with_sdk_version(&handle, 0x90c00c8)?;
    user::set_priority(&handle, 0)?;
    handle
});

pub mod user {
    use super::*;

    pub fn set_priority(session: &Handle, priority: u32) -> CtrResult {
        let raw_handle = unsafe { session.get_raw() };
        Command::new(0x8620040, priority).send(raw_handle)
    }

    #[derive(EndianRead, EndianWrite)]
    struct InitializeWithSdkVersionIn {
        version: u32,
        current_process_id: CurrentProcessId,
    }

    pub fn initialize_with_sdk_version(session: &Handle, version: u32) -> CtrResult {
        let input = InitializeWithSdkVersionIn {
            version,
            current_process_id: CurrentProcessId::new(),
        };
        let raw_handle = unsafe { session.get_raw() };
        Command::new(0x8610042, input).send(raw_handle)
    }

    #[derive(EndianRead, EndianWrite)]
    struct OpenFileDirectlyIn {
        zero: u32,
        archive_id: u32,
        archive_path_type: u32,
        archive_path_len: u32,
        file_path_type: u32,
        file_path_len: u32,
        flags: u32,
        attributes: u32,
        archive_path_buf: StaticBuffer,
        file_path_buf: StaticBuffer,
    }

    pub fn open_file_directly(
        archive_id: ArchiveId,
        archive_path: &FsPath,
        file_path: &FsPath,
        flags: OpenFlags,
        attributes: u32,
    ) -> CtrResult<Handle> {
        let input = OpenFileDirectlyIn {
            zero: 0,
            archive_id: archive_id as u32,
            archive_path_type: archive_path.get_raw_type(),
            archive_path_len: archive_path.len() as u32,
            file_path_type: file_path.get_raw_type(),
            file_path_len: file_path.len() as u32,
            flags: flags as u32,
            attributes,
            archive_path_buf: StaticBuffer::new(archive_path.get_inner(), 2),
            file_path_buf: StaticBuffer::new(file_path.get_inner(), 0),
        };
        let result: u32 = Command::new(0x8030204, input).send(get_handle())?;
        Ok(result.into())
    }

    #[derive(EndianRead, EndianWrite)]
    struct OpenFileIn {
        zero: u32,
        raw_archive_handle: u64,
        path_type: u32,
        path_len: u32,
        flags: u32,
        attributes: u32,
        path_buf: StaticBuffer,
    }

    #[derive(EndianRead, EndianWrite)]
    struct OpenFileOut {
        unk: u32,
        handle: u32,
    }

    pub fn open_file(
        raw_archive_handle: u64,
        path: &FsPath,
        flags: OpenFlags,
        attributes: u32,
    ) -> CtrResult<Handle> {
        let input = OpenFileIn {
            zero: 0,
            raw_archive_handle,
            path_type: path.get_raw_type(),
            path_len: path.len() as u32,
            flags: flags as u32,
            attributes,
            path_buf: StaticBuffer::new(path.get_inner(), 0),
        };
        let result: OpenFileOut = Command::new(0x80201C2, input).send(get_handle())?;
        Ok(result.handle.into())
    }

    #[derive(EndianRead, EndianWrite)]
    struct OpenArchiveIn {
        id: u32,
        path_type: u32,
        path_len: u32,
        path_buf: StaticBuffer,
    }

    /// Opens an archive and returns the raw archive handle.
    pub fn open_archive(id: ArchiveId, path: &FsPath) -> CtrResult<u64> {
        let input = OpenArchiveIn {
            id: id as u32,
            path_type: path.get_raw_type(),
            path_len: path.len() as u32,
            path_buf: StaticBuffer::new(path.get_inner(), 0),
        };
        Command::new(0x80C00C2, input).send(get_handle())
    }

    pub fn close_archive(raw_archive_handle: u64) -> CtrResult {
        if raw_archive_handle == 0 {
            return Err(error::invalid_handle());
        }

        Command::new(0x80E0080, raw_archive_handle).send(get_handle())
    }

    #[derive(Debug, EndianRead, EndianWrite)]
    struct CreateDirectoryIn {
        zero: u32,
        raw_archive_handle: u64,
        path_type: u32,
        path_len: u32,
        attributes: u32,
        path_buf: StaticBuffer,
    }

    pub fn create_directory(raw_archive_handle: u64, path: &FsPath, attributes: u32) -> CtrResult {
        let input = CreateDirectoryIn {
            zero: 0,
            raw_archive_handle,
            path_type: path.get_raw_type(),
            path_len: path.len() as u32,
            attributes,
            path_buf: StaticBuffer::new(path.get_inner(), 1),
        };
        Command::new(0x8090182, input).send(get_handle())
    }

    #[derive(Debug, EndianRead, EndianWrite)]
    struct RenameDirectoryIn {
        zero: u32,
        src_archive_handle: u64,
        src_path_type: u32,
        src_path_len: u32,
        dst_archive_handle: u64,
        dst_path_type: u32,
        dst_path_len: u32,
        src_path_buf: StaticBuffer,
        dst_path_buf: StaticBuffer,
    }

    pub fn rename_directory(
        raw_archive_handle: u64,
        src_path: &FsPath,
        dst_path: &FsPath,
    ) -> CtrResult {
        let input = RenameDirectoryIn {
            zero: 0,
            src_archive_handle: raw_archive_handle,
            src_path_type: src_path.get_raw_type(),
            src_path_len: src_path.len() as u32,
            dst_archive_handle: raw_archive_handle,
            dst_path_type: dst_path.get_raw_type(),
            dst_path_len: dst_path.len() as u32,
            src_path_buf: StaticBuffer::new(src_path.get_inner(), 1),
            dst_path_buf: StaticBuffer::new(dst_path.get_inner(), 2),
        };
        Command::new(0x80A0244, input).send(get_handle())
    }

    #[derive(Debug, EndianRead, EndianWrite)]
    struct OpenDirectoryIn {
        raw_archive_handle: u64,
        path_type: u32,
        path_len: u32,
        path_buf: StaticBuffer,
    }

    #[derive(Debug, EndianRead, EndianWrite)]
    struct OpenDirectoryOut {
        unk: u32,
        handle: u32,
    }

    pub fn open_directory(raw_archive_handle: u64, path: &FsPath) -> CtrResult<Handle> {
        let input = OpenDirectoryIn {
            raw_archive_handle,
            path_type: path.get_raw_type(),
            path_len: path.len() as u32,
            path_buf: StaticBuffer::new(path.get_inner(), 0),
        };
        let result: OpenDirectoryOut = Command::new(0x80B0102, input).send(get_handle())?;
        Ok(result.handle.into())
    }

    pub fn get_program_launch_info(process_id: u32) -> CtrResult<ProgramInfo> {
        Command::new(0x82F0040, process_id).send(get_handle())
    }

    pub fn get_product_info(process_id: u32) -> CtrResult<ProductInfo> {
        Command::new(0x82E0040, process_id).send(get_handle())
    }
}

pub mod file {
    use super::*;
    use crate::ipc::PermissionBuffer;

    pub fn get_size(handle: &Handle) -> CtrResult<u64> {
        let raw_handle = unsafe { handle.get_raw() };
        Command::new(0x8040000, ()).send(raw_handle)
    }

    #[derive(EndianRead, EndianWrite)]
    struct FileWriteIn {
        offset: u64,
        buffer_len: u32,
        flags: u32,
        buf: PermissionBuffer,
    }

    #[derive(EndianRead, EndianWrite)]
    struct FileWriteOut {
        bytes_written: u32,
    }

    pub fn write(
        handle: &Handle,
        offset: u64,
        buffer: &[u8],
        flags: WriteFlags,
    ) -> CtrResult<usize> {
        let input = FileWriteIn {
            offset,
            flags: flags.into(),
            buffer_len: buffer.len() as u32,
            buf: PermissionBuffer::new_read(buffer),
        };
        let raw_handle = unsafe { handle.get_raw() };
        let result: FileWriteOut = Command::new(0x8030102, input).send(raw_handle)?;

        Ok(result.bytes_written as usize)
    }

    #[derive(EndianRead, EndianWrite)]
    struct FileReadIn {
        offset: u64,
        max_read_size: u32,
        out_buffer: PermissionBuffer,
    }

    #[derive(EndianRead, EndianWrite)]
    struct FileReadOut {
        bytes_read: u32,
    }

    /// Reads from a file.
    /// The output size is not guaranteed to be the max read size.
    pub fn read(handle: &Handle, offset: u64, max_read_size: usize) -> CtrResult<Vec<u8>> {
        let mut out_buffer: Vec<u8> = vec![0; max_read_size];
        let input = FileReadIn {
            max_read_size: max_read_size as u32,
            offset,
            out_buffer: PermissionBuffer::new_write(&mut out_buffer),
        };
        let raw_handle = unsafe { handle.get_raw() };
        let result: FileReadOut = Command::new(0x80200C2, input).send(raw_handle)?;

        out_buffer.resize(result.bytes_read as usize, 0);

        Ok(out_buffer)
    }

    pub fn close(handle: &Handle) -> CtrResult {
        let raw_handle = unsafe { handle.get_raw() };
        Command::new(0x8080000, ()).send::<()>(raw_handle)
    }
}

#[derive(EndianRead, EndianWrite)]
pub struct FsDirectoryEntry {
    pub(super) name: [u8; 0x20c],
    pub(super) short_name: [u8; 0xa],
    pub(super) short_ext: [u8; 0x4],
    pub(super) valid: u8,
    pub(super) reserved: u8,
    pub(super) attributes: u32,
    pub(super) file_size: u64,
}

impl FsDirectoryEntry {
    pub fn name(&self) -> &[u8] {
        &self.name
    }

    pub fn file_size(&self) -> u64 {
        self.file_size
    }
}

pub mod dir {
    use no_std_io::Reader;

    use crate::ipc::PermissionBuffer;

    use super::*;

    pub fn close(handle: &Handle) -> CtrResult {
        let raw_handle = unsafe { handle.get_raw() };
        Command::new(0x8020000, ()).send::<()>(raw_handle)
    }

    #[derive(EndianRead, EndianWrite)]
    struct FsReadDirIn {
        max_entry_count: u32,
        out_buffer: PermissionBuffer,
    }

    pub fn read_next_entry(handle: &Handle) -> CtrResult<Option<FsDirectoryEntry>> {
        let mut out_buffer: [u8; 552] = [0; 552];
        let input = FsReadDirIn {
            max_entry_count: 1,
            out_buffer: PermissionBuffer::new_write(&mut out_buffer),
        };
        let raw_handle = unsafe { handle.get_raw() };
        let entries_read: u32 = Command::new(0x8010042, input).send(raw_handle)?;

        if entries_read == 0 {
            return Ok(None);
        }

        let entry = out_buffer.read_le::<FsDirectoryEntry>(0)?;
        Ok(Some(entry))
    }
}
