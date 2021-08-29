use crate::{
    ipc::{ThreadCommandBuilder, ThreadCommandParser},
    res::{CtrResult, GenericResultCode, ResultCode},
    srv::get_service_handle_direct,
    utils::cstring,
    Handle,
};
use alloc::{str, vec, vec::Vec};
use core::{
    convert::{Into, TryFrom},
    mem::ManuallyDrop,
    sync::atomic::{AtomicU32, Ordering},
};
use cstr_core::CString;
use num_enum::IntoPrimitive;
use safe_transmute::transmute_to_bytes;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C)]
pub struct ProductInfo {
    pub product_code: [u8; 0x10],
    pub company_code: [u8; 2],
    pub remaster_version: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum MediaType {
    Nand = 0,
    Sd = 1,
    GameCard = 2,
}

impl From<u32> for MediaType {
    fn from(raw_media_type: u32) -> Self {
        match raw_media_type {
            0 => MediaType::Nand,
            1 => MediaType::Sd,
            2 => MediaType::GameCard,
            _ => MediaType::Nand,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(IntoPrimitive)]
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

/// A path used to open archives and files.
/// Binary paths are created from vectors and Ascii/Empty paths are created from strings.
pub enum FsPath {
    Empty(Vec<u8>),
    Binary(Vec<u32>),
    Ascii(Vec<u8>),
}

impl FsPath {
    pub fn new_empty_path() -> Self {
        Self::Empty(vec![])
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
            Self::Empty(_) => 0,
            Self::Binary(path) => path.len() * 4,
            Self::Ascii(path) => path.len() + 1,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_inner(&self) -> &[u8] {
        match self {
            Self::Empty(path) => &path,
            Self::Binary(path) => transmute_to_bytes(&path),
            Self::Ascii(path) => &path,
        }
    }
}

impl From<Vec<u32>> for FsPath {
    fn from(path: Vec<u32>) -> Self {
        Self::Binary(path)
    }
}

impl TryFrom<&str> for FsPath {
    type Error = ResultCode;

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        if path.is_empty() {
            Ok(Self::new_empty_path())
        } else {
            let c_path = cstring::parse_result(CString::new(path))?;
            Ok(Self::Ascii(c_path.into_bytes_with_nul()))
        }
    }
}

static FS_HANDLE: AtomicU32 = AtomicU32::new(0);

fn get_handle() -> u32 {
    FS_HANDLE.load(Ordering::Relaxed)
}

/// Initializes the FS service. Required to use FS features.
pub fn init() -> CtrResult<ResultCode> {
    let fs_handle = get_service_handle_direct("fs:USER")?;

    user::initialize_with_sdk_version(&fs_handle, 0x90c00c8)?;

    let dropped_fs_handle = ManuallyDrop::new(fs_handle);
    let raw_fs_handle = unsafe { dropped_fs_handle.get_raw() };
    FS_HANDLE.store(raw_fs_handle, Ordering::Relaxed);

    user::set_priority(0)?;

    Ok(0)
}

pub mod user {
    use super::*;

    pub fn set_priority(priority: u32) -> CtrResult<ResultCode> {
        let mut command = ThreadCommandBuilder::new(0x862u16);
        command.push(priority);

        let mut parser = command
            .build()
            .send_sync_request_with_raw_handle(get_handle())?;
        parser.pop_result()
    }

    pub fn initialize_with_sdk_version(session: &Handle, version: u32) -> CtrResult<ResultCode> {
        let mut command = ThreadCommandBuilder::new(0x861u16);
        command.push(version);
        command.push_curent_process_id();

        let mut parser = command.build().send_sync_request(session)?;
        parser.pop_result()
    }

    pub fn open_file_directly(
        archive_id: ArchiveId,
        archive_path: &FsPath,
        file_path: &FsPath,
        flags: OpenFlags,
        attributes: u32,
    ) -> CtrResult<Handle> {
        let mut command = ThreadCommandBuilder::new(0x803u16);

        command.push(0u32);
        command.push(archive_id);
        command.push(archive_path.get_raw_type());
        command.push(archive_path.len() as u32);
        command.push(file_path.get_raw_type());
        command.push(file_path.len() as u32);
        command.push(flags);
        command.push(attributes);
        command.push_static_buffer(&archive_path.get_inner(), 2);
        command.push_static_buffer(&file_path.get_inner(), 0);

        let mut parser = command
            .build()
            .send_sync_request_with_raw_handle(get_handle())?;
        parser.pop_result()?;
        parser.pop();

        Ok(parser.pop().into())
    }

    pub fn open_file(
        raw_archive_handle: u64,
        path: &FsPath,
        flags: OpenFlags,
        attributes: u32,
    ) -> CtrResult<Handle> {
        let mut command = ThreadCommandBuilder::new(0x802u16);
        command.push(0u32);
        command.push_u64(raw_archive_handle);
        command.push(path.get_raw_type());
        command.push(path.len() as u32);
        command.push(flags);
        command.push(attributes);
        command.push_static_buffer(path.get_inner(), 0);

        let mut parser = command
            .build()
            .send_sync_request_with_raw_handle(get_handle())?;
        parser.pop_result()?;
        parser.pop();

        Ok(parser.pop().into())
    }

    /// Opens an archive and returns the raw archive handle.
    pub fn open_archive(id: ArchiveId, path: &FsPath) -> CtrResult<u64> {
        let mut command = ThreadCommandBuilder::new(0x80cu16);
        command.push(id);
        command.push(path.get_raw_type());
        command.push(path.len() as u32);
        command.push_static_buffer(path.get_inner(), 0);

        let mut parser = command
            .build()
            .send_sync_request_with_raw_handle(get_handle())?;
        parser.pop_result()?;

        Ok(parser.pop_u64())
    }

    pub fn close_archive(raw_archive_handle: u64) -> CtrResult<ResultCode> {
        if raw_archive_handle == 0 {
            return Err(GenericResultCode::NoArchive.into());
        }

        let mut command = ThreadCommandBuilder::new(0x80eu16);
        command.push_u64(raw_archive_handle);

        let mut parser = command
            .build()
            .send_sync_request_with_raw_handle(get_handle())?;
        parser.pop_result()
    }

    pub fn rename_directory(
        raw_archive_handle: u64,
        src_path: &FsPath,
        dst_path: &FsPath,
    ) -> CtrResult<ResultCode> {
        let mut command = ThreadCommandBuilder::new(0x80Au16);

        command.push(0u32);
        command.push_u64(raw_archive_handle);
        command.push(src_path.get_raw_type());
        command.push(src_path.len() as u32);
        command.push_u64(raw_archive_handle);
        command.push(dst_path.get_raw_type());
        command.push(dst_path.len() as u32);
        command.push_static_buffer(&src_path.get_inner(), 1);
        command.push_static_buffer(&dst_path.get_inner(), 2);

        let mut parser = command
            .build()
            .send_sync_request_with_raw_handle(get_handle())?;

        parser.pop_result()
    }

    #[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
    pub fn get_program_launch_info(process_id: u32) -> CtrResult<ProgramInfo> {
        let mut command = ThreadCommandBuilder::new(0x82Fu16);
        command.push(process_id);

        command
            .build()
            .send_sync_request_with_raw_handle(get_handle())?;

        let mut parser = ThreadCommandParser::new();
        parser.pop_result()?;

        Ok(ProgramInfo {
            program_id: parser.pop_u64(),
            media_type: parser.pop().into(),
            ..Default::default()
        })
    }

    #[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
    pub fn get_product_info(process_id: u32) -> CtrResult<ProductInfo> {
        let mut command = ThreadCommandBuilder::new(0x82Eu16);
        command.push(process_id);

        command
            .build()
            .send_sync_request_with_raw_handle(get_handle())?;

        let mut parser = ThreadCommandParser::new();
        parser.pop_result()?;

        let product_code = parser.pop_struct::<[u8; 16]>()?;
        let [company_code, remaster_version] = parser.pop_struct::<[u16; 2]>()?;

        Ok(ProductInfo {
            product_code,
            company_code: company_code.to_ne_bytes(),
            remaster_version,
        })
    }
}

pub mod file {
    use super::*;

    pub fn get_size(handle: &Handle) -> CtrResult<u64> {
        let command = ThreadCommandBuilder::new(0x804u16);

        let mut parser = command.build().send_sync_request(handle)?;
        parser.pop_result()?;

        Ok(parser.pop_u64())
    }

    pub fn write(
        handle: &Handle,
        offset: u64,
        buffer: &[u8],
        flags: WriteFlags,
    ) -> CtrResult<usize> {
        let mut command = ThreadCommandBuilder::new(0x803u16);
        command.push_u64(offset);
        command.push(buffer.len() as u32);
        command.push(flags);
        command.push_read_buffer(&buffer);

        let mut parser = command.build().send_sync_request(handle)?;
        parser.pop_result()?;

        Ok(parser.pop() as usize)
    }

    /// Reads from a file.
    /// The output size is not guaranteed to be the max read size.
    pub fn read(handle: &Handle, offset: u64, max_read_size: usize) -> CtrResult<Vec<u8>> {
        let mut out_buffer: Vec<u8> = vec![0; max_read_size];
        let mut command = ThreadCommandBuilder::new(0x802u16);
        command.push_u64(offset);
        command.push(max_read_size as u32);
        command.push_write_buffer(&mut out_buffer);

        let mut parser = command.build().send_sync_request(handle)?;
        parser.pop_result()?;

        let bytes_read = parser.pop() as usize;
        out_buffer.resize(bytes_read, 0);

        Ok(out_buffer)
    }
}
