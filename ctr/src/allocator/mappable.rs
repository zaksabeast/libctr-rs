#[ctr_macros::hos]
pub fn mappable_init(addr_min: u32, addr_max: u32) {
    unsafe { ctru_sys::mappableInit(addr_min, addr_max) }
}

#[ctr_macros::hos]
pub fn mappable_alloc(size: usize) -> *mut u8 {
    unsafe { ctru_sys::mappableAlloc(size as u32) as *mut u8 }
}
