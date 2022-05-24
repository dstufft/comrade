use crossterm::event;
use std::cell::RefCell;
use std::sync::Arc;

use comrade::events::{Event, EventKind};

use crate::app::{Eventable, Result, Tab};

pub(crate) struct EventsTab {
    title: String,
    messages: RefCell<Vec<Arc<String>>>,
    triggereds: RefCell<Vec<Vec<String>>>,
}

impl EventsTab {
    pub(in crate::app) fn init<T: Into<String>>(title: T) -> Box<dyn Tab> {
        Box::new(EventsTab {
            title: title.into(),
            messages: RefCell::new(Vec::new()),
            triggereds: RefCell::new(Vec::new()),
        })
    }

    pub(in crate::app) fn event(&self, event: Event) {
        match event.kind() {
            EventKind::Triggered {
                character,
                trigger,
                log,
            } => {
                let mut triggereds = self.triggereds.borrow_mut();
                triggereds.insert(
                    0,
                    vec![
                        format!("{} ({})", character.name, character.server),
                        trigger.name.clone(),
                        log.message().to_string(),
                    ],
                );
                let len = triggereds.len();
                if len > 100 {
                    triggereds.drain(100..len);
                }
            }
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

    pub(crate) fn triggereds(&self) -> Vec<Vec<String>> {
        self.triggereds.borrow().iter().cloned().collect()
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
