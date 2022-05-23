use crossterm::event;
use std::cell::RefCell;
use std::sync::Arc;

use comrade::events::{Event, EventKind};

use crate::app::{Eventable, Result, Tab};

pub(crate) struct EventsTab {
    title: String,
    messages: RefCell<Vec<Arc<String>>>,
}

impl EventsTab {
    pub(in crate::app) fn init<T: Into<String>>(title: T) -> Box<dyn Tab> {
        Box::new(EventsTab {
            title: title.into(),
            messages: RefCell::new(Vec::new()),
        })
    }

    pub(in crate::app) fn event(&self, event: Event) {
        match event.kind() {
            EventKind::DisplayText(text) => {
                let mut messages = self.messages.borrow_mut();
                messages.insert(0, text.clone());

                let len = messages.len();
                if len > 100 {
                    messages.drain(100..len);
                }
            }
        }
    }

    pub(crate) fn messages(&self) -> Vec<String> {
        self.messages
            .borrow()
            .iter()
            .map(|t| t.to_string())
            .collect()
    }
}

impl Eventable for EventsTab {
    fn on_event(&self, _event: event::Event) -> Result<()> {
        Ok(())
    }
}

impl Tab for EventsTab {
    fn id(&self) -> &str {
        "events"
    }

    fn title(&self) -> &str {
        self.title.as_str()
    }
}
