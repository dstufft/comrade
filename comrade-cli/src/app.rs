use std::time::{Duration, Instant};

use camino::Utf8PathBuf;
use crossterm::event;
use crossterm::event::{KeyCode, KeyModifiers};

use comrade::logwatch::{LogManager, RecommendedWatcher};

use crate::errors::{ApplicationError, TerminalError};
use crate::terminal::ComradeTerminal;
use crate::ui;

type Result<T, E = ApplicationError> = core::result::Result<T, E>;

pub(crate) struct App {
    finished: bool,
    filename: Utf8PathBuf,
    manager: LogManager<RecommendedWatcher>,
}

impl App {
    pub(crate) fn new(filename: Utf8PathBuf) -> Result<App> {
        let manager = LogManager::new()?;
        Ok(App {
            filename,
            manager,
            finished: false,
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
                self.on_event(event::read().map_err(TerminalError::IOError)?);
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
    fn quit(&mut self) {
        self.finished = true;
    }
}

impl App {
    fn on_start(&mut self) -> Result<()> {
        self.manager.add(&self.filename)?;

        Ok(())
    }

    fn on_end(&mut self) -> Result<()> {
        self.manager.remove(&self.filename)?;

        Ok(())
    }

    fn on_tick(&mut self) {}

    fn on_event(&mut self, event: event::Event) {
        if let event::Event::Key(key) = event {
            match (key.code, key.modifiers) {
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => self.quit(),
                (KeyCode::Char('q'), KeyModifiers::NONE) => self.quit(),
                _ => {}
            }
        }
    }
}
