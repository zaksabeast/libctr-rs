use super::ipc::{dir, file, user, ArchiveId, FsPath, OpenFlags, WriteFlags};
use crate::{res::CtrResult, utils::convert::bytes_to_utf16le_string, Handle};
use alloc::{string::String, vec::Vec};
use core::{convert::TryInto, ops::Drop};

/// Opens a file.
/// The file is closed automatically when dropped.
pub struct File {
    handle: Handle,
}

impl File {
    fn new_from_handle(handle: Handle) -> Self {
        Self { handle }
    }

    fn new_from_archive(archive: &FsArchive, path: &FsPath, flags: OpenFlags) -> CtrResult<Self> {
        let handle = user::open_file(archive.raw_archive_handle, path, flags, 0)?;
        Ok(Self::new_from_handle(handle))
    }

    pub fn new(
        archive_path: &FsPath,
        file_path: &FsPath,
        archive_id: ArchiveId,
        flags: OpenFlags,
    ) -> CtrResult<Self> {
        let handle = user::open_file_directly(archive_id, archive_path, file_path, flags, 0)?;
        Ok(Self::new_from_handle(handle))
    }

    pub fn write_str(&mut self, text: &str) -> CtrResult {
        self.write(text.as_bytes())
    }

    pub fn write(&mut self, data: &[u8]) -> CtrResult {
        let mut total_written_bytes = 0;
        let bytes_to_write = data.len();

        loop {
            let bytes_written =
                file::write(&self.handle, self.size()? as u64, data, WriteFlags::Flush)?;

            total_written_bytes += bytes_written;

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
        file::close(&self.handle);
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

    pub fn create_directory(&self, path: &FsPath, attributes: u32) -> CtrResult {
        user::create_directory(self.raw_archive_handle, path, attributes)
    }

    pub fn rename_directory(&self, src_path: &FsPath, dst_path: &FsPath) -> CtrResult {
        user::rename_directory(self.raw_archive_handle, src_path, dst_path)
    }

    pub fn open_directory(&self, path: &FsPath) -> CtrResult<FsDirectory> {
        let handle = user::open_directory(self.raw_archive_handle, path)?;
        Ok(FsDirectory::new_from_handle(handle))
    }
}

impl Drop for FsArchive {
    // If this fails, there's not much to recover from
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        user::close_archive(self.raw_archive_handle);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    name: String,
    file_size: u64,
}

impl DirEntry {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> u64 {
        self.file_size
    }
}

pub struct FsDirectory {
    handle: Handle,
}

impl FsDirectory {
    fn new_from_handle(handle: Handle) -> Self {
        Self { handle }
    }

    /// Reads the next directory entry.
    pub(super) fn read_next(&self) -> CtrResult<Option<DirEntry>> {
        let dir_entry = dir::read_next_entry(&self.handle)?;

        if let Some(entry) = dir_entry {
            return Ok(Some(DirEntry {
                name: bytes_to_utf16le_string(entry.name())?,
                file_size: entry.file_size(),
            }));
        }

        Ok(None)
    }
}

impl Drop for FsDirectory {
    // If this fails, there's not much to recover from
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        dir::close(&self.handle);
    }
}
