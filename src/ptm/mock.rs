use crate::res::CtrResult;

pub fn sysm_init() -> CtrResult {
    Ok(())
}

pub fn sysm_exit() {}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn sys_notify_sleep_preparation_complete(_ack_value: i32) -> CtrResult {
    Ok(())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn sys_reply_to_sleep_query(_deny: bool) -> CtrResult {
    Ok(())
}
