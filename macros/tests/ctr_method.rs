// This file is effectively just making sure the macros compile,
// and serves as an example.

use ctr::{
    ipc::WrittenCommand,
    result::CtrResult,
    sysmodule::server::{Service, ServiceRouter},
};
use ctr_macros::{ctr_method, match_ctr_route};
use no_std_io::{EndianRead, EndianWrite};
use num_enum::{FromPrimitive, IntoPrimitive};

// ----------------------------------------
// First service
// ----------------------------------------

#[derive(FromPrimitive, IntoPrimitive)]
#[repr(u16)]
enum MathService {
    #[num_enum(default)]
    Invalid = 0x0,
    AddNums = 0x1,
}

impl Service for MathService {
    const ID: usize = 0;
    const NAME: &'static str = "math";
    const MAX_SESSION_COUNT: i32 = 1;
}

#[derive(EndianRead, EndianWrite)]
struct AddNumsIn {
    first: u32,
    second: u32,
}

#[ctr_method(cmd = "MathService::AddNums", normal = 0x2, translate = 0x0)]
// Use no_std_io to safely get inputs and return outputs
fn add_nums(_server: &mut Sysmodule, _session_index: usize, input: AddNumsIn) -> CtrResult<u32> {
    Ok(input.first + input.second)
}

// ----------------------------------------
// Second service
// ----------------------------------------

#[derive(FromPrimitive, IntoPrimitive)]
#[repr(u16)]
enum GetSetService {
    #[num_enum(default)]
    Invalid = 0x0,
    GetData = 0x1,
    SetData = 0x2,
}

impl Service for GetSetService {
    const ID: usize = 1;
    const NAME: &'static str = "getset";
    const MAX_SESSION_COUNT: i32 = 1;
}

#[ctr_method(cmd = "GetSetService::GetData", normal = 0x2, translate = 0x0)]
// No input required
fn get_data(server: &mut Sysmodule, _session_index: usize) -> CtrResult<u32> {
    Ok(server.data)
}

#[ctr_method(cmd = "GetSetService::SetData", normal = 0x1, translate = 0x0)]
// Get a mutable reference to the server for session or global context
// No output required.  At least one normal_out is needed for the result code
fn set_data(server: &mut Sysmodule, _session_index: usize, some_data: u32) -> CtrResult {
    server.data = some_data;
    Ok(())
}

// ----------------------------------------
// Fake sysmodule
// ----------------------------------------

struct Sysmodule {
    data: u32,
}

impl ServiceRouter for Sysmodule {
    fn handle_request(
        &mut self,
        service_id: usize,
        session_index: usize,
    ) -> CtrResult<WrittenCommand> {
        // Session to service routing is handled automatically
        match_ctr_route!(
            Sysmodule,
            service_id,
            session_index,
            MathService::AddNums,
            GetSetService::GetData,
            GetSetService::SetData,
        )
    }

    fn accept_session(&mut self, _session_index: usize) {}

    fn close_session(&mut self, _session_index: usize) {}
}
