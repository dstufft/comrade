use crossterm::event;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use comrade::events::{Event, EventKind};

use crate::app::{Eventable, Result, Tab};

pub(crate) struct Timer {
    pub(crate) text: Arc<String>,
    pub(crate) duration: Duration,
    pub(crate) remaining: Duration,
}

impl Timer {
    pub(crate) fn percent(&self) -> u16 {
        let percent_f = (self.remaining.as_secs() as f64 / self.duration.as_secs() as f64) * 100.0;
        let percent: u16 = percent_f as u16;
        percent
    }
}

pub(crate) struct EventsTab {
    title: String,
    messages: RefCell<Vec<Arc<String>>>,
    triggereds: RefCell<Vec<Vec<String>>>,
    timers: RefCell<HashMap<String, Arc<Timer>>>,
}

impl EventsTab {
    pub(in crate::app) fn init<T: Into<String>>(title: T) -> Box<dyn Tab> {
        Box::new(EventsTab {
            title: title.into(),
            messages: RefCell::new(Vec::new()),
            triggereds: RefCell::new(Vec::new()),
            timers: RefCell::new(HashMap::new()),
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
            EventKind::Countdown {
                text,
                duration,
                remaining,
            } => {
                let mut timers = self.timers.borrow_mut();
                let timer = Arc::new(Timer {
                    text: text.clone(),
                    duration: *duration,
                    remaining: *remaining,
                });

                timers.insert(timer.text.to_string(), timer);
                timers.retain(|_k, t| !t.remaining.is_zero());
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

    pub(crate) fn timers(&self) -> Vec<Arc<Timer>> {
        self.timers.borrow().values().cloned().collect()
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
