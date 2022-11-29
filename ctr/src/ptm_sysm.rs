use crate::{
    res::{parse_result, CtrResult},
    service_session::create_session_manager,
};
use core::cmp::PartialEq;

create_session_manager!(ctru_sys, unsafe { ctru_sys::ptmSysmInit() }, unsafe {
    ctru_sys::ptmSysmExit()
});

/// Notifies sleep preparation is complete.
#[ctr_macros::hos]
pub fn sys_notify_sleep_preparation_complete(ack_value: i32) -> CtrResult {
    let result = unsafe { ctru_sys::PTMSYSM_NotifySleepPreparationComplete(ack_value) };
    parse_result(result)
}

/// Replies to the ptm::NotificationId::SleepRequested notification.  If denied, the console will not go to sleep.
#[ctr_macros::hos]
pub fn sys_reply_to_sleep_query(deny: bool) -> CtrResult {
    let result = unsafe { ctru_sys::PTMSYSM_ReplyToSleepQuery(deny) };
    parse_result(result)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationId {
    Termination = 0x100,
    SleepRequested = 0x101,
    GoingToSleep = 0x104,
    FullyWakingUp = 0x105,
    HalfAwake = 0x107,
    LaunchApp = 0x10c, // Luma only
}

impl PartialEq<u32> for NotificationId {
    fn eq(&self, other: &u32) -> bool {
        (*self as u32) == *other
    }
}

impl PartialEq<NotificationId> for u32 {
    fn eq(&self, other: &NotificationId) -> bool {
        *self == (*other as u32)
    }
}

/// Returns the value to acknowledge a notification.
pub fn sys_get_notification_ack_value(id: u32) -> i32 {
    let ack_values = [3, -1, 1, 0, 0, -1, 2];

    if !((NotificationId::SleepRequested as u32)..=(NotificationId::HalfAwake as u32)).contains(&id)
    {
        return -1;
    }

    let ack_value_index = (id - NotificationId::SleepRequested as u32) as usize;
    match ack_values.get(ack_value_index) {
        Some(ack_value) => *ack_value,
        None => -1,
    }
}
