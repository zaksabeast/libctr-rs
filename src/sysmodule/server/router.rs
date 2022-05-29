use super::Service;
use crate::{
    ipc::WrittenCommand,
    result::{CtrResult, ResultCode},
};
use no_std_io::{EndianRead, EndianWrite};

#[derive(Debug, EndianRead, EndianWrite)]
pub struct CtrSuccessResponse<T: EndianRead + EndianWrite> {
    result_code: ResultCode,
    data: T,
}

impl<T: EndianRead + EndianWrite> CtrSuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            result_code: ResultCode::success(),
            data,
        }
    }
}

pub trait ServiceRoute<S: Service, const COMMAND_ID: u16> {
    fn handle_request(&mut self, session_index: usize) -> CtrResult<WrittenCommand>;
}

pub trait ServiceRouter {
    fn handle_request(
        &mut self,
        service_id: usize,
        session_index: usize,
    ) -> CtrResult<WrittenCommand>;

    fn accept_session(&mut self, session_index: usize);
    fn close_session(&mut self, session_index: usize);
}
