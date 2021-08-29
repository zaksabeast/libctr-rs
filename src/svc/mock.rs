use super::{
    DebugFlag, EventResetType, MemInfo, MemQueryResponse, PageInfo, ProcessInfoType, UserBreakType,
};
use crate::{
    memory::MemoryPermission,
    res::{CtrResult, ResultCode},
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
pub fn send_raw_sync_request(_raw_handle: u32) -> CtrResult<ResultCode> {
    Ok(0)
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn close_handle(_handle: u32) -> CtrResult<ResultCode> {
    Ok(0)
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn reply_and_receive(
    _raw_handles: &[u32],
    _reply_target: Option<usize>,
) -> (usize, ResultCode) {
    (0, 0)
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn create_event(_reset_type: EventResetType) -> CtrResult<Handle> {
    Ok(0.into())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn sleep_thread(_nanoseconds: i64) {}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn signal_event(event: &Handle) -> CtrResult<ResultCode> {
    Ok(0)
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn exit_process() {}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn create_memory_block(
    _slice: &mut [u8],
    _my_permission: MemoryPermission,
    _other_process_permission: MemoryPermission,
) -> CtrResult<Handle> {
    Ok(0.into())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn unmap_memory_block(
    _memory_block_handle: &Handle,
    _slice: &mut [u8],
) -> CtrResult<ResultCode> {
    Ok(0)
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn wait_synchronization(_handle: &Handle, _wait_nanoseconds: i64) -> CtrResult<ResultCode> {
    Ok(0)
}

pub fn get_process_list() -> CtrResult<Vec<u32>> {
    Ok(vec![0; 0x40])
}

pub fn open_process(_process_id: u32) -> CtrResult<Handle> {
    Ok(0.into())
}

pub fn debug_active_process(_process_id: u32) -> CtrResult<Handle> {
    Ok(0.into())
}

pub fn read_process_memory(_debug_process: &Handle, _addr: u32, size: u32) -> CtrResult<Vec<u8>> {
    Ok(vec![0; size as usize])
}

pub fn write_process_memory(_debug_process: &Handle, _buffer: &[u8], _addr: u32) -> CtrResult<()> {
    Ok(())
}

pub fn continue_debug_event(_debug_process: &Handle, _flag: DebugFlag) -> CtrResult<()> {
    Ok(())
}

pub fn get_process_debug_event(_debug_process: &Handle) -> ResultCode {
    0
}

// Thanks to Luma3ds
pub fn eat_events(_debug_process: &Handle) -> CtrResult<()> {
    Ok(())
}

pub fn get_process_info(_process: &Handle, _info_type: ProcessInfoType) -> CtrResult<i64> {
    Ok(0)
}

pub fn copy_handle(
    _out_process: &Handle,
    _input: &Handle,
    _in_process: &Handle,
) -> CtrResult<Handle> {
    Ok(0.into())
}

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
