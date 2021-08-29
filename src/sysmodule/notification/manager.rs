use super::subscription::NotificationSubscription;
use crate::{
    ptm,
    res::{CtrResult, ResultCode},
    srv::{enable_notifications, receive_notification},
    Handle,
};
use alloc::{vec, vec::Vec};

pub type NotificationHandlerResult = CtrResult<()>;
pub type NotificationHandler = fn(u32) -> NotificationHandlerResult;

#[derive(Debug, PartialEq)]
pub enum NotificationType {
    /// A subscribed notification was handled.
    HandledSubscribed,
    /// A termination request was received.
    Termination,
    /// A notification was not handled.
    None,
}

/// Manages notification subscriptions
pub struct NotificationManager {
    handle: Handle,
    notification_subscriptions: Vec<NotificationSubscription>,
}

#[cfg_attr(test, mocktopus::macros::mockable)]
impl NotificationManager {
    pub fn new() -> CtrResult<Self> {
        let handle = enable_notifications()?;

        Ok(Self {
            handle,
            notification_subscriptions: vec![],
        })
    }

    pub fn subscribe(
        &mut self,
        notification_id: ptm::NotificationId,
        handler: NotificationHandler,
    ) -> CtrResult<ResultCode> {
        let notification_subscription = NotificationSubscription::new(notification_id, handler)?;
        self.notification_subscriptions
            .push(notification_subscription);
        Ok(0)
    }

    pub fn get_handle(&self) -> &Handle {
        &self.handle
    }

    /// Attempts to receive a notification and handle it with a previously provided subscription handler.
    pub fn handle_notification(&self) -> CtrResult<NotificationType> {
        let notification_id = receive_notification()?;

        if notification_id == ptm::NotificationId::Termination {
            return Ok(NotificationType::Termination);
        }

        let found_subscription = self
            .notification_subscriptions
            .iter()
            .find(|subscription| subscription.id == notification_id);

        if let Some(subscription) = found_subscription {
            subscription.handle_request()?;
            return Ok(NotificationType::HandledSubscribed);
        }

        Ok(NotificationType::None)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mocktopus::mocking::*;

    #[mocktopus::macros::mockable]
    fn mock_notification_handler(_: u32) -> NotificationHandlerResult {
        Ok(())
    }

    #[test]
    fn subscribe() {
        let mut manager = NotificationManager::new().unwrap();
        manager
            .subscribe(
                ptm::NotificationId::FullyWakingUp,
                mock_notification_handler,
            )
            .unwrap();

        assert_eq!(manager.notification_subscriptions.len(), 1);
    }

    mod handle_notification {
        use mocktopus::mocking::Mockable;

        use super::*;

        #[test]
        fn termination_notification() {
            let manager = NotificationManager::new().unwrap();
            receive_notification
                .mock_safe(|| MockResult::Return(Ok(ptm::NotificationId::Termination as u32)));

            let result = manager.handle_notification().unwrap();
            assert_eq!(result, NotificationType::Termination);
        }

        #[test]
        fn other_notification() {
            let mut manager = NotificationManager::new().unwrap();
            manager
                .subscribe(
                    ptm::NotificationId::FullyWakingUp,
                    mock_notification_handler,
                )
                .unwrap();
            receive_notification
                .mock_safe(|| MockResult::Return(Ok(ptm::NotificationId::FullyWakingUp as u32)));

            let result = manager.handle_notification().unwrap();
            assert_eq!(result, NotificationType::HandledSubscribed);
        }

        #[test]
        fn not_found_notification() {
            let manager = NotificationManager::new().unwrap();
            receive_notification
                .mock_safe(|| MockResult::Return(Ok(ptm::NotificationId::GoingToSleep as u32)));

            let result = manager.handle_notification().unwrap();
            assert_eq!(result, NotificationType::None);
        }

        #[test]
        fn forward_error_on_receive_notification_error() {
            let manager = NotificationManager::new().unwrap();
            receive_notification.mock_safe(|| MockResult::Return(Err(-1)));

            let result = manager.handle_notification();
            assert_eq!(result, Err(-1));
        }

        #[test]
        fn forward_error_on_handle_request_error() {
            mock_notification_handler.mock_safe(|_| MockResult::Return(Err(-1)));

            let mut manager = NotificationManager::new().unwrap();
            manager
                .subscribe(
                    ptm::NotificationId::FullyWakingUp,
                    mock_notification_handler,
                )
                .unwrap();
            receive_notification
                .mock_safe(|| MockResult::Return(Ok(ptm::NotificationId::FullyWakingUp as u32)));

            let result = manager.handle_notification();
            assert_eq!(result, Err(-1));
        }
    }
}
