use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};

pub(crate) type EventSender = Sender<Event>;
pub(crate) type EventReceiver = Receiver<Event>;

#[derive(Debug)]
pub enum EventKind {
    DisplayText(Arc<String>),
}

#[derive(Debug)]
pub struct Event {
    kind: EventKind,
}

impl Event {
    pub(crate) fn new(kind: EventKind) -> Event {
        Event { kind }
    }

    pub fn kind(&self) -> &EventKind {
        &self.kind
    }
}
