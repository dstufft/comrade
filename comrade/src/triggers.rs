use regex::{Captures, Regex};
use std::sync::Arc;

use crate::config::triggers::{Action as TriggerAction, Trigger};
use crate::errors::TriggerError;
use crate::events::{Event, EventKind};
use crate::watcher::LogEvent;

type Result<T, E = TriggerError> = core::result::Result<T, E>;

#[derive(Debug)]
enum ActionKind {
    DisplayText { text: Arc<String> },
}

#[derive(Debug)]
pub(crate) struct Action {
    log: Arc<LogEvent>,
    kind: ActionKind,
    finished: bool,
}

impl Action {
    fn new(log: Arc<LogEvent>, caps: &Captures, action: &TriggerAction) -> Action {
        // TODO: We could remove an allocation and memcpy here by turning some of
        //       these String into Arc<String>, and conditionally doing the expansion
        //       based on if there are expansion variables or not.. however that is
        //       more effort and it's not clear that it's worth it.
        let kind = match action {
            TriggerAction::DisplayText { text } => {
                let mut expanded = String::new();
                caps.expand(text.as_str(), &mut expanded);

                ActionKind::DisplayText {
                    text: Arc::new(expanded),
                }
            }
        };

        Action {
            log,
            kind,
            finished: false,
        }
    }

    pub(crate) fn events(&mut self) -> Option<Vec<Event>> {
        match &self.kind {
            ActionKind::DisplayText { text } => {
                self.finished = true;
                Some(vec![Event::new(EventKind::DisplayText(text.clone()))])
            }
        }
    }

    pub(crate) fn finished(&self) -> bool {
        self.finished
    }
}

#[derive(Debug)]
pub(crate) struct CompiledTrigger {
    trigger: Trigger,
    regex: Regex,
}

impl CompiledTrigger {
    pub(crate) fn new(trigger: &Trigger) -> Result<CompiledTrigger> {
        Ok(CompiledTrigger {
            trigger: trigger.clone(),
            regex: Regex::new(trigger.search_text.as_str())?,
        })
    }

    pub(crate) fn execute(&self, event: &Arc<LogEvent>) -> Option<Vec<Action>> {
        self.regex.captures(event.message.as_str()).map(|caps| {
            self.trigger
                .actions
                .iter()
                .map(|a| Action::new(event.clone(), &caps, a))
                .collect()
        })
    }
}
