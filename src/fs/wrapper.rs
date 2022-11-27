use super::ipc::{file, user, ArchiveId, FsPath, OpenFlags, WriteFlags};
use crate::{ipc::Command, res::CtrResult, Handle};
use alloc::vec::Vec;
use core::{convert::TryInto, ops::Drop};

/// Opens a file.
/// The file is closed automatically when dropped.
pub struct File {
    handle: Handle,
    file_size: u64,
}

impl File {
    fn new_from_handle(handle: Handle) -> CtrResult<Self> {
        let file_size = file::get_size(&handle)?;
        Ok(File { handle, file_size })
    }

    fn new_from_archive(archive: &FsArchive, path: &FsPath, flags: OpenFlags) -> CtrResult<Self> {
        let handle = user::open_file(archive.raw_archive_handle, path, flags, 0)?;
        Self::new_from_handle(handle)
    }

    pub fn new(
        archive_path: &FsPath,
        file_path: &FsPath,
        archive_id: ArchiveId,
        flags: OpenFlags,
    ) -> CtrResult<Self> {
        let handle = user::open_file_directly(archive_id, archive_path, file_path, flags, 0)?;
        Self::new_from_handle(handle)
    }

    pub fn write_str(&mut self, text: &str) -> CtrResult {
        self.write(text.into())
    }

    pub fn write(&mut self, data: Vec<u8>) -> CtrResult {
        let mut total_written_bytes = 0;
        let bytes_to_write = data.len();

        loop {
            let bytes_written =
                file::write(&self.handle, self.file_size, &data, WriteFlags::Flush)?;

            total_written_bytes += bytes_written;
            self.file_size += bytes_written as u64;

            if total_written_bytes == bytes_to_write {
                break;
            }
        }

        Ok(())
    }

    pub fn read(&self, offset: u64, max_size: usize) -> CtrResult<Vec<u8>> {
        let buffer = file::read(&self.handle, offset, max_size)?;
        Ok(buffer)
    }

    pub fn size(&self) -> CtrResult<usize> {
        let size = file::get_size(&self.handle)?.try_into()?;
        Ok(size)
    }
}

impl Drop for File {
    // file::Close in libctru
    // If this fails, there's not much to recover from
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        let raw_handle = unsafe { self.handle.get_raw() };
        Command::new(0x8080000, ()).send::<()>(raw_handle);
        // The handle will close when this is dropped
    }
}

/// Opens an archive containing directories and files.
/// The archive is closed when dropped.
pub struct FsArchive {
    raw_archive_handle: u64,
}

impl FsArchive {
    pub fn new(id: ArchiveId, path: &FsPath) -> CtrResult<Self> {
        let raw_archive_handle = user::open_archive(id, path)?;
        Ok(Self { raw_archive_handle })
    }

    pub fn open_file(&self, path: &FsPath, flags: OpenFlags) -> CtrResult<File> {
        File::new_from_archive(self, path, flags)
    }

    pub fn rename_directory(&self, src_path: &FsPath, dst_path: &FsPath) -> CtrResult {
        user::rename_directory(self.raw_archive_handle, src_path, dst_path)
    }
}

impl Drop for FsArchive {
    // If this fails, there's not much to recover from
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        user::close_archive(self.raw_archive_handle);
    }
}
