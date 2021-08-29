#[cfg(any(not(debug_assertions), not(target_os = "horizon")))]
pub fn log(_text: &str) {}

#[cfg(all(debug_assertions, target_os = "horizon"))]
use crate::{
    fs::{ArchiveId, File, FsPath, OpenFlags},
    ipc::get_thread_command_buffer,
    os::get_time,
};
#[cfg(all(debug_assertions, target_os = "horizon"))]
use alloc::format;
#[cfg(all(debug_assertions, target_os = "horizon"))]
use core::{convert::TryInto, ptr::copy_nonoverlapping};

/// Logs text to the SD card at /ctr-logs.txt.
#[cfg(all(debug_assertions, target_os = "horizon"))]
pub fn log(text: &str) {
    let cmd_buf = get_thread_command_buffer();
    let mut command_cache: [u32; 25] = [0; 25];

    // This is safe because the command buffer and the cache are aligned,
    // valid for 25 u32 reads/writes, and don't overlap
    unsafe { copy_nonoverlapping(cmd_buf.as_ptr(), command_cache.as_mut_ptr(), 25) };

    let archive_path: FsPath = "".try_into().unwrap();
    let file_path: FsPath = "/ctr-logs.txt".try_into().unwrap();
    if let Ok(mut file) = File::new(
        &archive_path,
        &file_path,
        ArchiveId::Sdmc,
        OpenFlags::ReadWriteCreate,
    ) {
        let new_line_text = format!("{} {}\n", get_time(), text);
        file.write_str(&new_line_text).unwrap();
    }

    // This is safe because the command buffer and the cache are aligned,
    // valid for 25 u32 reads/writes, and don't overlap
    unsafe { copy_nonoverlapping(command_cache.as_ptr(), cmd_buf.as_mut_ptr(), 25) };
}
