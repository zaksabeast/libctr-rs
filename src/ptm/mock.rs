use crate::res::{CtrResult, ResultCode};

pub fn sysm_init() -> CtrResult<ResultCode> {
    Ok(0)
}

pub fn sysm_exit() {}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn sys_notify_sleep_preparation_complete(_ack_value: i32) -> CtrResult<ResultCode> {
    Ok(0)
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn sys_reply_to_sleep_query(_deny: bool) -> CtrResult<ResultCode> {
    Ok(0)
}
