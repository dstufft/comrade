use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};
use std::path::PathBuf;
use std::sync::Arc;

use log::{debug, error, info, trace, warn};
use notify::{Event, EventHandler, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher};
use parking_lot::Mutex;

use crate::errors::LogWatcherError;

const LOGNAME: &str = "comrade.watcher";
const RAW_LOGNAME: &str = "comrade.watcher.raw";

struct LogHandler {
    filename: PathBuf,
    filename_short: String,
    reader: Option<BufReader<File>>,
    buffer: String,
}

impl LogHandler {
    fn new<P: Into<PathBuf>>(filename: P) -> LogHandler {
        let filename = filename.into();
        let filename_short = filename
            .file_name()
            .ok_or_else(|| LogWatcherError::InvalidPath {
                path: filename.clone(),
            })
            .unwrap()
            .to_str()
            .ok_or_else(|| LogWatcherError::InvalidPath {
                path: filename.clone(),
            })
            .unwrap()
            .to_string();

        let mut lr = LogHandler {
            filename,
            filename_short,
            reader: None,
            buffer: String::new(),
        };
        lr.reader = lr.open_reader();

        if let Some(ref mut reader) = lr.reader {
            reader.seek(SeekFrom::End(0)).unwrap();
            trace!(
                target: LOGNAME,
                "seeked to end of file: {}",
                lr.filename.to_string_lossy()
            )
        }

        lr
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
            while reader.read_line(&mut self.buffer).unwrap() > 0 {
                let line = self.buffer.trim_end();
                trace!(
                    target: RAW_LOGNAME,
                    "filename: {} line: {}",
                    self.filename_short,
                    line
                );
                self.buffer.clear();
            }
        }
    }
}

impl EventHandler for LogHandler {
    fn handle_event(&mut self, res: Result<Event>) {
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
    pub fn new<P: Into<PathBuf>>(filename: P) -> LogWatcher {
        let filename = filename.into();
        let handler = Arc::new(Mutex::new(LogHandler::new(filename.as_path())));
        let handler_ = handler.clone();
        let watcher =
            notify::recommended_watcher(move |res| handler_.lock().handle_event(res)).unwrap();

        LogWatcher {
            filename,
            handler,
            watcher,
        }
    }

    pub fn start(&mut self) {
        self.watcher
            .watch(self.filename.as_path(), RecursiveMode::NonRecursive)
            .unwrap();
    }

    pub fn stop(&mut self) {
        self.watcher.unwatch(self.filename.as_path()).unwrap();
    }
}
