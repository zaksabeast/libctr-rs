use crate::{
    ipc::ThreadCommandBuilder,
    res::{CtrResult, GenericResultCode},
    srv::get_service_handle_direct,
    svc,
    utils::convert::try_usize_into_u32,
};
use alloc::string::String;
use core::{
    mem::ManuallyDrop,
    sync::atomic::{AtomicU32, Ordering},
};
use safe_transmute::transmute_to_bytes_mut;

static CFG_HANDLE: AtomicU32 = AtomicU32::new(0);

fn get_handle() -> u32 {
    CFG_HANDLE.load(Ordering::Relaxed)
}

/// Initializes the CFG service. Required to use CFG features.
pub fn init() -> CtrResult {
    let handle = get_service_handle_direct("cfg:i")
        .or_else(|_| get_service_handle_direct("cfg:s"))
        .or_else(|_| get_service_handle_direct("cfg:u"))?;

    let dropped_handle = ManuallyDrop::new(handle);
    let raw_handle = unsafe { dropped_handle.get_raw() };
    CFG_HANDLE.store(raw_handle, Ordering::Relaxed);

    Ok(())
}

pub fn exit() -> CtrResult {
    let result = svc::close_handle(get_handle());

    if result.is_ok() {
        CFG_HANDLE.store(0, Ordering::Relaxed);
    }

    result
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
fn get_local_friend_code_seed_data_impl() -> CtrResult<[u8; 0x110]> {
    let mut out: [u8; 0x110] = [0; 0x110];
    let out_size = try_usize_into_u32(out.len())?;

    let mut command = ThreadCommandBuilder::new(0x404u16);
    command.push(out_size);
    command.push_write_buffer(&mut out);
    let mut parser = command
        .build()
        .send_sync_request_with_raw_handle(get_handle())?;

    parser.pop_result()?;
    Ok(out)
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn get_config_info_blk2(out: &mut [u8], block_id: u32) -> CtrResult {
    let out_size = try_usize_into_u32(out.len())?;

    let mut command = ThreadCommandBuilder::new(0x1u16);
    command.push(out_size);
    command.push(block_id);
    command.push_write_buffer(out);

    let mut parser = command
        .build()
        .send_sync_request_with_raw_handle(get_handle())?;

    parser.pop_result()?;
    Ok(())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn get_local_friend_code_seed_data() -> CtrResult<[u8; 0x110]> {
    init()?;
    let result = get_local_friend_code_seed_data_impl();
    exit()?;

    result
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn get_console_username() -> CtrResult<String> {
    let mut username_shorts: [u16; 15] = [0; 15];
    let username_buffer = transmute_to_bytes_mut(&mut username_shorts[0..14]);
    init()?;
    get_config_info_blk2(username_buffer, 0xa0000)?;
    exit()?;

    let null_terminator_index = username_shorts
        .iter()
        // We can unwrap here, because
        // we guaranteed a null terminator above
        .position(|short| *short == 0u16)
        .unwrap();

    String::from_utf16(&username_shorts[0..null_terminator_index])
        .map_err(|_| GenericResultCode::InvalidString.into())
}
