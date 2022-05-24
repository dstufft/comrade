use std::sync::Arc;
use std::time::{Duration, Instant};

use regex::{Captures, Regex};

use crate::config::triggers::{Action as TriggerAction, Trigger};
use crate::config::Character;
use crate::errors::TriggerError;
use crate::events::{Event, EventKind};
use crate::watcher::LogEvent;

type Result<T, E = TriggerError> = core::result::Result<T, E>;

#[derive(Debug)]
enum ActionKind {
    Triggered {
        character: Arc<Character>,
        trigger: Arc<Trigger>,
        log: Arc<LogEvent>,
    },
    DisplayText {
        text: Arc<String>,
    },
    Countdown {
        text: Arc<String>,
        duration: Duration,
        ends_at: Instant,
    },
}

#[derive(Debug)]
pub(crate) struct Action {
    kind: ActionKind,
    delay_until: Option<Instant>,
    finished: bool,
}

impl Action {
    fn new(caps: &Captures, action: &TriggerAction) -> Action {
        // TODO: We could remove an allocation and memcpy here by turning some of
        //       these String into Arc<String>, and conditionally doing the expansion
        //       based on if there are expansion variables or not.. however that is
        //       more effort and it's not clear that it's worth it.
        let (kind, delay) = match action {
            TriggerAction::DisplayText { text, delay } => {
                let mut expanded = String::new();
                caps.expand(text.as_str(), &mut expanded);

                (
                    ActionKind::DisplayText {
                        text: Arc::new(expanded),
                    },
                    delay,
                )
            }
            TriggerAction::Countdown {
                text,
                duration,
                delay,
            } => {
                let mut expanded = String::new();
                caps.expand(text.as_str(), &mut expanded);

                let start_delay = delay.unwrap_or(Duration::ZERO);

                (
                    ActionKind::Countdown {
                        text: Arc::new(expanded),
                        duration: *duration,
                        ends_at: Instant::now() + *duration + start_delay,
                    },
                    delay,
                )
            }
        };

        Action {
            kind,
            delay_until: delay.map(|d| Instant::now() + d),
            finished: false,
        }
    }

    fn triggered(character: Arc<Character>, trigger: Arc<Trigger>, log: Arc<LogEvent>) -> Action {
        Action {
            kind: ActionKind::Triggered {
                character,
                trigger,
                log,
            },
            delay_until: None,
            finished: false,
        }
    }

    pub(crate) fn events(&mut self) -> Option<Vec<Event>> {
        if let Some(delay_until) = self.delay_until {
            if Instant::now() >= delay_until {
                // Once we've reached our delay_until, then we'll set it to None so
                // that any future calls skip this code block.
                self.delay_until = None;
            } else {
                return None;
            }
        }

        match &self.kind {
            ActionKind::Triggered {
                character,
                trigger,
                log,
            } => {
                self.finished = true;
                Some(vec![Event::new(EventKind::Triggered {
                    character: character.clone(),
                    trigger: trigger.clone(),
                    log: log.clone(),
                })])
            }
            ActionKind::DisplayText { text } => {
                self.finished = true;
                Some(vec![Event::new(EventKind::DisplayText(text.clone()))])
            }
            ActionKind::Countdown {
                text,
                duration,
                ends_at,
            } => {
                if Instant::now() >= *ends_at {
                    self.finished = true;
                    Some(vec![Event::new(EventKind::Countdown {
                        text: text.clone(),
                        duration: *duration,
                        remaining: Duration::ZERO,
                    })])
                } else {
                    Some(vec![Event::new(EventKind::Countdown {
                        text: text.clone(),
                        duration: *duration,
                        remaining: ends_at.duration_since(Instant::now()),
                    })])
                }
            }
        }
    }

    pub(crate) fn finished(&self) -> bool {
        self.finished
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CompiledTrigger {
    character: Arc<Character>,
    trigger: Arc<Trigger>,
    regex: Regex,
}

impl CompiledTrigger {
    pub(crate) fn new(character: &Character, trigger: &Trigger) -> Result<CompiledTrigger> {
        Ok(CompiledTrigger {
            character: Arc::new(character.clone()),
            trigger: Arc::new(trigger.clone()),
            regex: Regex::new(trigger.search_text.as_str())?,
        })
    }

    pub(crate) fn execute(&self, event: &Arc<LogEvent>) -> Option<Vec<Action>> {
        self.regex.captures(event.message()).map(|caps| {
            let mut actions: Vec<Action> = self
                .trigger
                .actions
                .iter()
                .map(|a| Action::new(&caps, a))
                .collect();
            actions.insert(
                0,
                Action::triggered(self.character.clone(), self.trigger.clone(), event.clone()),
            );
            actions
        })
    }
}
