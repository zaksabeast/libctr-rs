use crate::{
    memory::MemoryPermission,
    res::{parse_result, CtrResult, GenericResultCode, ResultCode},
    Handle,
};
use alloc::{vec, vec::Vec};
use core::{convert::TryInto, ffi::c_void};
use cstr_core::CStr;
use num_enum::IntoPrimitive;

/// [Read more at the libctru docs](https://github.com/devkitPro/libctru/blob/4e25fb1d6c2ea124a9011c4b65f76f2968a9fb97/libctru/include/3ds/svc.h#L333-L337)
pub enum UserBreakType {
    Panic = 0,
    Assert = 1,
    User = 2,
    LoadRo = 3,
    UnloadRo = 4,
}

/// [Read more at the libctru docs](https://github.com/devkitPro/libctru/blob/4e25fb1d6c2ea124a9011c4b65f76f2968a9fb97/libctru/include/3ds/svc.h#L106-L108)
pub enum EventResetType {
    OneShot = 0,
    Sticky = 1,
    Pulse = 2,
}

#[derive(IntoPrimitive, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DebugFlag {
    InhibitUserCpuExceptionHandlers = 1,
    SignalFaultExceptionEvents = 2,
    InhibitUserCpuExceptionHandlersAndSignalFaultExceptionEvents = 3,
    SignalScheduleEvents = 4,
    SignalSyscallEvents = 8,
    SignalMapEvents = 16,
}

