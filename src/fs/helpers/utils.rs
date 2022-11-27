use super::Path;
use crate::{
    fs::{FsArchive, OpenFlags},
    res::CtrResult,
};
use alloc::vec::Vec;

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
