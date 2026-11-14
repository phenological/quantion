use std::{
    collections::HashSet,
    process::Child,
    sync::{Arc, Mutex, MutexGuard},
};

use serde_json::Value;

#[derive(Default)]
struct RequestState {
    pending: HashSet<String>,
    cancelled: HashSet<String>,
    running_child: Option<(String, Child)>,
}

#[derive(Clone, Default)]
pub struct Requests {
    state: Arc<Mutex<RequestState>>,
}

impl Requests {
    pub fn add(&self, id: &Value) {
        self.lock().pending.insert(id.to_string());
    }

    pub fn cancel(&self, id: &Value) {
        let id = id.to_string();
        let mut state = self.lock();
        if !state.pending.contains(&id) {
            return;
        }
        state.cancelled.insert(id.clone());
        if let Some((running_id, child)) = &mut state.running_child
            && *running_id == id
        {
            let _ = child.kill();
        }
    }

    pub fn is_cancelled(&self, id: &Value) -> bool {
        self.lock().cancelled.contains(&id.to_string())
    }

    pub fn close(&self, id: &Value) -> bool {
        let id = id.to_string();
        let mut state = self.lock();
        state.pending.remove(&id);
        state.cancelled.remove(&id)
    }

    pub fn watch_child(&self, id: &Value, mut child: Child) {
        let id = id.to_string();
        let mut state = self.lock();
        if state.cancelled.contains(&id) {
            let _ = child.kill();
        }
        state.running_child = Some((id, child));
    }

    pub fn take_child(&self) -> Option<Child> {
        self.lock().running_child.take().map(|(_, child)| child)
    }

    fn lock(&self) -> MutexGuard<'_, RequestState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
