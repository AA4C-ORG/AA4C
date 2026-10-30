use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use aa4c_types::Event;
use serde_json::Value;

/// 事件总线：每个订阅者一个队列；订阅者丢弃接收端后自动清理。
#[derive(Debug, Default)]
pub(crate) struct EventBus {
    subscribers: Mutex<Vec<Sender<Event>>>,
}

impl EventBus {
    pub(crate) fn subscribe(&self) -> Receiver<Event> {
        let (tx, rx) = channel();
        self.subscribers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(tx);
        rx
    }

    pub(crate) fn publish(&self, kind: &str, data: Value) {
        let at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
            .unwrap_or(0);
        let event = Event {
            kind: kind.to_string(),
            at,
            data,
        };
        self.subscribers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .retain(|tx| tx.send(event.clone()).is_ok());
    }
}
