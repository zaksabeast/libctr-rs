use crate::res::{parse_result, CtrResult};
use ctru_sys::{
    hidExit, hidInit, hidKeysDown, hidKeysDownRepeat, hidKeysHeld, hidKeysUp, hidScanInput,
};

pub fn init() -> CtrResult<()> {
    let result = unsafe { hidInit() };
    parse_result(result)?;
    Ok(())
}

pub fn exit() {
    unsafe { hidExit() };
}

pub fn scan_input() {
    unsafe { hidScanInput() };
}

pub fn keys_held() -> u32 {
    unsafe { hidKeysHeld() }
}

pub fn keys_down() -> u32 {
    unsafe { hidKeysDown() }
}

pub fn keys_down_repeat() -> u32 {
    unsafe { hidKeysDownRepeat() }
}

pub fn keys_up() -> u32 {
    unsafe { hidKeysUp() }
}
