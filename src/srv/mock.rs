use crate::{
    ptm,
    res::{CtrResult, ResultCode},
    Handle,
};

pub fn init() -> CtrResult<ResultCode> {
    Ok(0)
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn register_service(_name: &str, _max_sessions: i32) -> CtrResult<Handle> {
    Ok(0.into())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn enable_notifications() -> CtrResult<Handle> {
    Ok(0.into())
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn receive_notification() -> CtrResult<u32> {
    Ok(0)
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn subscribe_notification(_notification_id: ptm::NotificationId) -> CtrResult<ResultCode> {
    Ok(0)
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn unsubscribe_notification(_notification_id: ptm::NotificationId) -> CtrResult<ResultCode> {
    Ok(0)
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn unregister_service(_name: &str) -> CtrResult<ResultCode> {
    Ok(0)
}

#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
pub fn get_service_handle_direct(_name: &str) -> CtrResult<Handle> {
    Ok(0.into())
}
