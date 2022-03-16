use crate::{fs, ipc::ThreadCommandBuilder, res::CtrResult, srv::get_service_handle_direct, svc};
use core::{
    mem::ManuallyDrop,
    sync::atomic::{AtomicU32, Ordering},
};

static PM_DBG_HANDLE: AtomicU32 = AtomicU32::new(0);

fn get_raw_handle() -> u32 {
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
    let result = svc::close_handle(get_raw_handle());

    if result.is_ok() {
        PM_DBG_HANDLE.store(0, Ordering::Relaxed);
    }

    result
}

pub struct RunningAppInfo {
    pub program_info: fs::ProgramInfo,
    pub pid: u32,
    pub launch_flags: u32,
}

fn get_current_app_info_impl() -> CtrResult<RunningAppInfo> {
    let mut parser = ThreadCommandBuilder::new(0x100u16)
        .build()
        .send_sync_request_with_raw_handle(get_raw_handle())?;

    parser.pop_result()?;

    // Pop these individually instead of a struct since MediaType is not
    let program_id = parser.pop_u64();
    let media_type = parser.pop();
    let padding = parser.pop_struct::<[u8; 7]>()?;
    let pid = parser.pop();
    let launch_flags = parser.pop();

    Ok(RunningAppInfo {
        program_info: fs::ProgramInfo {
            program_id,
            padding,
            media_type: media_type.into(),
        },
        pid,
        launch_flags,
    })
}

/// This is a luma only command
#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn get_current_app_info() -> CtrResult<RunningAppInfo> {
    init()?;
    let result = get_current_app_info_impl();
    exit()?;

    result
}
