use super::session::Session;
use crate::{
    res::CtrResult,
    srv::{register_service, unregister_service},
    Handle,
};

/// A service that can receive commands from other processes.  It is unregistered when dropped.
pub struct RegisteredService {
    pub handle: Handle,
    pub name: String,
    pub max_sessions: i32,
}

impl RegisteredService {
    pub fn new(name: &str, max_sessions: i32) -> CtrResult<Self> {
        let handle = register_service(name, max_sessions)?;

        let service = Self {
            handle,
            name: name.to_owned(),
            max_sessions,
        };

        Ok(service)
    }

    /// Accepts a new session - for use a new session request has been received.
    pub fn accept_session(&self, service_id: usize) -> CtrResult<Session> {
        Session::accept_session(&self.handle, service_id)
    }
}

impl Drop for RegisteredService {
    // We're shutting down anyways, so ignore errors
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        unregister_service(&self.name);
    }
}

pub trait Service {
    const ID: usize;
    const NAME: &'static str;
    const MAX_SESSION_COUNT: i32;

    fn register() -> CtrResult<RegisteredService> {
        RegisteredService::new(Self::NAME, Self::MAX_SESSION_COUNT)
    }
}
