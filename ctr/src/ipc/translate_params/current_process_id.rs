use no_std_io::{EndianRead, EndianWrite};

const CURRENT_PROCESS_ID_DESCRIPTOR: u32 = 0x20;

#[derive(Debug, EndianRead, EndianWrite)]
pub struct CurrentProcessId {
    descriptor: u32,
    process_id: u32,
}

impl CurrentProcessId {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn raw(&self) -> u32 {
        self.process_id
    }
}

impl Default for CurrentProcessId {
    fn default() -> Self {
        CurrentProcessId {
            descriptor: CURRENT_PROCESS_ID_DESCRIPTOR,
            process_id: 0,
        }
    }
}
