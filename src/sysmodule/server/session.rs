use crate::{res::CtrResult, svc, Handle};

/// An individual session to a service.  
pub struct Session {
    handle: Handle,
    service_id: usize,
}

#[cfg_attr(test, mocktopus::macros::mockable)]
impl Session {
    /// Accepts a new session - for use a new session request has been received.
    ///
    /// When a request for this session is received, it will be handled by the provided request handler.
    pub fn accept_session(service_handle: &Handle, service_id: usize) -> CtrResult<Self> {
        let session_handle = svc::accept_session(service_handle)?;
        let session = Self {
            handle: session_handle,
            service_id,
        };
        Ok(session)
    }

    pub fn get_handle(&self) -> &Handle {
        &self.handle
    }

    pub fn service_id(&self) -> usize {
        self.service_id
    }
}
