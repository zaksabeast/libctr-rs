use crate::{
    ipc::ThreadCommandBuilder,
    res::{CtrResult, ResultCode},
    srv::get_service_handle_direct,
    svc,
};
use core::{
    mem::ManuallyDrop,
    sync::atomic::{AtomicU32, Ordering},
};
use safe_transmute::TriviallyTransmutable;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct RomId([u8; 16]);

// This is safe because all fields in the struct can function with any value.
unsafe impl TriviallyTransmutable for RomId {}

impl RomId {
    pub fn new(rom_id: [u8; 16]) -> Self {
        Self(rom_id)
    }

    pub fn get_inner(&self) -> &[u8] {
        &self.0
    }
}

static PS_HANDLE: AtomicU32 = AtomicU32::new(0);

fn get_raw_handle() -> u32 {
    PS_HANDLE.load(Ordering::Relaxed)
}

/// Initializes the AC service. Required to use AC features.
fn init() -> CtrResult<ResultCode> {
    let handle = get_service_handle_direct("ps:ps")?;

    let dropped_handle = ManuallyDrop::new(handle);
    let raw_handle = unsafe { dropped_handle.get_raw() };
    PS_HANDLE.store(raw_handle, Ordering::Relaxed);

    Ok(0)
}

fn exit() -> CtrResult<ResultCode> {
    let result = svc::close_handle(get_raw_handle());

    if result.is_ok() {
        PS_HANDLE.store(0, Ordering::Relaxed);
    }

    result
}

fn get_rom_id_impl(process_id: u32) -> CtrResult<RomId> {
    let mut command = ThreadCommandBuilder::new(0x6u16);
    command.push(process_id);

    let mut parser = command
        .build()
        .send_sync_request_with_raw_handle(get_raw_handle())?;
    parser.pop_result()?;
    parser.pop_struct::<RomId>()
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn get_rom_id(process_id: u32) -> CtrResult<RomId> {
    init()?;
    let result = get_rom_id_impl(process_id);
    exit()?;

    result
}