#[derive(IntoPrimitive, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ProcessInfoType {
    TitleId = 0x10001,
    StartAddress = 0x10005,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemInfo {
    pub base_addr: u32,
    pub size: usize,
    pub perm: u32,
    pub state: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageInfo {
    pub flags: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemQueryResponse {
    pub mem_info: MemInfo,
    pub page_info: PageInfo,
}

pub struct Port {
    pub(super) server: Handle,
    pub(super) client: Handle,
}

impl Port {
    pub fn server(&self) -> &Handle {
        &self.server
    }

    pub fn client(&self) -> &Handle {
        &self.client
    }

    pub fn into_handles(self) -> (Handle, Handle) {
        (self.server, self.client)
    }
}

/// Sends a sync request.
/// This is often used with atomic handles, which are u32s instead of Handles.
/// As a result, this takes a u32 to be more generic, and to avoid converting a u32 to a Handle, then immediately back into a u32.
#[inline(never)]
#[ctr_macros::hos]
pub fn send_raw_sync_request(raw_handle: u32) -> CtrResult {
    let mut result_code: i32;

    unsafe {
        core::arch::asm!("svc 0x32", in("r0") raw_handle, lateout("r0") result_code);
    }

    parse_result(result_code)
}

/// Closes a handle.
/// This is pretty much only for implementing Drop on Handle.
/// If you're thinking about using this, consider using a Handle and let it manage closing the underlying handle.
#[inline(never)]
#[cfg(target_os = "horizon")]
pub fn close_handle(handle: u32) -> CtrResult {
    let mut result_code: i32;

    unsafe {
        core::arch::asm!("svc 0x23", in("r0") handle, lateout("r0") result_code);
    }

    parse_result(result_code)
}

// This does not use ctr_macros::hos since closing a handle
// is exceptionally easy to do.
#[cfg(not(target_os = "horizon"))]
pub fn close_handle(_handle: u32) -> CtrResult {
    Ok(())
}

/// Breaks execution.
#[inline(never)]
#[ctr_macros::hos]
pub fn break_execution(reason: UserBreakType) -> ! {
    unsafe {
        core::arch::asm!("svc 0x3C", in("r0") reason as u32);
    }

    // Allow the empty loop to get the 'never' return type
    // We'll never reach this far because the above will break anyways
    #[allow(clippy::empty_loop)]
    loop {}
}

/// Accepts a session to a service.
#[ctr_macros::hos]
pub fn accept_session(port: &Handle) -> CtrResult<Handle> {
    let mut raw_handle = 0;
    let result = unsafe { ctru_sys::svcAcceptSession(&mut raw_handle, port.get_raw()) };

    parse_result(result)?;

    Ok(raw_handle.into())
}

/// Replies to a request and receives a new request.
#[ctr_macros::hos]
pub fn reply_and_receive(raw_handles: &[u32], reply_target: Option<usize>) -> (usize, ResultCode) {
    let raw_reply_target_handle = match reply_target {
        Some(target_index) => raw_handles[target_index],
        None => 0,
    };

    let mut index = -1;

    let result = unsafe {
        ctru_sys::svcReplyAndReceive(
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

#[ctr_macros::hos]
pub fn create_event(reset_type: EventResetType) -> CtrResult<Handle> {
    let mut raw_handle = 0;
    let result = unsafe { ctru_sys::svcCreateEvent(&mut raw_handle, reset_type as u32) };

    parse_result(result)?;

    Ok(raw_handle.into())
}

#[ctr_macros::hos]
pub fn sleep_thread(nanoseconds: i64) {
    unsafe { ctru_sys::svcSleepThread(nanoseconds) }
}

#[ctr_macros::hos]
pub fn signal_event(event: &Handle) -> CtrResult {
    let result = unsafe { ctru_sys::svcSignalEvent(event.get_raw()) };
    parse_result(result)
}

#[ctr_macros::hos]
pub fn exit_process() -> ! {
    unsafe {
        ctru_sys::svcExitProcess();
    }

    // Allow the empty loop to get the 'never' return type
    // We'll never reach this far because the above will break anyways
    #[allow(clippy::empty_loop)]
    loop {}
}

#[ctr_macros::hos]
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
        ctru_sys::svcCreateMemoryBlock(
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

#[ctr_macros::hos]
pub fn unmap_memory_block(memory_block_handle: &Handle, slice: &[u8]) -> CtrResult {
    let result = unsafe {
        ctru_sys::svcUnmapMemoryBlock(memory_block_handle.get_raw(), slice.as_ptr() as u32)
    };
    parse_result(result)
}

#[ctr_macros::hos]
pub fn wait_synchronization(handle: &Handle, wait_nanoseconds: i64) -> CtrResult {
    let result = unsafe { ctru_sys::svcWaitSynchronization(handle.get_raw(), wait_nanoseconds) };
    parse_result(result)
}

#[ctr_macros::hos]
pub fn get_process_list() -> CtrResult<Vec<u32>> {
    let mut process_ids: Vec<u32> = vec![0; 0x40];
    let mut process_count = 0;
    let result = unsafe {
        ctru_sys::svcGetProcessList(
            &mut process_count,
            process_ids.as_mut_ptr(),
            process_ids.len() as i32,
        )
    };

    parse_result(result)?;
    process_ids.truncate(process_count as usize);
    Ok(process_ids)
}

#[ctr_macros::hos]
pub fn open_process(process_id: u32) -> CtrResult<Handle> {
    let mut raw_handle = 0;
    let result = unsafe { ctru_sys::svcOpenProcess(&mut raw_handle, process_id) };

    parse_result(result)?;
    Ok(raw_handle.into())
}

#[ctr_macros::hos]
pub fn debug_active_process(process_id: u32) -> CtrResult<Handle> {
    let mut raw_handle = 0u32;
    let result = unsafe { ctru_sys::svcDebugActiveProcess(&mut raw_handle, process_id) };

    parse_result(result)?;
    Ok(raw_handle.into())
}

#[ctr_macros::hos]
pub fn read_process_memory(debug_process: &Handle, addr: u32, size: u32) -> CtrResult<Vec<u8>> {
    let mut buffer = vec![0; size as usize];
    let result = unsafe {
        ctru_sys::svcReadProcessMemory(
            buffer.as_mut_ptr() as *mut c_void,
            debug_process.get_raw(),
            addr,
            size,
        )
    };

    parse_result(result)?;
    Ok(buffer)
}

#[ctr_macros::hos]
pub fn write_process_memory(debug_process: &Handle, buffer: &[u8], addr: u32) -> CtrResult {
    let result = unsafe {
        ctru_sys::svcWriteProcessMemory(
            debug_process.get_raw(),
            buffer.as_ptr() as *const c_void,
            addr,
            buffer.len() as u32,
        )
    };

    parse_result(result)?;
    Ok(())
}

#[ctr_macros::hos]
pub fn continue_debug_event(debug_process: &Handle, flag: DebugFlag) -> CtrResult {
    let result = unsafe { ctru_sys::svcContinueDebugEvent(debug_process.get_raw(), flag.into()) };

    parse_result(result)?;
    Ok(())
}

// TODO: Implement for reals.  This is hacked together for now.
#[ctr_macros::hos]
pub fn get_process_debug_event(debug_process: &Handle) -> ResultCode {
    let mut info: [u8; 0x28] = [0; 0x28];
    unsafe {
        ctru_sys::svcGetProcessDebugEvent(
            core::mem::transmute::<*mut u8, *mut ctru_sys::DebugEventInfo>(info.as_mut_ptr()),
            debug_process.get_raw(),
        )
        .into()
    }
}

#[ctr_macros::hos]
pub fn get_process_info(process: &Handle, info_type: ProcessInfoType) -> CtrResult<i64> {
    let mut out = 0;
    let result =
        unsafe { ctru_sys::svcGetProcessInfo(&mut out, process.get_raw(), info_type as u32) };

    parse_result(result)?;

    Ok(out)
}

#[ctr_macros::hos]
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
        ctru_sys::svcQueryDebugProcessMemory(
            &mut mem_info,
            &mut page_info,
            debug_process.get_raw(),
            addr,
        )
    };

    parse_result(result)?;
    Ok(MemQueryResponse {
        mem_info: MemInfo {
            base_addr: mem_info.base_addr,
            size: mem_info.size as usize,
            perm: mem_info.perm,
            state: mem_info.state,
        },
        page_info: PageInfo {
            flags: page_info.flags,
        },
    })
}

#[ctr_macros::hos]
pub fn query_process_memory(process: &Handle, addr: u32) -> CtrResult<MemQueryResponse> {
    // Result svcQueryProcessMemory(MemInfo* info, PageInfo* out, Handle process, u32 addr);
    let mut mem_info = ctru_sys::MemInfo {
        base_addr: 0,
        perm: 0,
        size: 0,
        state: 0,
    };
    let mut page_info = ctru_sys::PageInfo { flags: 0 };
    let result = unsafe {
        ctru_sys::svcQueryProcessMemory(&mut mem_info, &mut page_info, process.get_raw(), addr)
    };

    parse_result(result)?;
    Ok(MemQueryResponse {
        mem_info: MemInfo {
            base_addr: mem_info.base_addr,
            size: mem_info.size as usize,
            perm: mem_info.perm,
            state: mem_info.state,
        },
        page_info: PageInfo {
            flags: page_info.flags,
        },
    })
}

#[inline(never)]
#[ctr_macros::hos]
pub fn exit_thread() -> ! {
    unsafe {
        ctru_sys::svcExitThread();
    }

    // Allow the empty loop to get the 'never' return type
    // We'll never reach this far because the above will exit anyways
    #[allow(clippy::empty_loop)]
    loop {}
}

#[inline(never)]
#[ctr_macros::hos]
pub fn get_thread_priority(handle: &Handle) -> CtrResult<i32> {
    let mut priority = 0;
    let result = unsafe { ctru_sys::svcGetThreadPriority(&mut priority, handle.get_raw()) };
    parse_result(result)?;
    Ok(priority)
}

/// This is primarily here to let thread code compile outside horizon.
///
/// # Safety
/// Providing invalid values will cause a crash.
/// Using a stack that overlaps with memory actively used in other processes is undefined behavior.
#[inline(never)]
#[ctr_macros::hos]
pub(crate) unsafe fn create_thread(
    func: unsafe extern "C" fn(*mut c_void),
    arg: u32,
    stack: *mut u32,
    priority: i32,
    processor_id: i32,
) -> CtrResult<Handle> {
    let mut handle = 0u32;
    let result = unsafe {
        ctru_sys::svcCreateThread(&mut handle, Some(func), arg, stack, priority, processor_id)
    };
    parse_result(result)?;
    Ok(handle.into())
}

#[inline(never)]
#[ctr_macros::hos]
pub fn get_process_id(process: &Handle) -> CtrResult<u32> {
    let mut out = 0;
    let result = unsafe { ctru_sys::svcGetProcessId(&mut out, process.get_raw()) };
    parse_result(result)?;
    Ok(out)
}

#[inline(never)]
#[ctr_macros::hos]
pub fn create_port(name: Option<&CStr>, max_sessions: i32) -> CtrResult<Port> {
    let mut server = 0;
    let mut client = 0;

    let name_ptr = match name {
        Some(name) => name.as_ptr(),
        None => core::ptr::null(),
    };

    let result =
        unsafe { ctru_sys::svcCreatePort(&mut server, &mut client, name_ptr, max_sessions) };
    parse_result(result)?;

    Ok(Port {
        server: server.into(),
        client: client.into(),
    })
}
