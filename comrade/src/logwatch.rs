pub use notify::RecommendedWatcher;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::hash_map;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::errors::LogWatcherError;

type Result<T, E = LogWatcherError> = core::result::Result<T, E>;

#[derive(Debug)]
struct LogReader {
    filename: PathBuf,
    reader: Option<BufReader<File>>,
    buffer: String,
}

impl LogReader {
    fn new<P: Into<PathBuf>>(filename: P) -> Result<LogReader> {
        let mut lr = LogReader {
            filename: filename.into(),
            reader: None,
            buffer: String::new(),
        };

        lr.reopen();
        if let Some(ref mut reader) = lr.reader {
            reader.seek(SeekFrom::End(0))?;
        }

        Ok(lr)
    }

    fn process(&mut self) {
        if let Some(ref mut reader) = self.reader {
            while reader.read_line(&mut self.buffer).unwrap() > 0 {
                println!("{:?}", self.buffer.trim_end());
                self.buffer.clear();
            }
        }
    }

    fn reopen(&mut self) {
        self.reader =
            File::open(self.filename.as_path()).map_or(None, |file| Some(BufReader::new(file)));
    }
}

struct LogDispatcher {
    readers: HashMap<PathBuf, LogReader>,
}

impl LogDispatcher {
    fn new() -> LogDispatcher {
        let readers = HashMap::new();
        LogDispatcher { readers }
    }

    fn handle_event(&mut self, res: notify::Result<Event>) {
        match res {
            Ok(event) => {
                for path in &event.paths {
                    if let Some(reader) = self.readers.get_mut(path) {
                        match event.kind {
                            EventKind::Create(_) => reader.reopen(),
                            EventKind::Modify(_) => reader.process(),
                            EventKind::Remove(_) => (),
                            EventKind::Access(_) => println!("access: {:?}", event),
                            EventKind::Other => println!("other: {:?}", event),
                            EventKind::Any => println!("any: {:?}", event),
                        }
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    fn add<P: Into<PathBuf>>(&mut self, filename: P, reader: LogReader) -> Result<()> {
        match self.readers.entry(filename.into()) {
            hash_map::Entry::Vacant(e) => {
                e.insert(reader);
                Ok(())
            }
            hash_map::Entry::Occupied(e) => Err(LogWatcherError::AlreadyWatching {
                path: e.key().to_owned(),
            }),
        }
    }

    fn remove<P: AsRef<Path>>(&mut self, filename: P) -> Result<()> {
        self.readers.remove(filename.as_ref());
        Ok(())
    }
}

pub struct LogManager<W: Watcher> {
    dispatcher: Arc<Mutex<LogDispatcher>>,
    watcher: W,
}

impl LogManager<RecommendedWatcher> {
    pub fn new() -> Result<Self> {
        let dispatcher = Arc::new(Mutex::new(LogDispatcher::new()));
        let wdispatcher = dispatcher.clone();
        let watcher = notify::recommended_watcher(move |res| {
            let mut d = wdispatcher
                .lock()
                .expect("Error acquiring lock on dispatcher");
            d.handle_event(res);
        })?;
        Ok(LogManager {
            dispatcher,
            watcher,
        })
    }

    pub fn add<P: Into<PathBuf>>(&mut self, filename: P) -> Result<()> {
        let filename = filename.into();
        let reader = LogReader::new(filename.clone())?;

        self.dispatcher
            .lock()
            .expect("Error acquiring lock on dispatcher")
            .add(filename.clone(), reader)?;
        self.watcher
            .watch(filename.as_ref(), RecursiveMode::NonRecursive)?;

        Ok(())
    }

    pub fn remove<P: AsRef<Path>>(&mut self, filename: P) -> Result<()> {
        let filename = filename.as_ref();

        self.dispatcher
            .lock()
            .expect("Error acquiring lock on dispatcher")
            .remove(filename)?;
        self.watcher.unwatch(filename)?;

        Ok(())
    }
}
