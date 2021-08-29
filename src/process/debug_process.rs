use super::Process;
use crate::{
    res::{parse_result, CtrResult, ResultCode},
    safe_transmute::transmute_one_pedantic,
    svc, Handle,
};
use alloc::vec::Vec;
use core::{convert::TryFrom, mem};
use safe_transmute::TriviallyTransmutable;

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

    pub fn read<T: TriviallyTransmutable>(&self, addr: u32) -> CtrResult<T> {
        let bytes = self.read_bytes(addr, mem::size_of::<T>() as u32)?;
        transmute_one_pedantic(&bytes)
    }

    pub fn write_bytes(&self, addr: u32, buffer: &[u8]) -> CtrResult<()> {
        svc::write_process_memory(&self.handle, buffer, addr)
    }

    pub fn get_debug_event(&self) -> CtrResult<()> {
        let result = svc::get_process_debug_event(&self.handle);
        parse_result(result)?;
        Ok(())
    }

    pub fn continue_debug_event(&self, flag: svc::DebugFlag) -> CtrResult<()> {
        svc::continue_debug_event(&self.handle, flag)
    }

    pub fn eat_events(&self) -> CtrResult<()> {
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
