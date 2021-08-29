use crate::{
    ptm,
    res::{parse_result, CtrResult, ResultCode},
    utils::cstring,
    Handle,
};
use cstr_core::CString;
use ctru_sys::{
    srvEnableNotification, srvGetServiceHandleDirect, srvInit, srvReceiveNotification,
    srvRegisterService, srvSubscribe, srvUnregisterService, srvUnsubscribe,
};

/// Initializes the SRV service.  Required to use srv features.
pub fn init() -> CtrResult<ResultCode> {
    let result = unsafe { srvInit() };
    parse_result(result)
}

pub fn register_service(name: &str, max_sessions: i32) -> CtrResult<Handle> {
    let c_name = cstring::parse_result(CString::new(name))?;
    let mut raw_handle = 0;
    let result = unsafe { srvRegisterService(&mut raw_handle, c_name.as_ptr(), max_sessions) };

    parse_result(result)?;

    Ok(raw_handle.into())
}

pub fn enable_notifications() -> CtrResult<Handle> {
    let mut raw_handle = 0;
    let result = unsafe { srvEnableNotification(&mut raw_handle) };

    parse_result(result)?;

    Ok(raw_handle.into())
}

pub fn receive_notification() -> CtrResult<u32> {
    let mut notification_id = 0u32;
    let result = unsafe { srvReceiveNotification(&mut notification_id) };

    parse_result(result)?;

    Ok(notification_id)
}

pub fn subscribe_notification(notification_id: ptm::NotificationId) -> CtrResult<ResultCode> {
    let result = unsafe { srvSubscribe(notification_id as u32) };
    parse_result(result)
}

pub fn unsubscribe_notification(notification_id: ptm::NotificationId) -> CtrResult<ResultCode> {
    let result = unsafe { srvUnsubscribe(notification_id as u32) };
    parse_result(result)
}

pub fn unregister_service(name: &str) -> CtrResult<ResultCode> {
    let c_name = cstring::parse_result(CString::new(name))?;
    let result = unsafe { srvUnregisterService(c_name.as_ptr()) };
    parse_result(result)
}

pub fn get_service_handle_direct(name: &str) -> CtrResult<Handle> {
    let mut raw_handle = 0;
    let c_name = cstring::parse_result(CString::new(name))?;
    let result = unsafe { srvGetServiceHandleDirect(&mut raw_handle, c_name.as_ptr()) };
    parse_result(result)?;
    Ok(raw_handle.into())
}
