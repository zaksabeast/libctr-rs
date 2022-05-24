use crate::{
    res::{CtrResult, GenericResultCode},
    svc, Handle,
};

pub struct Process {
    handle: Handle,
    process_id: u32,
}

impl Process {
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

    pub fn new_from_process_id(process_id: u32) -> CtrResult<Self> {
        let handle = svc::open_process(process_id)?;
        Ok(Self { handle, process_id })
    }

    pub fn new_from_title_id(title_id: u64) -> CtrResult<Self> {
        match Self::get_process_id_from_title_id(title_id) {
            Some(process_id) => Self::new_from_process_id(process_id),
            None => Err(GenericResultCode::InvalidValue.into()),
        }
    }

    pub fn get_title_id(&self) -> CtrResult<u64> {
        Self::get_process_title_id(&self.handle)
    }

    pub fn get_process_id(&self) -> u32 {
        self.process_id
    }

    pub fn copy_handle_to_process(&self, handle: &Handle) -> CtrResult<Handle> {
        let calling_process = Handle::get_current_process_handle();
        svc::copy_handle(&self.handle, handle, &calling_process)
    }
}
