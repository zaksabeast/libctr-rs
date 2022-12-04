use crate::{
    res::{error, CtrResult},
    svc::{self, MemQueryResponse},
    Handle,
};

pub struct Process {
    handle: Handle,
}

impl Process {
    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    pub fn get_process_title_id(process: &Handle) -> CtrResult<u64> {
        let title_id = svc::get_process_info(process, svc::ProcessInfoType::TitleId)?;
        let title_id_bytes = title_id.to_ne_bytes();
        Ok(u64::from_ne_bytes(title_id_bytes))
    }

    pub fn get_process_id_from_title_id(title_id: u64) -> Option<u32> {
        let process_list = svc::get_process_list().unwrap();

        process_list.iter().find_map(|process_id| {
            let process = svc::open_process(*process_id).unwrap();
            let process_title_id = Self::get_process_title_id(&process).unwrap();

            if process_title_id == title_id {
                Some(*process_id)
            } else {
                None
            }
        })
    }

    pub fn current() -> Process {
        Process {
            handle: Handle::CUR_PROCESS,
        }
    }

    pub fn new_from_process_id(process_id: u32) -> CtrResult<Self> {
        let handle = svc::open_process(process_id)?;
        Ok(Self { handle })
    }

    pub fn new_from_title_id(title_id: u64) -> CtrResult<Self> {
        match Self::get_process_id_from_title_id(title_id) {
            Some(process_id) => Self::new_from_process_id(process_id),
            None => Err(error::invalid_value()),
        }
    }

    pub fn get_title_id(&self) -> CtrResult<u64> {
        Self::get_process_title_id(&self.handle)
    }

    pub fn get_process_id(&self) -> CtrResult<u32> {
        svc::get_process_id(&self.handle)
    }

    pub fn copy_handle_to_process(&self, handle: &Handle) -> CtrResult<Handle> {
        svc::copy_handle(&self.handle, handle, &Handle::CUR_PROCESS)
    }

    pub fn query_memory(&self, addr: u32) -> CtrResult<MemQueryResponse> {
        svc::query_process_memory(&self.handle, addr)
    }

    pub fn invalidate_process_data_cache(&self, addr: u32, size: usize) -> CtrResult {
        svc::invalidate_process_data_cache(&self.handle, addr, size)
    }

    pub fn flush_process_data_cache(&self, addr: u32, size: usize) -> CtrResult {
        svc::flush_process_data_cache(&self.handle, addr, size)
    }
}
