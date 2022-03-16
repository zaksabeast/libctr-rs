use crate::{ipc::ThreadCommandBuilder, res::CtrResult, srv::get_service_handle_direct, svc};
use core::{
    mem::ManuallyDrop,
    sync::atomic::{AtomicU32, Ordering},
};
use num_enum::IntoPrimitive;

#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum NdmExclusiveState {
    None = 0,
    Infrastructure = 1,
    LocalCommunications = 2,
    Streetpass = 3,
    StreetpassData = 4,
}

static NDM_HANDLE: AtomicU32 = AtomicU32::new(0);

fn get_handle() -> u32 {
    NDM_HANDLE.load(Ordering::Relaxed)
}

/// Initializes the NDM service. Required to use NDM features.
pub fn init() -> CtrResult {
    let handle = get_service_handle_direct("ndm:u")?;

    let dropped_handle = ManuallyDrop::new(handle);
    let raw_handle = unsafe { dropped_handle.get_raw() };
    NDM_HANDLE.store(raw_handle, Ordering::Relaxed);

    Ok(())
}

pub fn exit() -> CtrResult {
    let result = svc::close_handle(get_handle());

    if result.is_ok() {
        NDM_HANDLE.store(0, Ordering::Relaxed)
    }

    result
}

fn enter_exclusive_state_impl(state: NdmExclusiveState) -> CtrResult {
    let mut command = ThreadCommandBuilder::new(0x1u16);
    command.push(state as u32);
    command.push_curent_process_id();

    let mut parser = command
        .build()
        .send_sync_request_with_raw_handle(get_handle())?;

    parser.pop_result()?;
    Ok(())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn enter_exclusive_state(state: NdmExclusiveState) -> CtrResult {
    init()?;
    let result = enter_exclusive_state_impl(state);
    exit()?;

    result
}
