use crate::{
    ipc::Command,
    res::CtrResult,
    service_session::{create_session_manager, session},
    srv::get_service_handle_direct,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RomId([u8; 16]);

impl RomId {
    pub fn new(rom_id: [u8; 16]) -> Self {
        Self(rom_id)
    }

    pub fn get_inner(&self) -> &[u8] {
        &self.0
    }
}

create_session_manager!(get_service_handle_direct("ps:ps")?);

fn get_rom_id_impl(process_id: u32) -> CtrResult<RomId> {
    let rom_id = Command::new(0x60040, process_id).send::<[u8; 16]>(get_handle())?;
    Ok(RomId(rom_id))
}

pub fn get_rom_id(process_id: u32) -> CtrResult<RomId> {
    session!(ps);
    get_rom_id_impl(process_id)
}
