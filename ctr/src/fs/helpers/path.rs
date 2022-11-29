use crate::{
    fs::{ArchiveId, FsPath},
    res::{error, CtrResult},
};
use alloc::string::String;

/// A file path.
/// Supported paths are `sd:/` for sd files and `syssave:/` for system save files.
/// For example:
/// - `sd:/3ds/file.txt`
/// - `syssave:/0000000000010032/1/friendlist`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    pub(super) archive_id: ArchiveId,
    pub(super) archive_path: FsPath,
    pub(super) file_path: FsPath,
}

impl Path {
    /// Creates a path and panics if the path is invalid.
    /// See [Path::new_checked] for valid path examples.
    pub fn new(path: impl AsRef<str>) -> Self {
        Self::new_checked(path).unwrap()
    }

    /// Creates a new path.  Returns an error if the path is invalid.
    ///
    /// A path is valid if it's formatted as `sd:/<directories/file>`
    /// or `syssave:/<save_id_path>/<directories/file>`.
    ///
    /// Note that a path can be valid even if a file does not exist.
    pub fn new_checked(path: impl AsRef<str>) -> CtrResult<Self> {
        let path = path.as_ref();

        // e.g. sd:/3ds/file.txt
        if path.starts_with("sd:/") && path.len() > 4 {
            return Ok(Self {
                archive_id: ArchiveId::Sdmc,
                archive_path: FsPath::new_empty_path(),
                // Need at least the '/' and one more character - 4 total
                file_path: path[3..].into(),
            });
        }

        // syssave:/0000000000010032/1/friendlist
        if path.starts_with("syssave:/") && path.len() > 26 {
            let archive_path = u64::from_str_radix(&path[9..25], 16)?;
            let archive_path_high = (archive_path >> 32) as u32;
            let archive_path_low = archive_path as u32;

            return Ok(Self {
                archive_id: ArchiveId::SystemSaveData,
                archive_path: FsPath::new_binary([archive_path_high, archive_path_low]),
                // Need at least the '/' and one more character - 4 total
                file_path: path[25..].into(),
            });
        }

        Err(error::invalid_value())
    }
}

impl From<&str> for Path {
    fn from(string: &str) -> Self {
        Self::new(string)
    }
}

impl From<&String> for Path {
    fn from(string: &String) -> Self {
        Self::new(string.as_str())
    }
}

impl From<String> for Path {
    fn from(string: String) -> Self {
        Self::new(string.as_str())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod new_checked {
        use super::*;

        #[test]
        fn should_parse_sd_path() {
            let result = Path::new_checked("sd:/3ds/test.txt").unwrap();
            assert_eq!(
                result,
                Path {
                    archive_id: ArchiveId::Sdmc,
                    archive_path: FsPath::new_empty_path(),
                    file_path: "/3ds/test.txt".into()
                }
            )
        }

        #[test]
        fn should_parse_syssave_path() {
            let result = Path::new_checked("syssave:/0000000000010032/1/friendlist").unwrap();
            assert_eq!(
                result,
                Path {
                    archive_id: ArchiveId::SystemSaveData,
                    archive_path: FsPath::new_binary([0x00000000, 0x00010032]),
                    file_path: "/1/friendlist".into()
                }
            )
        }

        #[test]
        fn should_error_if_only_sd_mount_is_provided() {
            let result = Path::new_checked("sd:/").unwrap_err();
            assert_eq!(result, error::invalid_value())
        }

        #[test]
        fn should_error_if_only_syssave_mount_is_provided() {
            let result = Path::new_checked("syssave:/").unwrap_err();
            assert_eq!(result, error::invalid_value())
        }

        #[test]
        fn should_error_if_syssave_first_directory_is_not_16_characters() {
            let result = Path::new_checked("syssave:/0").unwrap_err();
            assert_eq!(result, error::invalid_value())
        }

        #[test]
        fn should_error_if_syssave_first_directory_is_not_a_valid_u64() {
            let result = Path::new_checked("syssave:/000000000000test").unwrap_err();
            assert_eq!(result, error::invalid_value())
        }

        #[test]
        fn should_error_if_syssave_does_not_have_a_file_after_the_first_directory() {
            let result = Path::new_checked("syssave:/0000000000000000/").unwrap_err();
            assert_eq!(result, error::invalid_value())
        }
    }
}
