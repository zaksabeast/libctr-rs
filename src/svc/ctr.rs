use super::{DebugFlag, EventResetType, MemQueryResponse, ProcessInfoType, UserBreakType};
use crate::{
    memory::MemoryPermission,
    res::{parse_result, CtrResult, GenericResultCode, ResultCode},
    Handle,
};
use alloc::{vec, vec::Vec};
use core::{arch::asm, convert::TryInto, ffi::c_void, intrinsics::transmute};
use ctru_sys::{
    svcAcceptSession, svcContinueDebugEvent, svcCreateEvent, svcCreateMemoryBlock,
    svcDebugActiveProcess, svcExitProcess, svcGetProcessDebugEvent, svcGetProcessInfo,
    svcGetProcessList, svcOpenProcess, svcQueryDebugProcessMemory, svcReadProcessMemory,
    svcReplyAndReceive, svcSignalEvent, svcSleepThread, svcUnmapMemoryBlock,
    svcWaitSynchronization, svcWriteProcessMemory, DebugEventInfo,
};

#[inline(never)]
/// Sends a sync request.
/// This is often used with atomic handles, which are u32s instead of Handles.
/// As a result, this takes a u32 to be more generic, and to avoid converting a u32 to a Handle, then immediately back into a u32.
pub fn control_service(service_op: u32, service_name: &str) -> CtrResult<Handle> {
    let mut result_code;
    let mut handle: u32 = 0;

    unsafe {
        asm!("svc 0xB0", in("r0") service_op, in("r1") &mut handle, in("r2") service_name.as_ptr(), lateout("r0") result_code);
    }

    parse_result(result_code)?;
    Ok(handle.into())
}

#[inline(never)]
/// Sends a sync request.
/// This is often used with atomic handles, which are u32s instead of Handles.
/// As a result, this takes a u32 to be more generic, and to avoid converting a u32 to a Handle, then immediately back into a u32.
pub fn send_raw_sync_request(raw_handle: u32) -> CtrResult {
    let mut result_code;

    unsafe {
        asm!("svc 0x32", in("r0") raw_handle, lateout("r0") result_code);
    }

    parse_result(result_code)
}

#[inline(never)]
/// Closes a handle.
/// This is pretty much only for implementing Drop on Handle.
/// If you're thinking about using this, consider using a Handle and let it manage closing the underlying handle.
pub fn close_handle(handle: u32) -> CtrResult {
    let mut result_code;

    unsafe {
        asm!("svc 0x23", in("r0") handle, lateout("r0") result_code);
    }

    parse_result(result_code)
}

#[inline(never)]
/// Breaks execution.
pub fn break_execution(reason: UserBreakType) -> ! {
    unsafe {
        asm!("svc 0x3C", in("r0") reason as u32);
    }

    // Allow the empty loop to get the 'never' return type
    // We'll never reach this far because the above will break anyways
    #[allow(clippy::empty_loop)]
    loop {}
}

/// Accepts a session to a service.
pub fn accept_session(port: &Handle) -> CtrResult<Handle> {
    let mut raw_handle = 0;
    let result = unsafe { svcAcceptSession(&mut raw_handle, port.get_raw()) };

    parse_result(result)?;

    Ok(raw_handle.into())
}

/// Replies to a request and receives a new request.
pub fn reply_and_receive(raw_handles: &[u32], reply_target: Option<usize>) -> (usize, ResultCode) {
    let raw_reply_target_handle = match reply_target {
        Some(target_index) => raw_handles[target_index],
        None => 0,
    };

    let mut index = -1;

    let result = unsafe {
        svcReplyAndReceive(
            &mut index,
            raw_handles.as_ptr(),
            // If the handle count is wrong, there's not much we can do to recover
            raw_handles.len().try_into().unwrap(),
            raw_reply_target_handle,
        )
        .into()
    };

    (index as usize, result)
}

pub fn create_event(reset_type: EventResetType) -> CtrResult<Handle> {
    let mut raw_handle = 0;
    let result = unsafe { svcCreateEvent(&mut raw_handle, reset_type as u32) };

    parse_result(result)?;

    Ok(raw_handle.into())
}

pub fn sleep_thread(nanoseconds: i64) {
    unsafe { svcSleepThread(nanoseconds) }
}

pub fn signal_event(event: &Handle) -> CtrResult {
    let result = unsafe { svcSignalEvent(event.get_raw()) };
    parse_result(result)
}

pub fn exit_process() {
    unsafe { svcExitProcess() }
}

pub fn create_memory_block(
    slice: &mut [u8],
    my_permission: MemoryPermission,
    other_process_permission: MemoryPermission,
) -> CtrResult<Handle> {
    // Check alignment
    // svc::create_memory_block can only take alignments of 0x1000
    if (slice.as_ptr() as u32 & (0x1000 - 1)) != 0 {
        return Err(GenericResultCode::AlignmentError.into());
    }

    let mut handle: u32 = 0;
    let result = unsafe {
        svcCreateMemoryBlock(
            &mut handle,
            slice.as_mut_ptr() as u32,
            slice.len() as u32,
            my_permission as u32,
            other_process_permission as u32,
        )
    };

    parse_result(result)?;

    Ok(handle.into())
}

pub fn unmap_memory_block(memory_block_handle: &Handle, slice: &[u8]) -> CtrResult {
    let result =
        unsafe { svcUnmapMemoryBlock(memory_block_handle.get_raw(), slice.as_ptr() as u32) };
    parse_result(result)
}

pub fn wait_synchronization(handle: &Handle, wait_nanoseconds: i64) -> CtrResult {
    let result = unsafe { svcWaitSynchronization(handle.get_raw(), wait_nanoseconds) };
    parse_result(result)
}

