use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};
use std::path::PathBuf;
use std::sync::Arc;

use lazy_static::lazy_static;
use log::{debug, error, log_enabled, trace, warn};
use notify::{Event, EventHandler, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::Mutex;
use regex::Regex;

use crate::errors::LogWatcherError;

const LOGNAME: &str = "comrade.watcher";
const RAW_LOGNAME: &str = "comrade.watcher.raw";

lazy_static! {
    static ref RAW_LINE_RE: Regex = Regex::new(r"^\[([^]]+)\] (.+?)\r?\n$").unwrap();
}

type Result<T, E = LogWatcherError> = core::result::Result<T, E>;

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
    filename: PathBuf,
    filename_short: String,
    reader: Option<BufReader<File>>,
    buffer: String,
    filter: Box<dyn Fn(&str) -> bool + Send>,
}

impl LogHandler {
    fn new<P: Into<PathBuf>>(filename: P) -> Result<LogHandler> {
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
            filename,
            filename_short,
            reader: None,
            buffer: String::new(),
            filter: Box::new(|_line| false),
        };
        lr.reader = lr.open_reader();

        if let Some(ref mut reader) = lr.reader {
            reader.seek(SeekFrom::End(0))?;
            trace!(
                target: LOGNAME,
                "seeked to end of file: {}",
                lr.filename.to_string_lossy()
            )
        }

        Ok(lr)
    }

    fn open_reader(&mut self) -> Option<BufReader<File>> {
        match File::open(self.filename.as_path()) {
            Ok(file) => {
                debug!(
                    target: LOGNAME,
                    "opened file: {}",
                    self.filename.to_string_lossy()
                );
                Some(BufReader::new(file))
            }
            Err(err) => {
                debug!(
                    target: LOGNAME,
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
            while reader.read_line(&mut self.buffer).unwrap_or(0) > 0 {
                if log_enabled!(target: RAW_LOGNAME, log::Level::Trace) {
                    let line = self.buffer.trim_end();
                    trace!(
                        target: RAW_LOGNAME,
                        "filename: {} line: {}",
                        self.filename_short,
                        line
                    );
                }

                if let Some((_date, line)) = parse_raw_line(self.buffer.as_str()) {
                    if (self.filter)(line) {
                        debug!(target: LOGNAME, "matched line: {}", line);
                        // TODO: Send back to the application to do something with
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
                    warn!(target: LOGNAME, "unexpected event received: {:?}", event)
                }
            },
            Err(e) => {
                error!(
                    target: LOGNAME,
                    "an error occured while watching files: {:?}", e
                );
            }
        }
    }
}

pub struct LogWatcher {
    filename: PathBuf,
    handler: Arc<Mutex<LogHandler>>,
    watcher: RecommendedWatcher,
}

impl LogWatcher {
    pub fn new<P: Into<PathBuf>>(filename: P) -> Result<LogWatcher> {
        let filename = filename.into();
        let handler = Arc::new(Mutex::new(LogHandler::new(filename.as_path())?));
        let handler_ = handler.clone();
        let watcher = notify::recommended_watcher(move |res| handler_.lock().handle_event(res))?;

        Ok(LogWatcher {
            filename,
            handler,
            watcher,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        self.watcher
            .watch(self.filename.as_path(), RecursiveMode::NonRecursive)?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.watcher.unwatch(self.filename.as_path())?;
        Ok(())
    }

    pub fn set_filter(&self, filter: Box<dyn Fn(&str) -> bool + Send>) {
        self.handler.lock().set_filter(filter);
    }
}
