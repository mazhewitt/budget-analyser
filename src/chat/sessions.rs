use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::ai::llm::Message;

pub struct SessionEntry {
    pub history: Vec<Message>,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
}

#[derive(Clone)]
pub struct SessionStore {
    inner: Arc<RwLock<HashMap<String, SessionEntry>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_or_create(&self, conversation_id: Option<&str>) -> (String, Vec<Message>) {
        let id = conversation_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let mut store = self.inner.write().await;
        self.evict_expired(&mut store);

        let (history, is_new) = if let Some(entry) = store.get_mut(&id) {
            entry.last_accessed = SystemTime::now();
            (entry.history.clone(), false)
        } else {
            (Vec::new(), true)
        };

        if is_new {
            store.insert(
                id.clone(),
                SessionEntry {
                    history: Vec::new(),
                    created_at: SystemTime::now(),
                    last_accessed: SystemTime::now(),
                },
            );
        }

        (id, history)
    }

    pub async fn save_history(&self, conversation_id: &str, history: Vec<Message>) {
        let mut store = self.inner.write().await;
        if let Some(entry) = store.get_mut(conversation_id) {
            entry.history = history;
            entry.last_accessed = SystemTime::now();
        }
    }

    pub async fn delete(&self, conversation_id: &str) {
        let mut store = self.inner.write().await;
        store.remove(conversation_id);
    }

    fn evict_expired(&self, store: &mut HashMap<String, SessionEntry>) {
        let now = SystemTime::now();
        let ttl = Duration::from_secs(2 * 60 * 60);

        store.retain(|_, entry| {
            match now.duration_since(entry.last_accessed) {
                Ok(elapsed) => elapsed < ttl,
                Err(_) => true,
            }
        });
    }
}