pub fn get_process_list() -> CtrResult<Vec<u32>> {
    let mut process_ids: Vec<u32> = vec![0; 0x40];
    let mut process_count = 0;
    let result = unsafe {
        svcGetProcessList(
            &mut process_count,
            process_ids.as_mut_ptr(),
            process_ids.len() as i32,
        )
    };

    parse_result(result)?;
    process_ids.truncate(process_count as usize);
    Ok(process_ids)
}

pub fn open_process(process_id: u32) -> CtrResult<Handle> {
    let mut raw_handle = 0;
    let result = unsafe { svcOpenProcess(&mut raw_handle, process_id) };

    parse_result(result)?;
    Ok(raw_handle.into())
}

pub fn debug_active_process(process_id: u32) -> CtrResult<Handle> {
    let mut raw_handle = 0u32;
    let result = unsafe { svcDebugActiveProcess(&mut raw_handle, process_id) };

    parse_result(result)?;
    Ok(raw_handle.into())
}

pub fn read_process_memory(debug_process: &Handle, addr: u32, size: u32) -> CtrResult<Vec<u8>> {
    let mut buffer = vec![0; size as usize];
    let result = unsafe {
        svcReadProcessMemory(
            buffer.as_mut_ptr() as *mut c_void,
            debug_process.get_raw(),
            addr,
            size,
        )
    };

    parse_result(result)?;
    Ok(buffer)
}

pub fn write_process_memory(debug_process: &Handle, buffer: &[u8], addr: u32) -> CtrResult {
    let result = unsafe {
        svcWriteProcessMemory(
            debug_process.get_raw(),
            buffer.as_ptr() as *const c_void,
            addr,
            buffer.len() as u32,
        )
    };

    parse_result(result)?;
    Ok(())
}

pub fn continue_debug_event(debug_process: &Handle, flag: DebugFlag) -> CtrResult {
    let result = unsafe { svcContinueDebugEvent(debug_process.get_raw(), flag.into()) };

    parse_result(result)?;
    Ok(())
}

// TODO: Implement for reals.  This is hacked together for now.
pub fn get_process_debug_event(debug_process: &Handle) -> ResultCode {
    let mut info: [u8; 0x28] = [0; 0x28];
    unsafe {
        svcGetProcessDebugEvent(
            core::mem::transmute::<*mut u8, *mut DebugEventInfo>(info.as_mut_ptr()),
            debug_process.get_raw(),
        )
        .into()
    }
}

// Thanks to Luma3ds
pub fn eat_events(debug_process: &Handle) -> CtrResult {
    loop {
        if get_process_debug_event(debug_process) == 0xd8402009 {
            break;
        }
        continue_debug_event(
            debug_process,
            DebugFlag::InhibitUserCpuExceptionHandlersAndSignalFaultExceptionEvents,
        )?;
    }

    Ok(())
}

pub fn get_process_info(process: &Handle, info_type: ProcessInfoType) -> CtrResult<i64> {
    let mut out = 0;
    let result = unsafe { svcGetProcessInfo(&mut out, process.get_raw(), info_type as u32) };

    parse_result(result)?;

    Ok(out)
}

#[inline(never)]
pub fn copy_handle(out_process: &Handle, input: &Handle, in_process: &Handle) -> CtrResult<Handle> {
    let mut result;
    let mut out_handle = 0u32;
    unsafe {
        asm!(
            "
            str r0, [sp, #-4]!
            svc 0xB1
            ldr r2, [sp], #4
            str r1, [r2]
            ",
            in("r0") &mut out_handle,
            in("r1") out_process.get_raw(),
            in("r2") input.get_raw(),
            in("r3") in_process.get_raw(),
            lateout("r0") result
        )
    }

    parse_result(result)?;
    Ok(out_handle.into())
}

pub fn query_debug_process_memory(
    debug_process: &Handle,
    addr: u32,
) -> CtrResult<MemQueryResponse> {
    let mut mem_info = ctru_sys::MemInfo {
        base_addr: 0,
        perm: 0,
        size: 0,
        state: 0,
    };
    let mut page_info = ctru_sys::PageInfo { flags: 0 };
    let result = unsafe {
        svcQueryDebugProcessMemory(&mut mem_info, &mut page_info, debug_process.get_raw(), addr)
    };

    parse_result(result)?;
    Ok(MemQueryResponse {
        mem_info: unsafe { transmute::<ctru_sys::MemInfo, super::MemInfo>(mem_info) },
        page_info: unsafe { transmute::<ctru_sys::PageInfo, super::PageInfo>(page_info) },
    })
}

#[inline(never)]
/// Luma only.
/// Converts a virtual address into a physical address.
///
/// Returns an error if the pointer is invalid or if the caller
/// does not have permissions to the pointer.
pub fn convert_va_to_pa(virtual_addr: *mut u8, write_check: bool) -> CtrResult<*mut u8> {
    let mut physical_addr: *mut u8;

    unsafe {
        asm!("svc 0x90", in("r0") virtual_addr, in("r1") write_check as u32, lateout("r0") physical_addr)
    };

    if physical_addr.is_null() {
        return Err(GenericResultCode::InvalidPointer.into());
    }

    Ok(physical_addr)
}

/// Gets the uncached address of a physical address.
/// Returns an error if the pointer is null.
pub fn convert_pa_to_uncached_pa(physical_addr: *mut u8) -> CtrResult<*mut u8> {
    if physical_addr.is_null() {
        return Err(GenericResultCode::InvalidPointer.into());
    }

    let uncached_physical_addr = ((physical_addr as u32) | (1 << 31)) as *mut u8;

    Ok(uncached_physical_addr)
}
