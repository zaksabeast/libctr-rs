use super::NotificationHandler;
use crate::{
    ptm,
    res::CtrResult,
    srv::{subscribe_notification, unsubscribe_notification},
};

/// A notification subscription with an associated handler that is unsubscribed when dropped.
pub(super) struct NotificationSubscription {
    pub(super) id: ptm::NotificationId,
    handler: NotificationHandler,
}

impl NotificationSubscription {
    pub fn new(id: ptm::NotificationId, handler: NotificationHandler) -> CtrResult<Self> {
        subscribe_notification(id)?;
        Ok(Self { id, handler })
    }

    pub fn handle_request(&self) -> CtrResult {
        (self.handler)(self.id as u32)
    }
}

impl Drop for NotificationSubscription {
    // There's not much we can do if this fails
    // and a failed unsubscription doesn't justify a panic
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        unsubscribe_notification(self.id);
    }
}
