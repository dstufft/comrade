use std::sync::Arc;
use std::time::Instant;

use crossbeam_channel::{Receiver, Sender};

use crate::config::triggers::Trigger;
use crate::config::Character;
use crate::watcher::LogEvent;

pub(crate) type EventSender = Sender<Event>;
pub(crate) type EventReceiver = Receiver<Event>;

#[derive(Debug)]
pub enum EventKind {
    Triggered {
        character: Arc<Character>,
        trigger: Arc<Trigger>,
        log: Arc<LogEvent>,
    },
    DisplayText(Arc<String>),
}

#[derive(Debug)]
pub struct Event {
    created: Instant,
    kind: EventKind,
}

impl Event {
    pub(crate) fn new(kind: EventKind) -> Event {
        Event {
            created: Instant::now(),
            kind,
        }
    }

    pub fn created(&self) -> Instant {
        self.created
    }

    pub fn kind(&self) -> &EventKind {
        &self.kind
    }
}
