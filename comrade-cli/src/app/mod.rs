use std::time::{Duration, Instant};

use crossterm::event;
use crossterm::event::{KeyCode, KeyModifiers};
use downcast_rs::{impl_downcast, Downcast};
use indexmap::map::IndexMap;
use log::debug;

use comrade::Comrade;

pub(crate) use crate::app::tabs::{ConfigTab, DebugTab, EventsTab, LogsTab};
use crate::errors::{ApplicationError, TerminalError};
use crate::terminal::ComradeTerminal;
use crate::ui;

mod tabs;

type Result<T, E = ApplicationError> = core::result::Result<T, E>;

pub(crate) trait Eventable {
    fn on_event(&self, event: event::Event) -> Result<()>;
}

pub(crate) trait Tab: Eventable + Downcast {
    fn id(&self) -> &str;
    fn title(&self) -> &str;
}

impl_downcast!(Tab);

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

    pub(crate) fn next(&mut self) {
        self.index += 1;
        if self.index >= self.tabs.len() {
            self.index = 0;
        }
    }

    pub(crate) fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.tabs.len() - 1;
        }
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
    comrade: Comrade,
}

impl App {
    pub(crate) fn new<T: Into<String>>(title: T, comrade: Comrade) -> App {
        App {
            title: title.into(),
            finished: false,
            tabs: Tabs::new(vec![
                EventsTab::init("Events"),
                ConfigTab::init("Config"),
                LogsTab::init("Logs"),
                DebugTab::init("Debug"),
            ]),
            comrade,
        }
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
        self.comrade.init()?;
        self.comrade.start()?;

        Ok(())
    }

    fn on_end(&mut self) -> Result<()> {
        self.comrade.stop()?;

        Ok(())
    }

    fn on_tick(&mut self) {
        let tab: &EventsTab = self
            .tabs()
            .tab("events")
            .expect("could not find events tab");

        // In theory if there is a constant stream of events, then we will
        // never finish this tick, which we don't want to happen because
        // we only render the UI between ticks. Thus we'll only read so many
        // of these before we bail out and let the next set happen.
        //
        // Another possible solution to this would be to spin this off into
        // it's own thread, and have it update our data structures, and remove
        // the concept of ticks in the UI all together. However for ease of
        // implementation we're going with the tick approach for now.
        let mut processed = 0;
        while let Some(event) = self.comrade.event() {
            debug!("received event: {:?}", event);

            tab.event(event);

            processed += 1;
            if processed > 1000 {
                break;
            }
        }
    }

    fn on_event(&mut self, event: event::Event) -> Result<()> {
        if let event::Event::Key(key) = event {
            match (key.modifiers, key.code) {
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => self.quit(),
                (KeyModifiers::CONTROL, KeyCode::Char('q')) => self.quit(),
                (KeyModifiers::CONTROL, KeyCode::Right) => self.tabs.next(),
                (KeyModifiers::CONTROL, KeyCode::Left) => self.tabs.previous(),
                _ => {}
            }
        }

        // Our current tab needs to be able to respond to any events as well.
        self.tabs.current().on_event(event)?;

        Ok(())
    }
}
