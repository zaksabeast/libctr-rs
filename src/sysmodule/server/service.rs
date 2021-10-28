use super::session::Session;
use crate::{
    ipc::{ThreadCommand, ThreadCommandParser},
    res::CtrResult,
    srv::{register_service, unregister_service},
    Handle,
};
use alloc::{borrow::ToOwned, string::String};

pub trait ServiceContext {
    fn accept_session(&mut self) {}
    fn close_session(&mut self, _session_index: usize) {}
}

pub type RequestHandlerResult<'a> = CtrResult<ThreadCommand<'a>>;
pub type RequestHandler<Context> =
    fn(&mut Context, ThreadCommandParser, usize) -> RequestHandlerResult<'_>;

/// A service that can receive commands from other processes.  It is unregistered when dropped.
pub struct Service<Context: ServiceContext> {
    pub handle: Handle,
    pub name: String,
    pub max_sessions: i32,
    request_handler: RequestHandler<Context>,
}

impl<Context: ServiceContext> Service<Context> {
    pub fn new(
        name: &str,
        max_sessions: i32,
        request_handler: RequestHandler<Context>,
    ) -> CtrResult<Self> {
        let handle = register_service(name, max_sessions)?;

        let service = Self {
            handle,
            name: name.to_owned(),
            max_sessions,
            request_handler,
        };

        Ok(service)
    }

    /// Accepts a new session - for use a new session request has been received.
    pub fn accept_session(&self) -> CtrResult<Session<Context>> {
        Session::accept_session(&self.handle, self.request_handler)
    }
}

impl<Context: ServiceContext> Drop for Service<Context> {
    // We're shutting down anyways, so ignore errors
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        unregister_service(&self.name);
    }
}
