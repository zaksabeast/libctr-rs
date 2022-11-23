use crate::res::{parse_result, CtrResult};

#[ctr_macros::hos]
pub fn init() -> CtrResult {
    let result = unsafe { ctru_sys::hidInit() };
    parse_result(result)?;
    Ok(())
}

#[ctr_macros::hos]
pub fn exit() {
    unsafe { ctru_sys::hidExit() };
}

#[ctr_macros::hos]
pub fn scan_input() {
    unsafe { ctru_sys::hidScanInput() };
}

#[ctr_macros::hos]
pub fn keys_held() -> u32 {
    unsafe { ctru_sys::hidKeysHeld() }
}

#[ctr_macros::hos]
pub fn keys_down() -> u32 {
    unsafe { ctru_sys::hidKeysDown() }
}

#[ctr_macros::hos]
pub fn keys_down_repeat() -> u32 {
    unsafe { ctru_sys::hidKeysDownRepeat() }
}

#[ctr_macros::hos]
pub fn keys_up() -> u32 {
    unsafe { ctru_sys::hidKeysUp() }
}
