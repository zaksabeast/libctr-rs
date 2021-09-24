use super::{service::Service, session::Session, ServiceContext};
use crate::{
    ipc::{set_static_buffers, ThreadCommandBuilder, ThreadCommandParser},
    res::{check_if_fail, CtrResult, GenericResultCode, ResultCode},
    sysmodule::notification::{NotificationManager, NotificationType},
};
use alloc::{boxed::Box, vec, vec::Vec};
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
pub struct ServiceManager<Context: ServiceContext> {
    services: Vec<Service<Context>>,
    sessions: Vec<Session<Context>>,
    notification_manager: NotificationManager,
    reply_target: Option<usize>,
    global_context: Box<Context>,
}

#[cfg_attr(test, mocktopus::macros::mockable)]
impl<Context: ServiceContext> ServiceManager<Context> {
    pub fn new(
        services: Vec<Service<Context>>,
        notification_manager: NotificationManager,
        global_context: Box<Context>,
    ) -> Self {
        Self {
            services,
            notification_manager,
            global_context,
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
        match result {
            (index, -0x36dfe7e6) => {
                if index == 0xffffffff {
                    if let Some(reply_target) = self.reply_target {
                        ReplyAndReceiveResult::ClosedSession(reply_target)
                    } else {
                        ReplyAndReceiveResult::Err(0)
                    }
                } else {
                    ReplyAndReceiveResult::ClosedSession(index - 1 - self.services.len())
                }
            }
            (_, result_code) if check_if_fail(result_code) => {
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
        ThreadCommandBuilder::new(0xffffu16)
            .build()
            .reply_and_receive(&raw_handles, reply_target)
    }

    fn run_command(&mut self, session_index: usize) -> (usize, ResultCode) {
        let (raw_handles, reply_target) = self.get_raw_handles_and_adjusted_reply_target();
        let command_parser = ThreadCommandParser::new();

        let command_id = command_parser.get_command_id();

        let response = self.sessions[session_index]
            .handle_request(&mut self.global_context, command_parser, session_index)
            .unwrap_or_else(|result_code| {
                if GenericResultCode::InvalidCommand == result_code {
                    let mut command_response = ThreadCommandBuilder::new(0u16);
                    command_response.push(0xd9001830u32);
                    command_response.build()
                } else {
                    let mut command_response = ThreadCommandBuilder::new(command_id);
                    command_response.push(result_code as u32);
                    command_response.build()
                }
            });

        response.reply_and_receive(&raw_handles, reply_target)
    }

    /// The main loop of the service manager, and the system module by extension.
    ///
    /// This will run until a termination request is received.
    /// It is responsible for replying to targets and handling requests.
    pub fn run(&mut self) -> CtrResult<ResultCode> {
        let zeros: [u8; 0x100] = [0; 0x100];
        set_static_buffers(&zeros);

        let mut response = self.set_thread_ready();

        loop {
            match self.parse_reply_and_receive_result(response) {
                ReplyAndReceiveResult::Err(result_code) => Err(result_code),
                ReplyAndReceiveResult::ClosedSession(index) => {
                    self.sessions.remove(index);
                    self.global_context.close_session(index);
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
                    let session = self.services[index].accept_session()?;

                    self.sessions.push(session);
                    self.global_context.accept_session();
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

        Ok(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ipc::ThreadCommandBuilder, sysmodule::server::RequestHandlerResult, Handle};
    use mocktopus::mocking::*;

    struct Context {}

    impl ServiceContext for Context {}

    type ServiceManagerWithContext = ServiceManager<Context>;

    #[mocktopus::macros::mockable]
    fn handle_test_service_request(
        context: &mut Context,
        command_parser: ThreadCommandParser,
        _session_index: usize,
    ) -> RequestHandlerResult {
        let command = ThreadCommandBuilder::new(1u16);
        Ok(command.build())
    }

    fn create_test_service_manager() -> ServiceManagerWithContext {
        let service = Service::new("test", 1, handle_test_service_request).unwrap();
        let services = vec![service];
        let notification_manger = NotificationManager::new().unwrap();
        let global_context = Box::new(Context {});
        let session_handle = Handle::from(0);
        let session =
            Session::accept_session(&session_handle, handle_test_service_request).unwrap();

        let mut manager = ServiceManager::new(services, notification_manger, global_context);
        manager.sessions.push(session);

        manager
    }

    fn create_test_service_manager_for_runner(
        mut mock_parsed_results: Vec<ReplyAndReceiveResult>,
    ) -> ServiceManagerWithContext {
        // This will cause ServiceManager::run to iterate over the mocked results
        ServiceManagerWithContext::parse_reply_and_receive_result
            .mock_safe(move |_, _| MockResult::Return(mock_parsed_results.remove(0)));

        // By default, when a notification is mocked it will cause ServiceManager::run to terminate
        // and avoid an infinite loop
        NotificationManager::handle_notification
            .mock_safe(|_| MockResult::Return(Ok(NotificationType::Termination)));

        create_test_service_manager()
    }

    mod parse_reply_and_receive_result {
        use super::*;

        #[test]
        fn parse_notification() {
            let manager = create_test_service_manager();
            let result = manager.parse_reply_and_receive_result((0, 0));
            assert_eq!(result, ReplyAndReceiveResult::Notification);
        }

        #[test]
        fn parse_session_request() {
            let handle_index = 1;
            let service_index = handle_index - 1;

            let manager = create_test_service_manager();
            let result = manager.parse_reply_and_receive_result((handle_index, 0));

            assert_eq!(result, ReplyAndReceiveResult::SessionRequest(service_index));
        }

        #[test]
        fn parse_service_command() {
            let manager = create_test_service_manager();

            let handle_index = 2;
            let session_index = handle_index - 1 - manager.services.len();

            let result = manager.parse_reply_and_receive_result((handle_index, 0));

            assert_eq!(result, ReplyAndReceiveResult::ServiceCommand(session_index));
        }

        #[test]
        fn parse_closed_session_for_reply_target() {
            let reply_target_index = 1;

            let mut manager = create_test_service_manager();
            manager.reply_target = Some(reply_target_index);
            let result = manager.parse_reply_and_receive_result((0xffffffff, -0x36dfe7e6));

            assert_eq!(
                result,
                ReplyAndReceiveResult::ClosedSession(reply_target_index)
            );
        }

        #[test]
        fn err_closed_session_if_reply_target_is_none() {
            let mut manager = create_test_service_manager();
            manager.reply_target = None;
            let result = manager.parse_reply_and_receive_result((0xffffffff, -0x36dfe7e6));
            assert_eq!(result, ReplyAndReceiveResult::Err(0));
        }

        #[test]
        fn parse_closed_session_for_index() {
            let manager = create_test_service_manager();

            let handle_index = 3;
            let session_index = handle_index - 1 - manager.services.len();

            let result = manager.parse_reply_and_receive_result((handle_index, -0x36dfe7e6));

            assert_eq!(result, ReplyAndReceiveResult::ClosedSession(session_index));
        }

        #[test]
        fn parse_err() {
            let manager = create_test_service_manager();
            let result = manager.parse_reply_and_receive_result((0, -1));
            assert_eq!(result, ReplyAndReceiveResult::Err(-1));
        }
    }

    mod run {
        use super::*;
        use crate::Handle;

        #[test]
        fn error() {
            let mock_parsed_results = vec![ReplyAndReceiveResult::Err(0xc0de)];
            let mut manager = create_test_service_manager_for_runner(mock_parsed_results);

            let result = manager.run();

            assert_eq!(result, Err(0xc0de));
        }

        #[test]
        fn close_session() {
            let mock_parsed_results = vec![
                ReplyAndReceiveResult::ClosedSession(0),
                ReplyAndReceiveResult::Notification,
            ];
            let mut manager = create_test_service_manager_for_runner(mock_parsed_results);
            manager.sessions.clear();

            let mock_session =
                Session::accept_session(&Handle::from(0), handle_test_service_request).unwrap();
            manager.sessions.push(mock_session);

            let result = manager.run();

            assert_eq!(result, Ok(0));
            assert_eq!(manager.sessions.len(), 0);
            assert_eq!(manager.reply_target, None);
        }

        #[test]
        fn termination_notification_request() {
            let mock_parsed_results = vec![ReplyAndReceiveResult::Notification];
            let mut manager = create_test_service_manager_for_runner(mock_parsed_results);

            let result = manager.run();

            assert_eq!(result, Ok(0));
        }

        #[test]
        fn notification_request() {
            let mock_parsed_results = vec![
                ReplyAndReceiveResult::Notification,
                ReplyAndReceiveResult::Notification,
            ];
            let mut manager = create_test_service_manager_for_runner(mock_parsed_results);

            let mut mock_notifications = vec![
                NotificationType::HandledSubscribed,
                NotificationType::Termination,
            ];
            NotificationManager::handle_notification
                .mock_safe(move |_| MockResult::Return(Ok(mock_notifications.remove(0))));

            let result = manager.run();

            assert_eq!(result, Ok(0));
            assert_eq!(manager.reply_target, None);
        }

        #[test]
        fn forward_notification_error() {
            let mock_parsed_results = vec![ReplyAndReceiveResult::Notification];
            let mut manager = create_test_service_manager_for_runner(mock_parsed_results);

            NotificationManager::handle_notification.mock_safe(|_| MockResult::Return(Err(-1)));

            let result = manager.run();

            assert_eq!(result, Err(-1));
        }

        #[test]
        fn session_request() {
            let mock_parsed_results = vec![
                ReplyAndReceiveResult::SessionRequest(0),
                ReplyAndReceiveResult::Notification,
            ];
            let mut manager = create_test_service_manager_for_runner(mock_parsed_results);
            manager.sessions.clear();

            let result = manager.run();

            assert_eq!(result, Ok(0));
            assert_eq!(manager.sessions.len(), 1);
            assert_eq!(manager.reply_target, None);
        }

        #[test]
        fn forward_session_request_error() {
            let mock_parsed_results = vec![
                ReplyAndReceiveResult::SessionRequest(0),
                ReplyAndReceiveResult::Notification,
            ];
            let mut manager = create_test_service_manager_for_runner(mock_parsed_results);

            Session::<Context>::accept_session.mock_safe(|_, _| MockResult::Return(Err(-1)));

            let result = manager.run();

            assert_eq!(result, Err(-1));
        }

        #[test]
        fn service_command() {
            let session_index = 0;
            let mock_parsed_results = vec![
                ReplyAndReceiveResult::ServiceCommand(session_index),
                ReplyAndReceiveResult::Notification,
            ];
            let mut manager = create_test_service_manager_for_runner(mock_parsed_results);

            let mock_session =
                Session::accept_session(&Handle::from(0), handle_test_service_request).unwrap();
            manager.sessions = vec![mock_session];

            let result = manager.run();

            assert_eq!(result, Ok(0));
            assert_eq!(manager.reply_target, Some(session_index));
        }
    }

    mod run_command {
        use super::*;
        use crate::ipc::{ThreadCommand, ThreadCommandBuilder};

        #[test]
        fn create_an_error_response_on_error() {
            let mock_parsed_results = vec![ReplyAndReceiveResult::ServiceCommand(0)];
            let mut manager = create_test_service_manager_for_runner(mock_parsed_results);

            ThreadCommandParser::new.mock_safe(|| {
                let command = ThreadCommandBuilder::new(0xAAu16);
                MockResult::Return(command.build().into())
            });

            Session::handle_request.mock_safe(
                |_self: &Session<()>, _context, _command_parser, _session_index| {
                    MockResult::Return(Err(GenericResultCode::InvalidString.into()))
                },
            );

            ThreadCommand::reply_and_receive.mock_safe(
                |thread_command, raw_handles, reply_target| {
                    let param_pool = thread_command.param_pool;
                    assert_eq!(param_pool[0], 0xAA);
                    assert_eq!(param_pool[1], u32::from(GenericResultCode::InvalidString));
                    MockResult::Continue((thread_command, raw_handles, reply_target))
                },
            );

            manager.run_command(0);
        }

        #[test]
        fn create_an_invalid_command_response_on_invalid_command_error() {
            let mock_parsed_results = vec![ReplyAndReceiveResult::ServiceCommand(0)];
            let mut manager = create_test_service_manager_for_runner(mock_parsed_results);

            ThreadCommandParser::new.mock_safe(|| {
                let command = ThreadCommandBuilder::new(0xAAu16);
                MockResult::Return(command.build().into())
            });

            Session::handle_request.mock_safe(
                |_self: &Session<()>, _context, _command_parser, _session_index| {
                    MockResult::Return(Err(GenericResultCode::InvalidCommand.into()))
                },
            );

            ThreadCommand::reply_and_receive.mock_safe(
                |thread_command, raw_handles, reply_target| {
                    let param_pool = thread_command.param_pool;
                    assert_eq!(param_pool[0], 0);
                    assert_eq!(param_pool[1], 0xd9001830u32);
                    MockResult::Continue((thread_command, raw_handles, reply_target))
                },
            );

            manager.run_command(0);
        }
    }
}
