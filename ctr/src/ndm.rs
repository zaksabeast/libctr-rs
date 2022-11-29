use crate::{
    ipc::{Command, CurrentProcessId},
    res::CtrResult,
    service_session::{create_session_manager, session},
    srv::get_service_handle_direct,
};
use no_std_io::{EndianRead, EndianWrite};
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

create_session_manager!(get_service_handle_direct("ndm:u")?);

#[derive(EndianRead, EndianWrite)]
struct EnterExclusiveStateIn {
    state: u32,
    current_process_id: CurrentProcessId,
}

fn enter_exclusive_state_impl(state: NdmExclusiveState) -> CtrResult {
    let input = EnterExclusiveStateIn {
        state: state as u32,
        current_process_id: CurrentProcessId::new(),
    };
    Command::new(0x10042, input).send(get_handle())
}

pub fn enter_exclusive_state(state: NdmExclusiveState) -> CtrResult {
    session!(ndm);
    enter_exclusive_state_impl(state)
}
