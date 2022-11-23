use crate::{
    ptm,
    res::{parse_result, CtrResult},
    utils::cstring,
    Handle,
};
use cstr_core::CString;

/// Initializes the SRV service.  Required to use srv features.
#[ctr_macros::hos]
pub fn init() -> CtrResult {
    let result = unsafe { ctru_sys::srvInit() };
    parse_result(result)
}

#[ctr_macros::hos]
pub fn register_service(name: &str, max_sessions: i32) -> CtrResult<Handle> {
    let c_name = cstring::parse_result(CString::new(name))?;
    let mut raw_handle = 0;
    let result =
        unsafe { ctru_sys::srvRegisterService(&mut raw_handle, c_name.as_ptr(), max_sessions) };

    parse_result(result)?;

    Ok(raw_handle.into())
}

#[ctr_macros::hos]
pub fn enable_notifications() -> CtrResult<Handle> {
    let mut raw_handle = 0;
    let result = unsafe { ctru_sys::srvEnableNotification(&mut raw_handle) };

    parse_result(result)?;

    Ok(raw_handle.into())
}

#[ctr_macros::hos]
pub fn receive_notification() -> CtrResult<u32> {
    let mut notification_id = 0u32;
    let result = unsafe { ctru_sys::srvReceiveNotification(&mut notification_id) };

    parse_result(result)?;

    Ok(notification_id)
}

#[ctr_macros::hos]
pub fn subscribe_notification(notification_id: ptm::NotificationId) -> CtrResult {
    let result = unsafe { ctru_sys::srvSubscribe(notification_id as u32) };
    parse_result(result)
}

#[ctr_macros::hos]
pub fn unsubscribe_notification(notification_id: ptm::NotificationId) -> CtrResult {
    let result = unsafe { ctru_sys::srvUnsubscribe(notification_id as u32) };
    parse_result(result)
}

#[ctr_macros::hos]
pub fn unregister_service(name: &str) -> CtrResult {
    let c_name = cstring::parse_result(CString::new(name))?;
    let result = unsafe { ctru_sys::srvUnregisterService(c_name.as_ptr()) };
    parse_result(result)
}

#[ctr_macros::hos]
pub fn get_service_handle_direct(name: &str) -> CtrResult<Handle> {
    let mut raw_handle = 0;
    let c_name = cstring::parse_result(CString::new(name))?;
    let result = unsafe { ctru_sys::srvGetServiceHandleDirect(&mut raw_handle, c_name.as_ptr()) };
    parse_result(result)?;
    Ok(raw_handle.into())
}
