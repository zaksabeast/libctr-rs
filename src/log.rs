#[cfg(target_os = "horizon")]
use crate::{
    fs::{ArchiveId, File, FsPath, OpenFlags},
    ipc::get_thread_command_buffer,
    os::get_time,
};
#[cfg(target_os = "horizon")]
use alloc::format;
use alloc::string::{String, ToString};
use core::fmt;
#[cfg(target_os = "horizon")]
use core::{convert::TryInto, ptr::copy_nonoverlapping};

#[derive(Clone, Copy, Debug)]
enum LogType {
    #[cfg(debug_assertions)]
    Debug,
    Error,
}

impl fmt::Display for LogType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(not(target_os = "horizon"))]
fn log(_file_name: &str, _log_type: LogType, _text: &str) {}

/// Logs text to the SD card in a file at the root of the sd card.
#[cfg(target_os = "horizon")]
fn log(file_name: &str, log_type: LogType, text: &str) {
    let cmd_buf = get_thread_command_buffer();
    let mut command_cache: [u32; 25] = [0; 25];

    // This is safe because the command buffer and the cache are aligned,
    // valid for 25 u32 reads/writes, and don't overlap
    unsafe { copy_nonoverlapping(cmd_buf.as_ptr(), command_cache.as_mut_ptr(), 25) };

    let archive_path: FsPath = "".try_into().unwrap();
    let file_path: FsPath = file_name.try_into().unwrap();
    if let Ok(mut file) = File::new(
        &archive_path,
        &file_path,
        ArchiveId::Sdmc,
        OpenFlags::ReadWriteCreate,
    ) {
        let new_line_text = format!("[{}] {}: {}\n", log_type, get_time(), text);
        file.write_str(&new_line_text).unwrap();
    }

    // This is safe because the command buffer and the cache are aligned,
    // valid for 25 u32 reads/writes, and don't overlap
    unsafe { copy_nonoverlapping(command_cache.as_ptr(), cmd_buf.as_mut_ptr(), 25) };
}

pub struct Logger {
    file_name: String,
}

impl Logger {
    pub fn new(file_name: &str) -> Self {
        Self {
            file_name: file_name.to_string(),
        }
    }

    #[cfg(debug_assertions)]
    pub fn debug(&self, text: &str) {
        log(&self.file_name, LogType::Debug, text)
    }

    #[cfg(not(debug_assertions))]
    pub fn debug(&self, _text: &str) {}

    pub fn error(&self, text: &str) {
        log(&self.file_name, LogType::Error, text)
    }
}
