use std::sync::{mpsc::Sender, Arc};

use dashmap::DashMap;

use crate::{schemas::Gamestate, services::ServiceError};

#[derive(Clone)]
pub struct Events {
    inner: Arc<DashMap<(uuid::Uuid, String), Vec<Sender<Gamestate>>>>,
}

impl Events {
    pub async fn send_event(
        &self,
        gs: Gamestate,
        game_id: uuid::Uuid,
        user_id: String,
    ) -> Result<(), ServiceError> {
        tracing::info!("send_event");
        match self.inner.get_mut(&(game_id, user_id.clone())) {
            None => Err(ServiceError::QueueError(
                "Unable to queue event".to_string(),
            )),
            Some(mut senders) => {
                senders.retain(|sender| sender.send(gs.clone()).is_ok());
                if senders.len() > 0 {
                    return Ok(());
                }
                Err(ServiceError::QueueError(
                    "Unable to queue event".to_string(),
                ))
            }
        }
    }

    pub async fn register(
        &self,
        game_id: uuid::Uuid,
        user_id: String,
        sender: Sender<Gamestate>,
    ) -> Result<(), ()> {
        let existing = self.inner.get_mut(&(game_id, user_id.clone()));
        match existing {
            None => {
                self.inner.insert((game_id, user_id), vec![sender]);
            }
            Some(mut value) => {
                value.push(sender);
            }
        }
        Ok(())
    }
    pub fn new() -> Self {
        return Self {
            inner: Arc::new(DashMap::new()),
        };
    }
}
