use crossterm::event;

use crate::app::{Eventable, Result, Tab};

pub(crate) struct ConfigTab {
    title: String,
}

impl ConfigTab {
    pub(in crate::app) fn init<T: Into<String>>(title: T) -> Box<dyn Tab> {
        Box::new(ConfigTab {
            title: title.into(),
        })
    }
}

impl Eventable for ConfigTab {
    fn on_event(&self, _event: event::Event) -> Result<()> {
        Ok(())
    }
}

impl Tab for ConfigTab {
    fn id(&self) -> &str {
        "config"
    }

    fn title(&self) -> &str {
        self.title.as_str()
    }
}
