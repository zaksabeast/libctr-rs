use super::Process;
use crate::{
    res::{CtrResult, ResultCode},
    svc, Handle,
};
use alloc::vec::Vec;
use core::{convert::TryFrom, mem};
use no_std_io::{EndianRead, Reader};

pub struct DebugProcess {
    handle: Handle,
    process: Process,
}

impl DebugProcess {
    pub fn new(title_id: u64) -> CtrResult<Self> {
        let process = Process::new_from_title_id(title_id)?;
        Self::new_from_process(process)
    }

    pub fn new_from_process(process: Process) -> CtrResult<Self> {
        let handle = svc::debug_active_process(process.get_process_id())?;
        Ok(Self { handle, process })
    }

    pub fn get_process(&self) -> &Process {
        &self.process
    }

    pub fn read_bytes(&self, addr: u32, size: u32) -> CtrResult<Vec<u8>> {
        svc::read_process_memory(&self.handle, addr, size)
    }

    pub fn read<T: EndianRead>(&self, addr: u32) -> CtrResult<T> {
        let result = self
            .read_bytes(addr, mem::size_of::<T>() as u32)?
            .read_le(0)?;
        Ok(result)
    }

    pub fn write_bytes(&self, addr: u32, buffer: &[u8]) -> CtrResult {
        svc::write_process_memory(&self.handle, buffer, addr)
    }

    pub fn get_debug_event(&self) -> CtrResult {
        svc::get_process_debug_event(&self.handle).into_result()
    }

    pub fn continue_debug_event(&self, flag: svc::DebugFlag) -> CtrResult {
        svc::continue_debug_event(&self.handle, flag)
    }

    pub fn eat_events(&self) -> CtrResult {
        svc::eat_events(&self.handle)
    }

    pub fn get_mem_info(&self, addr: u32) -> CtrResult<svc::MemQueryResponse> {
        svc::query_debug_process_memory(&self.handle, addr)
    }
}

impl TryFrom<Process> for DebugProcess {
    type Error = ResultCode;

    fn try_from(process: Process) -> CtrResult<Self> {
        Self::new_from_process(process)
    }
}
