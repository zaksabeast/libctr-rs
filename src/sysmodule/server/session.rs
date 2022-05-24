use super::service::{RequestHandler, RequestHandlerResult};
use crate::{res::CtrResult, svc, Handle};

/// An individual session to a service.  
pub struct Session<ServiceContext> {
    handle: Handle,
    request_handler: RequestHandler<ServiceContext>,
}

#[cfg_attr(test, mocktopus::macros::mockable)]
impl<ServiceContext> Session<ServiceContext> {
    /// Accepts a new session - for use a new session request has been received.
    ///
    /// When a request for this session is received, it will be handled by the provided request handler.
    pub fn accept_session(
        service_handle: &Handle,
        request_handler: RequestHandler<ServiceContext>,
    ) -> CtrResult<Self> {
        let session_handle = svc::accept_session(service_handle)?;
        let session = Self {
            handle: session_handle,
            request_handler,
        };
        Ok(session)
    }

    pub fn handle_request(
        &self,
        context: &mut ServiceContext,
        session_index: usize,
    ) -> RequestHandlerResult {
        (self.request_handler)(context, session_index)
    }

    pub fn get_handle(&self) -> &Handle {
        &self.handle
    }
}
