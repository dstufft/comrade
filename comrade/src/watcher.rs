use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{Local, NaiveDateTime};
use crossbeam_channel::{bounded, Receiver, Sender};
use lazy_static::lazy_static;
use log::{debug, error, log_enabled, trace, warn};
use notify::{Event, EventHandler, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::Mutex;
use regex::Regex;

use crate::errors::LogWatcherError;

lazy_static! {
    static ref RAW_LINE_RE: Regex = Regex::new(r"^\[([^]]+)\] (.+?)\r?\n$").unwrap();
}

type Result<T, E = LogWatcherError> = core::result::Result<T, E>;

type LogSender = Sender<Arc<LogEvent>>;
pub(crate) type LogReceiver = Receiver<Arc<LogEvent>>;

#[derive(Debug)]
pub(crate) struct LogEvent {
    pub(crate) id: String,
    pub(crate) date: NaiveDateTime,
    pub(crate) message: String,
}

#[inline(always)]
fn parse_raw_line(line: &str) -> Option<(&str, &str)> {
    RAW_LINE_RE.captures(line).map(|caps| {
        (
            caps.get(1)
                .expect("regex somehow matched without mandatory date capture")
                .as_str(),
            caps.get(2)
                .expect("regex somehow matched without mandatory line capture")
                .as_str(),
        )
    })
}

struct LogHandler {
    id: String,
    filename: PathBuf,
    filename_short: String,
    reader: Option<BufReader<File>>,
    buffer: String,
    filter: Box<dyn Fn(&str) -> bool + Send>,
    sender: LogSender,
}

impl LogHandler {
    fn new<P: Into<PathBuf>>(filename: P, id: String, sender: LogSender) -> Result<LogHandler> {
        let filename = filename.into();
        let filename_short = filename
            .file_name()
            .ok_or_else(|| LogWatcherError::InvalidPath {
                path: filename.clone(),
            })?
            .to_str()
            .ok_or_else(|| LogWatcherError::InvalidPath {
                path: filename.clone(),
            })?
            .to_string();

        let mut lr = LogHandler {
            id,
            filename,
            filename_short,
            reader: None,
            buffer: String::new(),
            filter: Box::new(|_line| false),
            sender,
        };
        lr.reader = lr.open_reader();

        if let Some(ref mut reader) = lr.reader {
            reader.seek(SeekFrom::End(0))?;
            trace!("seeked to end of file: {}", lr.filename.to_string_lossy())
        }

        Ok(lr)
    }

    fn open_reader(&mut self) -> Option<BufReader<File>> {
        match File::open(self.filename.as_path()) {
            Ok(file) => {
                debug!("opened file: {}", self.filename.to_string_lossy());
                Some(BufReader::new(file))
            }
            Err(err) => {
                debug!(
                    "error opening file: {} error: {:?}",
                    self.filename.to_string_lossy(),
                    err
                );
                None
            }
        }
    }

    fn reopen_reader(&mut self) {
        self.reader = self.open_reader();
    }

    fn process_lines(&mut self) {
        if let Some(ref mut reader) = self.reader {
            let log_error = |e| {
                error!(
                    "error reading file; filename: {} error: {}",
                    self.filename_short, e,
                );
                e
            };

            while reader
                .read_line(&mut self.buffer)
                .map_err(log_error)
                .unwrap_or(0)
                > 0
            {
                if log_enabled!(target: "comrade::watcher::raw", log::Level::Trace) {
                    let line = self.buffer.trim_end();
                    trace!(
                        target: "comrade::watcher::raw",
                        "filename: {} line: {}",
                        self.filename_short,
                        line
                    );
                }

                if let Some((date_str, line)) = parse_raw_line(self.buffer.as_str()) {
                    if (self.filter)(line) {
                        trace!("matched line: {}", line);
                        let date = NaiveDateTime::parse_from_str(date_str, "%a %b %d %H:%M:%S %Y")
                            .unwrap_or_else(|e| {
                                error!("could not parse date: {} got error: {}", date_str, e);
                                Local::now().naive_local()
                            });

                        self.sender
                            .send(Arc::new(LogEvent {
                                id: self.id.clone(),
                                date,
                                message: line.to_string(),
                            }))
                            .expect("sender should not be disconnected");
                    }
                }

                self.buffer.clear();
            }
        }
    }

    fn set_filter(&mut self, filter: Box<dyn Fn(&str) -> bool + Send>) {
        self.filter = filter;
    }
}

impl EventHandler for LogHandler {
    fn handle_event(&mut self, res: notify::Result<Event>) {
        match res {
            Ok(event) => match event.kind {
                EventKind::Create(_) => self.reopen_reader(),
                EventKind::Modify(_) => self.process_lines(),
                EventKind::Remove(_) => (),
                EventKind::Access(_) => (),
                _ => {
                    warn!("unexpected event received: {:?}", event)
                }
            },
            Err(e) => {
                error!("an error occured while watching files: {:?}", e);
            }
        }
    }
}

struct LogWatcher {
    filename: PathBuf,
    handler: Arc<Mutex<LogHandler>>,
    watcher: RecommendedWatcher,
}

impl LogWatcher {
    fn new(filename: PathBuf, id: String, sender: LogSender) -> Result<LogWatcher> {
        let handler = Arc::new(Mutex::new(LogHandler::new(filename.as_path(), id, sender)?));
        let handler_ = handler.clone();
        let watcher = notify::recommended_watcher(move |res| handler_.lock().handle_event(res))?;

        Ok(LogWatcher {
            filename,
            handler,
            watcher,
        })
    }

    fn start(&mut self) -> Result<()> {
        self.watcher
            .watch(self.filename.as_path(), RecursiveMode::NonRecursive)?;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.watcher.unwatch(self.filename.as_path())?;
        Ok(())
    }

    fn set_filter(&self, filter: Box<dyn Fn(&str) -> bool + Send>) {
        self.handler.lock().set_filter(filter);
    }
}

pub(crate) struct Watchers {
    watchers: HashMap<String, LogWatcher>,
    sender: LogSender,
    receiver: LogReceiver,
}

impl Default for Watchers {
    fn default() -> Watchers {
        let (sender, receiver) = bounded(1000);

        Watchers {
            watchers: HashMap::default(),
            sender,
            receiver,
        }
    }
}

impl Watchers {
    pub(crate) fn add(&mut self, id: String, filename: PathBuf) -> Result<()> {
        self.watchers.insert(
            id.clone(),
            LogWatcher::new(filename, id, self.sender.clone())?,
        );

        Ok(())
    }

    pub(crate) fn start(&mut self) -> Result<()> {
        for watcher in self.watchers.values_mut() {
            watcher.start()?;
        }

        Ok(())
    }

    pub(crate) fn stop(&mut self) -> Result<()> {
        for watcher in self.watchers.values_mut() {
            watcher.stop()?;
        }

        Ok(())
    }

    pub(crate) fn set_filter(&self, id: &str, filter: Box<dyn Fn(&str) -> bool + Send>) {
        if let Some(watcher) = self.watchers.get(id) {
            watcher.set_filter(filter)
        }
    }

    pub(crate) fn receiver(&self) -> LogReceiver {
        self.receiver.clone()
    }
}
