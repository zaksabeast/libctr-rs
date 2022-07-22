#[cfg(target_os = "horizon")]
use crate::{
    fs::{ArchiveId, File, FsPath, OpenFlags},
    ipc::{backup_thread_command_buffer, restore_thread_command_buffer},
    os::get_time,
};
#[cfg(target_os = "horizon")]
use std::convert::TryInto;
use std::fmt;

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
    let command_cache: [u8; 0x64] = backup_thread_command_buffer();

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

    restore_thread_command_buffer(command_cache);
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
