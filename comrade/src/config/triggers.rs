use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::time::Duration;

use log::{debug, error};
use regex::RegexSet;
use serde::Deserialize;
use serde_with::{serde_as, DurationSeconds};

use crate::config::{Character, CharacterId, Result};
use crate::errors::ConfigError;
use crate::triggers::CompiledTrigger;

const TRIGGER_FILENAME: &str = "Triggers.toml";

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
pub struct TriggerRef {
    pub source: TriggerSource,
    pub id: TriggerId,
}

impl TriggerRef {
    pub(crate) fn new(source: TriggerSource, id: TriggerId) -> TriggerRef {
        TriggerRef { source, id }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DisabledTrigger {
    pub source: TriggerSource,
    pub id: TriggerId,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Action {
    DisplayText {
        text: String,
        #[serde_as(as = "Option<DurationSeconds<u64>>")]
        #[serde(default)]
        delay: Option<Duration>,
    },
    Countdown {
        text: String,
        #[serde_as(as = "DurationSeconds<u64>")]
        duration: Duration,
        #[serde_as(as = "Option<DurationSeconds<u64>>")]
        #[serde(default)]
        delay: Option<Duration>,
    },
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
#[serde(transparent)]
pub struct TriggerId(String);

#[derive(Debug, Deserialize, Clone)]
pub struct Trigger {
    pub name: String,
    #[serde(default)]
    pub comment: String,
    pub search_text: String,
    pub actions: Vec<Action>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Clone)]
pub enum TriggerSource {
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "remote")]
    Remote(String),
}

#[derive(Debug, Deserialize)]
pub(crate) struct TriggerMeta {
    pub(crate) source: TriggerSource,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TriggerSet {
    pub(crate) meta: TriggerMeta,

    #[serde(default)]
    pub(crate) triggers: BTreeMap<TriggerId, Trigger>,
}

#[derive(Default, Debug)]
pub(crate) struct Triggers {
    _triggers: BTreeMap<TriggerSource, TriggerSet>,
    compiled: HashMap<CharacterId, Vec<CompiledTrigger>>,
    filters: HashMap<CharacterId, RegexSet>,
}

impl Triggers {
    pub(super) fn load(
        data_dir: &Path,
        characters: &HashMap<CharacterId, Character>,
    ) -> Result<Triggers> {
        let mut triggers = BTreeMap::new();
        let mut compiled = HashMap::new();
        let mut filters = HashMap::new();

        // Load our local triggers
        match load_triggers_from_dir(data_dir.join("local").as_path(), true)? {
            Some(trg) => {
                for (trigger_id, trigger) in trg.triggers.iter() {
                    for (character_id, character) in characters {
                        if !character.disabled_triggers.contains_key(&TriggerRef::new(
                            trg.meta.source.clone(),
                            trigger_id.clone(),
                        )) {
                            // Precompile our Trigger
                            compiled
                                .entry(character_id.clone())
                                .or_insert_with(Vec::new)
                                .push(CompiledTrigger::new(character, trigger)?);

                            // Add this pattern to the list of patterns for this character
                            // for later compilation of our filter function.
                            filters
                                .entry(character_id.clone())
                                .or_insert_with(Vec::new)
                                .push(trigger.search_text.clone());
                        }
                    }
                }
                triggers.insert(trg.meta.source.clone(), trg);
            }
            None => {}
        }

        // TODO: Load Remote Triggers

        // Compile our filter functions
        let filters = filters
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    RegexSet::new(v).expect("error compiling after validation?"),
                )
            })
            .collect();

        Ok(Triggers {
            _triggers: triggers,
            compiled,
            filters,
        })
    }

    pub(crate) fn filter(&self, id: &CharacterId) -> Box<dyn Fn(&str) -> bool + Send> {
        match self.filters.get(id) {
            Some(re) => {
                let re = re.clone();
                Box::new(move |line| re.is_match(line))
            }
            None => Box::new(|_line| false),
        }
    }

    pub(crate) fn compiled(&self, id: &CharacterId) -> Option<&[CompiledTrigger]> {
        self.compiled.get(id).map(|v| v.as_slice())
    }
}

fn load_triggers_from_dir(dir: &Path, allow_missing: bool) -> Result<Option<TriggerSet>> {
    debug!("loading triggers from {}", dir.display());

    let path = dir.join(TRIGGER_FILENAME);
    let file = fs::OpenOptions::new().read(true).open(path.as_path());

    match file {
        Ok(mut f) => {
            let mut buffer = String::new();
            f.read_to_string(&mut buffer)?;

            Ok(toml_edit::de::from_str(buffer.as_str()).map_err(|source| {
                ConfigError::DeserializationError {
                    source,
                    filename: path.clone(),
                }
            })?)
        }
        Err(e) => {
            error!(
                "error opening triggers; filename: {} error: {:?}",
                path.display(),
                e
            );

            if allow_missing {
                Ok(None)
            } else {
                Err(e.into())
            }
        }
    }
}
