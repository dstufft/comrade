use crossterm::event;

use crate::app::{Eventable, Result, Tab};

pub(crate) struct DebugTab {
    title: String,
}

impl DebugTab {
    pub(in crate::app) fn init<T: Into<String>>(title: T) -> Box<dyn Tab> {
        Box::new(DebugTab {
            title: title.into(),
        })
    }
}

impl Eventable for DebugTab {
    fn on_event(&self, _event: event::Event) -> Result<()> {
        Ok(())
    }
}

impl Tab for DebugTab {
    fn id(&self) -> &str {
        "debug"
    }

    fn title(&self) -> &str {
        self.title.as_str()
    }
}
