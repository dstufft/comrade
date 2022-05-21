use std::cell::{Ref, RefCell};

use crossterm::event;
use crossterm::event::{KeyCode, KeyModifiers};
use tui_logger::{TuiWidgetEvent, TuiWidgetState};

use crate::app::{Eventable, Result, Tab};
use crate::ui;

pub(crate) struct DebugTab {
    title: String,
    state: RefCell<TuiWidgetState>,
}

impl DebugTab {
    pub(in crate::app) fn init<T: Into<String>>(title: T) -> Box<dyn Tab> {
        Box::new(DebugTab {
            title: title.into(),
            state: RefCell::new(ui::init_logger_state()),
        })
    }

    pub(crate) fn state(&self) -> Ref<TuiWidgetState> {
        self.state.borrow()
    }

    fn transition(&self, event: &TuiWidgetEvent) {
        let state = &mut *self.state.borrow_mut();
        state.transition(event);
    }
}

impl Eventable for DebugTab {
    fn on_event(&self, event: event::Event) -> Result<()> {
        if let event::Event::Key(key) = event {
            if key.modifiers == KeyModifiers::NONE {
                match key.code {
                    KeyCode::Esc => self.transition(&TuiWidgetEvent::EscapeKey),
                    KeyCode::PageUp => self.transition(&TuiWidgetEvent::PrevPageKey),
                    KeyCode::PageDown => self.transition(&TuiWidgetEvent::NextPageKey),
                    KeyCode::Up => self.transition(&TuiWidgetEvent::UpKey),
                    KeyCode::Down => self.transition(&TuiWidgetEvent::DownKey),
                    KeyCode::Left => self.transition(&TuiWidgetEvent::LeftKey),
                    KeyCode::Right => self.transition(&TuiWidgetEvent::RightKey),
                    KeyCode::Char(' ') => self.transition(&TuiWidgetEvent::SpaceKey),
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        self.transition(&TuiWidgetEvent::PlusKey)
                    }
                    KeyCode::Char('-') => self.transition(&TuiWidgetEvent::MinusKey),
                    KeyCode::Char('h') => self.transition(&TuiWidgetEvent::HideKey),
                    KeyCode::Char('f') => self.transition(&TuiWidgetEvent::FocusKey),
                    _ => {}
                }
            }
        }

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
