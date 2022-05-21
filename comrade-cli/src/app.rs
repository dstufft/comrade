use std::cell::{Ref, RefCell};
use std::time::{Duration, Instant};

use camino::Utf8PathBuf;
use crossterm::event;
use crossterm::event::{KeyCode, KeyModifiers};
use downcast_rs::{impl_downcast, Downcast};
use indexmap::map::IndexMap;
use tui_logger::{TuiWidgetEvent, TuiWidgetState};

use comrade::logwatch::{LogManager, RecommendedWatcher};

use crate::errors::{ApplicationError, TerminalError};
use crate::terminal::ComradeTerminal;
use crate::ui;

type Result<T, E = ApplicationError> = core::result::Result<T, E>;

pub(crate) trait Eventable {
    fn on_event(&self, event: event::Event) -> Result<()>;
}

pub(crate) trait Tab: Eventable + Downcast {
    fn id(&self) -> &str;
    fn title(&self) -> &str;
}

impl_downcast!(Tab);

pub(crate) struct DebugTab {
    title: String,
    state: RefCell<TuiWidgetState>,
}

impl DebugTab {
    fn init<T: Into<String>>(title: T) -> Box<dyn Tab> {
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

pub(crate) struct Tabs {
    tabs: IndexMap<String, Box<dyn Tab>>,
    index: usize,
}

impl Tabs {
    fn new(tabs: Vec<Box<dyn Tab>>) -> Tabs {
        Tabs {
            tabs: tabs.into_iter().map(|t| (t.id().to_string(), t)).collect(),
            index: 0,
        }
    }

    pub(crate) fn titles(&self) -> Vec<&str> {
        self.tabs.values().map(|t| t.title()).collect()
    }

    pub(crate) fn index(&self) -> usize {
        self.index
    }

    pub(crate) fn current(&self) -> &dyn Tab {
        &**self
            .tabs
            .values()
            .nth(self.index)
            .expect("no tab for index")
    }

    pub(crate) fn tab<T: Tab>(&self, id: &str) -> Option<&T> {
        self.tabs
            .get(id)
            .map(|t| &**t)
            .and_then(|b| b.as_any().downcast_ref::<T>())
    }
}

pub(crate) struct App {
    title: String,
    finished: bool,
    tabs: Tabs,
    filename: Utf8PathBuf,
    manager: LogManager<RecommendedWatcher>,
}

impl App {
    pub(crate) fn new<T: Into<String>>(title: T, filename: Utf8PathBuf) -> Result<App> {
        let manager = LogManager::new()?;
        Ok(App {
            title: title.into(),
            finished: false,
            tabs: Tabs::new(vec![DebugTab::init("Debug")]),
            filename,
            manager,
        })
    }

    pub(crate) fn run(&mut self, term: &mut ComradeTerminal, tick_rate: Duration) -> Result<()> {
        self.on_start()?;

        let mut last_tick = Instant::now();
        while !self.finished {
            term.draw(|f| ui::draw(f, self))
                .map_err(TerminalError::IOError)?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).map_err(TerminalError::IOError)? {
                self.on_event(event::read().map_err(TerminalError::IOError)?)?;
            }

            if last_tick.elapsed() >= tick_rate {
                self.on_tick();
                last_tick = Instant::now();
            }
        }

        self.on_end()?;

        Ok(())
    }
}

impl App {
    pub(crate) fn title(&self) -> &str {
        self.title.as_str()
    }

    pub(crate) fn tabs(&self) -> &Tabs {
        &self.tabs
    }
}

impl App {
    fn quit(&mut self) {
        self.finished = true;
    }

    fn on_start(&mut self) -> Result<()> {
        self.manager.add(&self.filename)?;

        Ok(())
    }

    fn on_end(&mut self) -> Result<()> {
        self.manager.remove(&self.filename)?;

        Ok(())
    }

    fn on_tick(&mut self) {}

    fn on_event(&mut self, event: event::Event) -> Result<()> {
        if let event::Event::Key(key) = event {
            match (key.modifiers, key.code) {
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => self.quit(),
                (KeyModifiers::NONE, KeyCode::Char('q')) => self.quit(),
                _ => {}
            }
        }

        // Our current tab needs to be able to respond to any events as well.
        self.tabs().current().on_event(event)?;

        Ok(())
    }
}
