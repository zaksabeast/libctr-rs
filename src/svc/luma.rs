//   This paricular file is licensed under the following terms:

//   This software is provided 'as-is', without any express or implied warranty. In no event will the authors be held liable
//   for any damages arising from the use of this software.
//
//   Permission is granted to anyone to use this software for any purpose, including commercial applications, and to alter it
//   and redistribute it freely, subject to the following restrictions:
//
//    The origin of this software must not be misrepresented; you must not claim that you wrote the original software.
//    If you use this software in a product, an acknowledgment in the product documentation would be appreciated but is not required.
//
//    Altered source versions must be plainly marked as such, and must not be misrepresented as being the original software.
//    This notice may not be removed or altered from any source distribution.

// These were taken from
// - https://github.com/LumaTeam/Luma3DS/blob/21db0d45bdffc933491be24abb933929fc91ca96/sysmodules/rosalina/source/csvc.s
// - https://github.com/LumaTeam/Luma3DS/blob/44c171180567c672cb82d8f70281157845b0fcbf/sysmodules/rosalina/source/menus/cheats.c

use super::{continue_debug_event, get_process_debug_event, DebugFlag};
use crate::{
    res::{parse_result, CtrResult, GenericResultCode},
    Handle,
};
use cstr_core::CStr;

/// Consumes all debug events.
#[ctr_macros::hos]
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

/// Luma only.
#[inline(never)]
#[ctr_macros::hos]
pub fn control_service(service_op: u32, service_name: &CStr) -> CtrResult<Handle> {
    let mut result_code: i32;
    let mut handle: u32 = 0;

    unsafe {
        core::arch::asm!("svc 0xB0", in("r0") service_op, in("r1") &mut handle, in("r2") service_name.as_ptr(), lateout("r0") result_code);
    }

    parse_result(result_code)?;
    Ok(handle.into())
}

/// Luma only.
/// Copies a handle from one process to another.
#[inline(never)]
#[ctr_macros::hos]
pub fn copy_handle(out_process: &Handle, input: &Handle, in_process: &Handle) -> CtrResult<Handle> {
    let mut result: i32;
    let mut out_handle = 0u32;
    unsafe {
        core::arch::asm!(
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

/// Luma only.
/// Converts a virtual address into a physical address.
///
/// Returns a null pointer if the pointer is invalid or if the caller
/// does not have permissions to the pointer.
#[inline(never)]
#[ctr_macros::hos]
pub fn convert_va_to_pa(virtual_addr: *mut u8, write_check: bool) -> *mut u8 {
    let mut physical_addr: *mut u8;

    unsafe {
        core::arch::asm!("svc 0x90", in("r0") virtual_addr, in("r1") write_check as u32, lateout("r0") physical_addr)
    };

    physical_addr
}

/// Luma only.
/// Gets the uncached address of a physical address.
/// Returns an error if the pointer is null.
#[inline(never)]
#[ctr_macros::hos]
pub fn convert_pa_to_uncached_pa(physical_addr: *mut u8) -> CtrResult<*mut u8> {
    if physical_addr.is_null() {
        return Err(GenericResultCode::InvalidPointer.into());
    }

    // Thanks to Luma3ds for this code
    let uncached_physical_addr = ((physical_addr as u32) | (1 << 31)) as *mut u8;

    Ok(uncached_physical_addr)
}

/// Luma only.
///
/// # Safety
/// This maps memory, so standard memory/pointer rules apply.
/// However this also maps memory between processes.  Most use of this is inherently undefined behavior.
#[inline(never)]
#[ctr_macros::hos]
pub unsafe fn map_memory_ex(process: &Handle, dst: *mut u8, src: u32, size: usize) -> CtrResult {
    let mut result_code: i32;
    core::arch::asm!("svc 0xa0", in("r0") process.get_raw(), in("r1") dst as u32, in("r2") src as u32, in("r3") size as u32, lateout("r0") result_code);
    parse_result(result_code)
}

/// Luma only.
///
/// # Safety
/// This maps memory, so standard memory/pointer rules apply.
/// However this also maps memory between processes.  Most use of this is inherently undefined behavior.
#[inline(never)]
#[ctr_macros::hos]
pub unsafe fn unmap_memory_ex(process: &Handle, dst: *mut u8, size: usize) -> CtrResult {
    let mut result_code: i32;
    core::arch::asm!("svc 0xa1", in("r0") process.get_raw(), in("r1") dst as u32, in("r2") size as u32, lateout("r0") result_code);
    parse_result(result_code)
}
