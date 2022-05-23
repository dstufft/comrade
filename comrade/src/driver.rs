//! Trigger Driver
//!
//! Reads incoming log events, determines what triggers that log event matched
//! for, and then drives the actual event actions.
//!
//! In Comrade, an event action is implemented as an object that emits action
//! events. The intent is that something else, like a UI front end, will be
//! handling these events and present them to the user in some fashion (TTS, Text,
//! Timer Bar, etc).

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use arc_swap::Cache;
use crossbeam_channel::{bounded, select, tick, Receiver, Sender};
use log::{error, trace};

use crate::config::{CachedConfig, ConfigRef};
use crate::errors::DriverError;
use crate::events::{Event, EventReceiver, EventSender};
use crate::triggers::Action;
use crate::watcher::{LogEvent, LogReceiver};

type Result<T, E = DriverError> = core::result::Result<T, E>;

enum Commands {
    Stop,
}

#[inline(always)]
fn action_events(sender: &EventSender, action: &mut Action) {
    if let Some(events) = action.events() {
        for event in events {
            if let Err(e) = sender.send(event) {
                error!("error sending event error: {:?}", e);
            }
        }
    }
}

struct DriverThread {
    running: bool,
    config: CachedConfig,
    cmds: Receiver<Commands>,
    logs: LogReceiver,
    events: EventSender,
    actions: Vec<Action>,
    ticks: Receiver<Instant>,
}

// Note: All of the methods, other than the start method, of this
//       struct will run in a worker thread.
impl DriverThread {
    fn start(
        config: ConfigRef,
        logs: LogReceiver,
        events: EventSender,
    ) -> Result<Sender<Commands>> {
        let (s_cmds, cmds) = bounded(0);

        thread::Builder::new()
            .name("comrade driver".to_string())
            .spawn(move || {
                let worker = DriverThread {
                    running: true,
                    config: Cache::new(config),
                    cmds,
                    logs,
                    events,
                    actions: Vec::new(),
                    ticks: tick(Duration::from_millis(250)),
                };
                worker.run();
            })?;

        Ok(s_cmds)
    }

    fn run(mut self) {
        while self.running {
            select! {
                recv(self.cmds) -> msg => match msg {
                    Ok(cmd) => self.on_command(cmd),
                    Err(_) => self.on_command(Commands::Stop),
                },
                recv(self.logs) -> msg => match msg {
                    Ok(event) => self.on_log_event(event),
                    Err(e) => {
                        error!("error occured reading from log events: {:?}", e);
                    }
                },
                recv(self.ticks) -> _ => self.on_tick(),
            }
        }
    }

    fn on_command(&mut self, command: Commands) {
        match command {
            Commands::Stop => self.running = false,
        }
    }

    fn on_log_event(&mut self, matched: Arc<LogEvent>) {
        trace!("received log event: {:?}", matched);
        let config = self.config.load();

        // If we don't know this character, then it's probably been removed
        // since this event was sent.
        if let Some(_character) = config.characters.get(&*matched.id) {
            // TODO: Could we do something smart here, and modify our filter so that
            //       instead of returning a bool, it returns the matched triggers and
            //       then only try those? The biggest issue with that, is technically
            //       the configuration can change between LogEvent being generated and
            //       this method being called, so the order of the triggers could have
            //       changed. So we'd need a Vec of strings, and it might be too heavy
            //       on the allocations? Maybe examine a short string library?
            for trigger in config.triggers.compiled().values() {
                // TODO: Determine if this trigger is enabled for this character.
                if let Some(actions) = trigger.execute(&matched) {
                    for mut action in actions {
                        action_events(&self.events, &mut action);

                        if !action.finished() {
                            self.actions.push(action);
                        }
                    }
                }
            }
        }
    }

    fn on_tick(&mut self) {
        for action in self.actions.iter_mut() {
            action_events(&self.events, action);
        }
        self.actions.retain(|action| !action.finished());
    }
}

pub(crate) struct Driver {
    cmds: Sender<Commands>,
    events: EventReceiver,
}

impl Driver {
    pub(crate) fn create(config: ConfigRef, log_receiver: LogReceiver) -> Driver {
        let (s_events, events) = bounded(1000);
        let cmds = DriverThread::start(config, log_receiver, s_events)
            .expect("could not start driver thread");

        Driver { cmds, events }
    }

    pub(crate) fn event(&self) -> Option<Event> {
        self.events.try_recv().ok()
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        self.cmds
            .send(Commands::Stop)
            .expect("should not be able to disconnect before stop is sent");
    }
}
