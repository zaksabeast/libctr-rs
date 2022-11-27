use super::Path;
use crate::{
    fs::{DirEntry, FsArchive, FsDirectory, OpenFlags},
    res::CtrResult,
};
use alloc::vec::Vec;
use core::iter::Iterator;

/// A convenience function to read an entire file into a vector.
pub fn read(path: impl Into<Path>) -> CtrResult<Vec<u8>> {
    let path = path.into();
    let archive = FsArchive::new(path.archive_id, &path.archive_path)?;
    let file = archive.open_file(&path.file_path, OpenFlags::Read)?;
    let file_size = file.size()?;
    file.read(0, file_size)
}

/// A convenience function to write data to a file.
pub fn write(path: impl Into<Path>, contents: impl AsRef<[u8]>) -> CtrResult {
    let path = path.into();
    let archive = FsArchive::new(path.archive_id, &path.archive_path)?;
    let mut file = archive.open_file(&path.file_path, OpenFlags::Write)?;
    file.write(contents.as_ref())
}

pub fn create_dir(path: impl Into<Path>) -> CtrResult {
    let path = path.into();
    let archive = FsArchive::new(path.archive_id, &path.archive_path)?;
    archive.create_directory(&path.file_path, 0)
}

pub struct ReadDir {
    directory: FsDirectory,
}

impl Iterator for ReadDir {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.directory.read_next().ok().flatten()
    }
}

pub fn read_dir(path: impl Into<Path>) -> CtrResult<ReadDir> {
    let path = path.into();
    let archive = FsArchive::new(path.archive_id, &path.archive_path)?;
    let directory = archive.open_directory(&path.file_path)?;
    Ok(ReadDir { directory })
}
