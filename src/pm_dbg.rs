use crate::{fs, ipc::Command, res::CtrResult, srv::get_service_handle_direct, svc};
use core::{
    mem::ManuallyDrop,
    sync::atomic::{AtomicU32, Ordering},
};
use no_std_io::{EndianRead, EndianWrite};

static PM_DBG_HANDLE: AtomicU32 = AtomicU32::new(0);

fn get_handle() -> u32 {
    PM_DBG_HANDLE.load(Ordering::Relaxed)
}

/// Initializes the pm:dbg service. Required to use pm:dbg features.
fn init() -> CtrResult {
    let handle = get_service_handle_direct("pm:dbg")?;

    let dropped_handle = ManuallyDrop::new(handle);
    let raw_handle = unsafe { dropped_handle.get_raw() };
    PM_DBG_HANDLE.store(raw_handle, Ordering::Relaxed);

    Ok(())
}

fn exit() -> CtrResult {
    let result = svc::close_handle(get_handle());

    if result.is_ok() {
        PM_DBG_HANDLE.store(0, Ordering::Relaxed);
    }

    result
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct RunningAppInfo {
    pub program_info: fs::ProgramInfo,
    pub pid: u32,
    pub launch_flags: u32,
}

fn get_current_app_info_impl() -> CtrResult<RunningAppInfo> {
    Command::new(0x1000000, ()).send(get_handle())
}

/// This is a luma only command
pub fn get_current_app_info() -> CtrResult<RunningAppInfo> {
    init()?;
    let result = get_current_app_info_impl();
    exit()?;

    result
}
