use core::cmp::PartialEq;

#[derive(Clone, Copy, Debug, PartialEq)]
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
#[cfg_attr(not(target_os = "horizon"), mocktopus::macros::mockable)]
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
