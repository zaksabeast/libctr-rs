#[repr(C)]
struct OsKernelConfig {
    kernel_ver: u32,
    update_flag: u32,
    ns_tid: u64,
    kernel_syscore_ver: u32,
    env_info: u8,
    unit_info: u8,
    boot_env: u8,
    unk_0x17: u8,
    kernel_ctrsdk_ver: u32,
    unk_0x1c: u32,
    firmlaunch_flags: u32,
    unk_0x24: [u8; 0xc],
    app_memtype: u32,
    unk_0x34: [u8; 0xc],
    memregion_sz: [u32; 3],
    unk_0x4c: [u8; 0x14],
    firm_ver: u32,
    firm_syscore_ver: u32,
    firm_ctrsdk_ver: u32,
}

const OS_KERNELCFG_VADDR: *const OsKernelConfig = 0x1FF80000 as *const OsKernelConfig;

#[ctr_macros::hos]
fn get_os_kernel_config() -> &'static OsKernelConfig {
    unsafe { &*OS_KERNELCFG_VADDR }
}

#[derive(Debug, PartialEq, Eq)]
pub struct KernelVersion {
    raw: u32,
}

impl KernelVersion {
    pub fn new(major: u8, minor: u8, revision: u8) -> Self {
        Self {
            raw: (major as u32) << 24 | (minor as u32) << 16 | (revision as u32) << 8,
        }
    }

    pub fn major(&self) -> u8 {
        (self.raw >> 24) as u8
    }

    pub fn minor(&self) -> u8 {
        (self.raw >> 16) as u8
    }

    pub fn revision(&self) -> u8 {
        (self.raw >> 8) as u8
    }
}

impl From<u32> for KernelVersion {
    fn from(raw: u32) -> Self {
        Self { raw: raw & !0xff }
    }
}

pub struct Kernel();

impl Kernel {
    pub fn version() -> KernelVersion {
        get_os_kernel_config().kernel_ver.into()
    }
}

/// Returns the OS time in milliseconds.
#[ctr_macros::hos]
pub fn get_time() -> u64 {
    unsafe { ctru_sys::osGetTime() }
}
