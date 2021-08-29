use crate::res::{parse_result, CtrResult, ResultCode};
use ctru_sys::{
    ptmSysmExit, ptmSysmInit, PTMSYSM_NotifySleepPreparationComplete, PTMSYSM_ReplyToSleepQuery,
};

/// Initializes the ptm:sysm service.  Required to use ptm:sysm features.
pub fn sysm_init() -> CtrResult<ResultCode> {
    let result = unsafe { ptmSysmInit() };
    parse_result(result)
}

/// Exits the ptm:sysm service.
pub fn sysm_exit() {
    unsafe { ptmSysmExit() }
}

/// Notifies sleep preparation is complete.
pub fn sys_notify_sleep_preparation_complete(ack_value: i32) -> CtrResult<ResultCode> {
    let result = unsafe { PTMSYSM_NotifySleepPreparationComplete(ack_value) };
    parse_result(result)
}

/// Replies to the ptm::NotificationId::SleepRequested notification.  If denied, the console will not go to sleep.
pub fn sys_reply_to_sleep_query(deny: bool) -> CtrResult<ResultCode> {
    let result = unsafe { PTMSYSM_ReplyToSleepQuery(deny) };
    parse_result(result)
}
