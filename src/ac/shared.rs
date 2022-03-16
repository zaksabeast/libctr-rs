use crate::{
    ipc::ThreadCommandBuilder,
    ndm::{enter_exclusive_state, NdmExclusiveState},
    res::CtrResult,
    srv::get_service_handle_direct,
    svc,
    svc::EventResetType,
    utils::convert::try_usize_into_u32,
    Handle,
};
use alloc::{format, string::String};
use core::{
    mem::{transmute, ManuallyDrop},
    sync::atomic::{AtomicU32, Ordering},
};
use safe_transmute::{transmute_one_to_bytes_mut, TriviallyTransmutable};

static AC_HANDLE: AtomicU32 = AtomicU32::new(0);

fn get_raw_handle() -> u32 {
    AC_HANDLE.load(Ordering::Relaxed)
}

/// Initializes the AC service. Required to use AC features.
pub fn init() -> CtrResult {
    let handle =
        get_service_handle_direct("ac:i").or_else(|_| get_service_handle_direct("ac:u"))?;

    let dropped_handle = ManuallyDrop::new(handle);
    let raw_handle = unsafe { dropped_handle.get_raw() };
    AC_HANDLE.store(raw_handle, Ordering::Relaxed);

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C)]
pub struct SsidInfo {
    pub length: u32,
    pub name: [u8; 32],
}

// This is safe because all fields in the struct can function with any value.
unsafe impl TriviallyTransmutable for SsidInfo {}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C)]
pub struct ApInfo {
    pub ssid: SsidInfo,
    pub bssid: [u8; 6],
    pub unknown: [u8; 10],
}

// This is safe because all fields in the struct can function with any value.
unsafe impl TriviallyTransmutable for ApInfo {}

