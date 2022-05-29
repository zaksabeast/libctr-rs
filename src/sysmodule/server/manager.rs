use super::{service::RegisteredService, session::Session, ServiceRouter};
use crate::{
    ipc::{set_static_buffer, Command, StaticBuffer},
    res::{CtrResult, GenericResultCode, ResultCode},
    sysmodule::notification::{NotificationManager, NotificationType},
};
use alloc::{vec, vec::Vec};
use core::iter;

#[derive(PartialEq, Debug)]
enum ReplyAndReceiveResult {
    Notification,
    SessionRequest(usize),
    ServiceCommand(usize),
    ClosedSession(usize),
    Err(ResultCode),
}

/// The service manager handles accepting sessions to services, closing sessions,
/// fullfilling session command requests, and handling notifications.
///
/// It assumes the session limits requested by its Services are respected.
pub struct ServiceManager<Router: ServiceRouter> {
    services: Vec<RegisteredService>,
    sessions: Vec<Session>,
    notification_manager: NotificationManager,
    reply_target: Option<usize>,
    router: Router,
}

#[cfg_attr(test, mocktopus::macros::mockable)]
impl<Router: ServiceRouter> ServiceManager<Router> {
    pub fn new(
        services: Vec<RegisteredService>,
        notification_manager: NotificationManager,
        router: Router,
    ) -> Self {
        Self {
            services,
            notification_manager,
            router,
            sessions: vec![],
            reply_target: None,
        }
    }

    fn get_raw_handles_and_adjusted_reply_target(&self) -> (Vec<u32>, Option<usize>) {
        // Sending a copy of a handle to another process is memory safe, and this symodule isn't keeping a copy locally
        let raw_service_handles: Vec<u32> = unsafe {
            self.services
                .iter()
                .map(|service| service.handle.get_raw())
                .collect()
        };

        let raw_session_handles: Vec<u32> = unsafe {
            self.sessions
                .iter()
                .map(|session| session.get_handle().get_raw())
                .collect()
        };

        let adjusted_reply_target = self
            .reply_target
            .map(|target_index| target_index + 1 + self.services.len());

        let raw_handles: Vec<u32> = unsafe {
            iter::once(self.notification_manager.get_handle().get_raw())
                .chain(raw_service_handles)
                .chain(raw_session_handles)
                .collect()
        };

        (raw_handles, adjusted_reply_target)
    }

    fn parse_reply_and_receive_result(&self, result: (usize, ResultCode)) -> ReplyAndReceiveResult {
        let (index, result_code) = result;
        let raw_result_code = result_code.into_raw();
        match (index, raw_result_code) {
            (index, 0xc920181a) => {
                if index == 0xffffffff {
                    if let Some(reply_target) = self.reply_target {
                        ReplyAndReceiveResult::ClosedSession(reply_target)
                    } else {
                        ReplyAndReceiveResult::Err(GenericResultCode::InvalidValue.into())
                    }
                } else {
                    ReplyAndReceiveResult::ClosedSession(index - 1 - self.services.len())
                }
            }
            (_, _raw_result_code) if result_code.get_is_error() => {
                ReplyAndReceiveResult::Err(result_code)
            }
            (0, _) => ReplyAndReceiveResult::Notification,
            // This guard assumes svc::reply_and_receive sends the notification handle
            // before service handles
            (index, _) if index < 1 + self.services.len() => {
                ReplyAndReceiveResult::SessionRequest(index - 1)
            }
            (index, _) => ReplyAndReceiveResult::ServiceCommand(index - 1 - self.services.len()),
        }
    }

    fn set_thread_ready(&self) -> (usize, ResultCode) {
        let (raw_handles, reply_target) = self.get_raw_handles_and_adjusted_reply_target();
        Command::new(0xffff, ())
            .write()
            .reply_and_receive(&raw_handles, reply_target)
    }

    fn run_command(&mut self, session_index: usize) -> (usize, ResultCode) {
        let (raw_handles, reply_target) = self.get_raw_handles_and_adjusted_reply_target();
        let command_id = <Command>::current_command_id();
        let service_id = self.sessions[session_index].service_id();

        let response = self
            .router
            .handle_request(service_id, session_index)
            .unwrap_or_else(|result_code| {
                if GenericResultCode::InvalidCommand == result_code {
                    Command::new_from_parts(0x0u16, 0x1, 0x0, 0xd9001830u32).write()
                } else {
                    Command::new_from_parts(command_id, 0x1, 0x0, result_code).write()
                }
            });

        response.reply_and_receive(&raw_handles, reply_target)
    }

    /// The main loop of the service manager, and the system module by extension.
    ///
    /// This will run until a termination request is received.
    /// It is responsible for replying to targets and handling requests.
    pub fn run(&mut self) -> CtrResult {
        let first: [u8; 0x800] = [0; 0x800];
        let second: [u8; 0x800] = [0; 0x800];
        let third: [u8; 0x800] = [0; 0x800];
        set_static_buffer(&StaticBuffer::new(&first, 0));
        set_static_buffer(&StaticBuffer::new(&second, 1));
        set_static_buffer(&StaticBuffer::new(&third, 2));

        let mut response = self.set_thread_ready();

        loop {
            match self.parse_reply_and_receive_result(response) {
                ReplyAndReceiveResult::Err(result_code) => Err(result_code),
                ReplyAndReceiveResult::ClosedSession(index) => {
                    self.sessions.remove(index);
                    self.router.close_session(index);
                    self.reply_target = None;
                    response = self.set_thread_ready();
                    Ok(0)
                }
                ReplyAndReceiveResult::Notification => {
                    let notification_type = self.notification_manager.handle_notification()?;
                    if notification_type == NotificationType::Termination {
                        break;
                    }

                    self.reply_target = None;
                    response = self.set_thread_ready();
                    Ok(0)
                }
                ReplyAndReceiveResult::SessionRequest(index) => {
                    let session = self.services[index].accept_session(index)?;

                    self.sessions.push(session);
                    self.router.accept_session(self.sessions.len() - 1);
                    self.reply_target = None;
                    response = self.set_thread_ready();
                    Ok(0)
                }
                ReplyAndReceiveResult::ServiceCommand(index) => {
                    self.reply_target = Some(index);
                    response = self.run_command(index);
                    Ok(0)
                }
            }?;
        }

        Ok(())
    }
}
