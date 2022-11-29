use crate::{
    fs,
    ipc::Command,
    res::CtrResult,
    service_session::{create_session_manager, session},
    srv::get_service_handle_direct,
};
use no_std_io::{EndianRead, EndianWrite};

create_session_manager!(get_service_handle_direct("pm:dbg")?);

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
    session!(pm_dbg);
    get_current_app_info_impl()
}
