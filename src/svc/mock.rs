use super::{
    DebugFlag, EventResetType, MemInfo, MemQueryResponse, PageInfo, ProcessInfoType, UserBreakType,
};
use crate::{
    memory::MemoryPermission,
    res::{CtrResult, GenericResultCode, ResultCode},
    Handle,
};
use alloc::{vec, vec::Vec};

pub fn break_execution(_reason: UserBreakType) -> ! {
    panic!()
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn accept_session(_port: &Handle) -> CtrResult<Handle> {
    Ok(0.into())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn send_raw_sync_request(_raw_handle: u32) -> CtrResult {
    Ok(())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn close_handle(_handle: u32) -> CtrResult {
    Ok(())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn reply_and_receive(
    _raw_handles: &[u32],
    _reply_target: Option<usize>,
) -> (usize, ResultCode) {
    (0, ResultCode::success())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn create_event(_reset_type: EventResetType) -> CtrResult<Handle> {
    Ok(0.into())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn sleep_thread(_nanoseconds: i64) {}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn signal_event(event: &Handle) -> CtrResult {
    Ok(())
}

pub fn exit_process() -> ! {
    loop {}
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn create_memory_block(
    _slice: &mut [u8],
    _my_permission: MemoryPermission,
    _other_process_permission: MemoryPermission,
) -> CtrResult<Handle> {
    Ok(0.into())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn unmap_memory_block(_memory_block_handle: &Handle, _slice: &mut [u8]) -> CtrResult {
    Ok(())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn wait_synchronization(_handle: &Handle, _wait_nanoseconds: i64) -> CtrResult {
    Ok(())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn get_process_list() -> CtrResult<Vec<u32>> {
    Ok(vec![0; 0x40])
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn open_process(_process_id: u32) -> CtrResult<Handle> {
    Ok(0.into())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn debug_active_process(_process_id: u32) -> CtrResult<Handle> {
    Ok(0.into())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn read_process_memory(_debug_process: &Handle, _addr: u32, size: u32) -> CtrResult<Vec<u8>> {
    Ok(vec![0; size as usize])
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn write_process_memory(_debug_process: &Handle, _buffer: &[u8], _addr: u32) -> CtrResult {
    Ok(())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn continue_debug_event(_debug_process: &Handle, _flag: DebugFlag) -> CtrResult {
    Ok(())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn get_process_debug_event(_debug_process: &Handle) -> ResultCode {
    ResultCode::success()
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn eat_events(_debug_process: &Handle) -> CtrResult {
    Ok(())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn get_process_info(_process: &Handle, _info_type: ProcessInfoType) -> CtrResult<i64> {
    Ok(0)
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn copy_handle(
    _out_process: &Handle,
    _input: &Handle,
    _in_process: &Handle,
) -> CtrResult<Handle> {
    Ok(0.into())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn query_debug_process_memory(
    _debug_process: &Handle,
    _addr: u32,
) -> CtrResult<MemQueryResponse> {
    Ok(MemQueryResponse {
        mem_info: MemInfo {
            base_addr: 0,
            size: 0,
            perm: 0,
            state: 0,
        },
        page_info: PageInfo { flags: 0 },
    })
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn convert_va_to_pa(_virtual_addr: *mut u8, _write_check: bool) -> CtrResult<*mut u8> {
    // Return an error by default, because we don't want to return a pointer.
    Err(GenericResultCode::InvalidPointer.into())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn convert_pa_to_uncached_pa(_physical_addr: *mut u8) -> CtrResult<*mut u8> {
    // Return an error by default, because we don't want to return a pointer.
    Err(GenericResultCode::InvalidPointer.into())
}
