use crate::{
    ipc::{Command, CurrentProcessId, Handles, StaticBuffer},
    ndm::{enter_exclusive_state, NdmExclusiveState},
    res::CtrResult,
    srv::get_service_handle_direct,
    svc,
    svc::EventResetType,
    Handle,
};
use no_std_io::{EndianRead, EndianWrite, Reader};
use std::{
    mem::ManuallyDrop,
    sync::atomic::{AtomicU32, Ordering},
};

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

#[derive(Clone, Copy, Debug, PartialEq, Default, EndianRead, EndianWrite)]
#[repr(C)]
pub struct SsidInfo {
    pub length: u32,
    pub name: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Default, EndianRead, EndianWrite)]
#[repr(C)]
pub struct ApInfo {
    pub ssid: SsidInfo,
    pub bssid: [u8; 6],
    pub unknown: [u8; 10],
}

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

#[derive(Clone, Copy, Debug, PartialEq, EndianRead, EndianWrite)]
#[repr(C)]
pub struct ConnectingHotspotSubnet {
    pub data: [u8; 76],
}

impl Default for ConnectingHotspotSubnet {
    fn default() -> Self {
        Self { data: [0; 76] }
    }
}

#[derive(EndianRead, EndianWrite)]
struct GetCurrentApInfoIn {
    size: u32,
    process_id: CurrentProcessId,
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn acu_get_current_ap_info() -> CtrResult<ApInfo> {
    let mut ap_info_bytes: [u8; 0x34] = [0; 0x34];
    let input = GetCurrentApInfoIn {
        size: 0x34,
        process_id: CurrentProcessId::new(),
    };
    let ap_info = StaticBuffer::new_mut(&mut ap_info_bytes, 0);
    Command::new_with_static_out(0xe0042, input, ap_info).send(get_raw_handle())?;
    let ap_info: ApInfo = ap_info_bytes.read_le(0).unwrap();
    Ok(ap_info)
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn acu_get_wifi_status() -> CtrResult<u32> {
    Command::new(0xD0000, ()).send(get_raw_handle())
}

#[derive(EndianRead, EndianWrite)]
struct CloseAsyncIn {
    process_id: CurrentProcessId,
    handle: Handles,
}

pub fn close_async(event_handle: &Handle) -> CtrResult {
    let input = CloseAsyncIn {
        process_id: CurrentProcessId::new(),
        handle: unsafe { Handles::new(vec![event_handle.get_raw()]) },
    };
    Command::new(0x80004, input).send(get_raw_handle())
}

#[derive(EndianRead, EndianWrite)]
struct GetNzoneApSsidIn {
    size: u32,
    process_id: CurrentProcessId,
}

pub fn acu_get_nzone_ap_ssid() -> CtrResult<SsidInfo> {
    let mut ssid_info_bytes: [u8; 0x24] = [0; 0x24];
    let input = GetNzoneApSsidIn {
        size: 0x24,
        process_id: CurrentProcessId::new(),
    };
    let ssid_info = StaticBuffer::new_mut(&mut ssid_info_bytes, 0);
    Command::new_with_static_out(0x110042, input, ssid_info).send(get_raw_handle())?;
    let ssid_info: SsidInfo = ssid_info_bytes.read_le(0).unwrap();
    Ok(ssid_info)
}

#[derive(EndianRead, EndianWrite)]
struct GetConnectingHotspotSubnetIn {
    size: u32,
    process_id: CurrentProcessId,
}

pub fn acu_get_connecting_hotspot_subnet() -> CtrResult<ConnectingHotspotSubnet> {
    let mut connecting_hotspot_subnet_bytes: [u8; 0x4c] = [0; 0x4c];
    let input = GetConnectingHotspotSubnetIn {
        size: 0x4c,
        process_id: CurrentProcessId::new(),
    };
    let connecting_hotspot_subnet = StaticBuffer::new_mut(&mut connecting_hotspot_subnet_bytes, 0);
    Command::new_with_static_out(0x130042, input, connecting_hotspot_subnet)
        .send(get_raw_handle())?;
    let connecting_hotspot_subnet: ConnectingHotspotSubnet =
        connecting_hotspot_subnet_bytes.read_le(0).unwrap();
    Ok(connecting_hotspot_subnet)
}

#[derive(EndianRead, EndianWrite)]
struct SetProperyIn<T: EndianRead + EndianWrite> {
    properties: T,
    ac_controller_in: StaticBuffer,
}

#[derive(EndianRead, EndianWrite)]
struct RequestEulaVersionIn {
    version_1: u32,
    version_2: u32,
}

#[derive(EndianRead, EndianWrite)]
struct ConnectAcIn {
    process_id: CurrentProcessId,
    handle: Handles,
    ac_controller: StaticBuffer,
}

const AC_CONFIG_SIZE: usize = 0x200;
pub struct AcController([u8; AC_CONFIG_SIZE]);

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
impl AcController {
    pub fn new() -> CtrResult<Self> {
        let mut inner_config: [u8; AC_CONFIG_SIZE] = [0; AC_CONFIG_SIZE];
        Command::new_with_static_out(0x10000, (), StaticBuffer::new_mut(&mut inner_config, 0))
            .send(get_raw_handle())?;
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

    fn set_property<T: EndianRead + EndianWrite>(
        &mut self,
        command_id: u16,
        properties: T,
        property_count: u16,
    ) -> CtrResult {
        let input = SetProperyIn {
            properties,
            ac_controller_in: StaticBuffer::new(&self.0, 0),
        };
        let static_out = StaticBuffer::new(&self.0, 0);
        Command::new_from_parts_with_static_out(command_id, property_count, 2, input, static_out)
            .send(get_raw_handle())
    }

    pub fn set_area(&mut self, area: u8) -> CtrResult {
        self.set_property(0x25u16, area as u32, 1)
    }

    pub fn set_infra_priority(&mut self, infra_priority: u8) -> CtrResult {
        self.set_property(0x26u16, infra_priority as u32, 1)
    }

    pub fn set_power_save_mode(&mut self, power_save_mode: u8) -> CtrResult {
        self.set_property(0x28u16, power_save_mode as u32, 1)
    }

    pub fn set_request_eula_version(&mut self, version_1: u8, version_2: u8) -> CtrResult {
        let input = RequestEulaVersionIn {
            version_1: version_1 as u32,
            version_2: version_2 as u32,
        };
        self.set_property(0x2Du16, input, 2)
    }

    pub fn add_deny_ap_type(&mut self, ap_type: u32) -> CtrResult {
        self.set_property(0x24u16, ap_type as u32, 1)
    }

    pub fn get_infra_priority(&self) -> CtrResult<u8> {
        let out: u32 =
            Command::new(0x270002, StaticBuffer::new(&self.0, 1)).send(get_raw_handle())?;
        Ok(out as u8)
    }

    pub fn connect_async(&self, connection_handle: &Handle) -> CtrResult {
        let input = ConnectAcIn {
            process_id: CurrentProcessId::new(),
            handle: unsafe { Handles::new(vec![connection_handle.get_raw()]) },
            ac_controller: StaticBuffer::new(&self.0, 1),
        };
        Command::new(0x40006, input).send(get_raw_handle())
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
