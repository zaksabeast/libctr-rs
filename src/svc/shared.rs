use num_enum::IntoPrimitive;

/// [Read more at the libctru docs](https://github.com/devkitPro/libctru/blob/4e25fb1d6c2ea124a9011c4b65f76f2968a9fb97/libctru/include/3ds/svc.h#L333-L337)
pub enum UserBreakType {
    Panic = 0,
    Assert = 1,
    User = 2,
    LoadRo = 3,
    UnloadRo = 4,
}

/// [Read more at the libctru docs](https://github.com/devkitPro/libctru/blob/4e25fb1d6c2ea124a9011c4b65f76f2968a9fb97/libctru/include/3ds/svc.h#L106-L108)
pub enum EventResetType {
    OneShot = 0,
    Sticky = 1,
    Pulse = 2,
}

#[derive(IntoPrimitive, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DebugFlag {
    InhibitUserCpuExceptionHandlers = 1,
    SignalFaultExceptionEvents = 2,
    InhibitUserCpuExceptionHandlersAndSignalFaultExceptionEvents = 3,
    SignalScheduleEvents = 4,
    SignalSyscallEvents = 8,
    SignalMapEvents = 16,
}

#[derive(IntoPrimitive, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ProcessInfoType {
    TitleId = 0x10001,
    StartAddress = 0x10005,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemInfo {
    pub base_addr: u32,
    pub size: u32,
    pub perm: u32,
    pub state: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageInfo {
    pub flags: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemQueryResponse {
    pub mem_info: MemInfo,
    pub page_info: PageInfo,
}