impl ApInfo {
    pub fn get_formatted_bssid(&self) -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.bssid[0],
            self.bssid[1],
            self.bssid[2],
            self.bssid[3],
            self.bssid[4],
            self.bssid[5]
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct ConnectingHotspotSubnet {
    pub data: [u8; 76],
}

// This is safe because all fields in the struct can function with any value.
unsafe impl TriviallyTransmutable for ConnectingHotspotSubnet {}

impl Default for ConnectingHotspotSubnet {
    fn default() -> Self {
        Self { data: [0; 76] }
    }
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn acu_get_current_ap_info() -> CtrResult<ApInfo> {
    let mut ap_info: ApInfo = Default::default();

    let size = core::mem::size_of::<ApInfo>();
    let size_u32 = try_usize_into_u32(size)?;

    static_assertions::assert_eq_size_val!(ap_info, [0u8; 0x34]);

    let mut command = ThreadCommandBuilder::new(0xEu16);
    command.push(size_u32);
    command.push_curent_process_id();
    command.push_output_static_buffer(transmute_one_to_bytes_mut(&mut ap_info), 0);

    let mut parser = command
        .build()
        .send_sync_request_with_raw_handle(get_raw_handle())?;
    parser.pop_result()?;

    Ok(ap_info)
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn acu_get_wifi_status() -> CtrResult<u32> {
    let command = ThreadCommandBuilder::new(0xDu16);
    let mut parser = command
        .build()
        .send_sync_request_with_raw_handle(get_raw_handle())?;
    parser.pop_result()?;

    Ok(parser.pop())
}

pub fn close_async(event_handle: &Handle) -> CtrResult {
    let mut command = ThreadCommandBuilder::new(0x8u16);
    command.push_curent_process_id();

    // This is safe since we're not duplicating the handle outside of an svc
    unsafe { command.push_raw_handle(event_handle.get_raw()) };

    let mut parser = command
        .build()
        .send_sync_request_with_raw_handle(get_raw_handle())?;
    parser.pop_result()?;

    Ok(())
}

pub fn acu_get_nzone_ap_ssid() -> CtrResult<SsidInfo> {
    let ssid_info = Default::default();
    let ssid_info_buffer = &mut [ssid_info];

    let size = core::mem::size_of::<ApInfo>();
    let size_u32 = try_usize_into_u32(size)?;

    static_assertions::assert_eq_size_val!(ssid_info, [0u8; 0x24]);

    let mut command = ThreadCommandBuilder::new(0x11u16);
    command.push(size_u32);
    command.push_curent_process_id();
    command.push_output_static_buffer(ssid_info_buffer, 0);

    let mut parser = command
        .build()
        .send_sync_request_with_raw_handle(get_raw_handle())?;
    parser.pop_result()?;

    Ok(ssid_info)
}

pub fn acu_get_connecting_hotspot_subnet() -> CtrResult<ConnectingHotspotSubnet> {
    let connecting_hotspot_subnet = Default::default();
    let connecting_hotspot_subnet_buffer = &mut [connecting_hotspot_subnet];

    let size = core::mem::size_of::<ApInfo>();
    let size_u32 = try_usize_into_u32(size)?;

    static_assertions::assert_eq_size_val!(connecting_hotspot_subnet, [0u8; 0x4c]);

    let mut command = ThreadCommandBuilder::new(0x13u16);
    command.push(size_u32);
    command.push_curent_process_id();
    command.push_output_static_buffer(connecting_hotspot_subnet_buffer, 0);

    let mut parser = command
        .build()
        .send_sync_request_with_raw_handle(get_raw_handle())?;
    parser.pop_result()?;

    Ok(connecting_hotspot_subnet)
}

const AC_CONFIG_SIZE: usize = 0x200;
pub struct AcController([u8; AC_CONFIG_SIZE]);

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
impl AcController {
    pub fn new() -> CtrResult<Self> {
        let mut inner_config: [u8; AC_CONFIG_SIZE] = [0; AC_CONFIG_SIZE];

        let mut command = ThreadCommandBuilder::new(0x1u16);
        command.push_output_static_buffer(&mut inner_config, 0);

        let mut parser = command
            .build()
            .send_sync_request_with_raw_handle(get_raw_handle())?;

        parser.pop_result()?;
        Ok(Self(inner_config))
    }

    pub fn check_if_connected() -> CtrResult<bool> {
        let status = acu_get_wifi_status()?;
        Ok(status != 0)
    }

    pub fn quick_connect() -> CtrResult {
        let mut ac_controller = AcController::new()?;
        ac_controller.set_area(0)?;
        ac_controller.set_infra_priority(1)?;
        ac_controller.set_power_save_mode(1)?;

        let connection_event = svc::create_event(EventResetType::OneShot)?;

        ac_controller.connect(&connection_event)?;

        svc::wait_synchronization(&connection_event, -1)?;

        Ok(())
    }

    pub fn disconnect() -> CtrResult {
        let disconnect_event = svc::create_event(EventResetType::OneShot)?;
        close_async(&disconnect_event)?;
        svc::wait_synchronization(&disconnect_event, -1)?;

        Ok(())
    }

    fn set_property<T: Into<u32> + Copy>(&mut self, command_id: u16, values: &[T]) -> CtrResult {
        let mut command = ThreadCommandBuilder::new(command_id);

        for value in values {
            command.push(*value);
        }

        // This is safe because the command will read the config from here
        // and output the mutable result after the command has run.
        // The immutable reference is only used at the exact same time
        // and place as the mutable reference, and the entity using
        // those references uses them in a safe order.
        // Although it looks like this _might_ be avoidable at the cost of memory.
        // It looks like it could be possible for the input and output to be
        // two different configs.
        unsafe {
            let unsafe_reference = transmute::<&mut AcController, &AcController>(self);
            command.push_static_buffer(&unsafe_reference.0, 0);
        }

        command.push_output_static_buffer(&mut self.0, 0);

        let mut parser = command
            .build()
            .send_sync_request_with_raw_handle(get_raw_handle())?;

        parser.pop_result()?;
        Ok(())
    }

    pub fn set_area(&mut self, area: u8) -> CtrResult {
        self.set_property(0x25u16, &[area])
    }

    pub fn set_infra_priority(&mut self, infra_priority: u8) -> CtrResult {
        self.set_property(0x26u16, &[infra_priority])
    }

    pub fn set_power_save_mode(&mut self, power_save_mode: u8) -> CtrResult {
        self.set_property(0x28u16, &[power_save_mode])
    }

    pub fn set_request_eula_version(&mut self, version_1: u8, version_2: u8) -> CtrResult {
        self.set_property(0x2Du16, &[version_1, version_2])
    }

    pub fn add_deny_ap_type(&mut self, ap_type: u32) -> CtrResult {
        self.set_property(0x24u16, &[ap_type])
    }

    pub fn get_infra_priority(&self) -> CtrResult<u8> {
        let mut command = ThreadCommandBuilder::new(0x27u16);
        command.push_static_buffer(&self.0, 1);

        let mut parser = command
            .build()
            .send_sync_request_with_raw_handle(get_raw_handle())?;
        parser.pop_result()?;

        Ok(parser.pop() as u8)
    }

    pub fn connect_async(&self, connection_handle: &Handle) -> CtrResult {
        let mut command = ThreadCommandBuilder::new(0x4u16);
        command.push_curent_process_id();

        // This is safe since we're not duplicating the handle outside of an svc
        unsafe { command.push_raw_handle(connection_handle.get_raw()) };

        command.push_static_buffer(&self.0, 1);

        let mut parser = command
            .build()
            .send_sync_request_with_raw_handle(get_raw_handle())?;
        parser.pop_result()?;

        Ok(())
    }

    pub fn connect(&mut self, connection_handle: &Handle) -> CtrResult {
        let infra_priority = self.get_infra_priority()?;

        if infra_priority == 0 {
            enter_exclusive_state(NdmExclusiveState::Infrastructure)?;
            self.set_request_eula_version(0, 0)?;
        } else {
            self.add_deny_ap_type(0x40)?;
        }

        self.connect_async(connection_handle)?;

        Ok(())
    }
}
