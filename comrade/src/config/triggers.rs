use std::collections::BTreeMap;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::time::Duration;

use log::{debug, error};
use regex::RegexSet;
use serde::Deserialize;
use serde_with::{serde_as, DurationSeconds};

use crate::config::Result as ConfigResult;
use crate::errors::{ConfigError, TriggerError};
use crate::triggers::CompiledTrigger;

const TRIGGER_FILENAME: &str = "Triggers.toml";

type Result<T, E = TriggerError> = core::result::Result<T, E>;

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub(crate) enum Action {
    DisplayText {
        text: String,
        #[serde_as(as = "Option<DurationSeconds<u64>>")]
        #[serde(default)]
        delay: Option<Duration>,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Trigger {
    #[serde(rename = "name")]
    pub(crate) _name: String,
    #[serde(default, rename = "comment")]
    pub(crate) _comment: String,
    pub(crate) search_text: String,
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Clone)]
pub(crate) enum TriggerSource {
    Local,
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
    pub(crate) triggers: BTreeMap<String, Trigger>,
}

#[derive(Default, Debug)]
pub(crate) struct Triggers {
    triggers: BTreeMap<TriggerSource, TriggerSet>,
    compiled: BTreeMap<(TriggerSource, String), CompiledTrigger>,
}

impl Triggers {
    pub(super) fn load(data_dir: &Path) -> ConfigResult<Triggers> {
        let mut triggers = BTreeMap::new();
        let mut compiled = BTreeMap::new();

        // Load our local triggers
        match load_triggers_from_dir(data_dir.join("local").as_path(), true)? {
            Some(trg) => {
                for (k, trigger) in trg.triggers.iter() {
                    compiled.insert(
                        (trg.meta.source.clone(), k.clone()),
                        CompiledTrigger::new(trigger)?,
                    );
                }
                triggers.insert(trg.meta.source.clone(), trg);
            }
            None => {}
        }

        // TODO: Load Remote Triggers

        Ok(Triggers { triggers, compiled })
    }

    pub(crate) fn as_filter(&self) -> Result<Box<dyn Fn(&str) -> bool + Send>> {
        let mut regexs = Vec::new();
        for ts in self.triggers.values() {
            for trigger in ts.triggers.values() {
                regexs.push(trigger.search_text.as_str());
            }
        }

        let rs = RegexSet::new(regexs)?;

        Ok(Box::new(move |line| rs.is_match(line)))
    }

    pub(crate) fn compiled(&self) -> &BTreeMap<(TriggerSource, String), CompiledTrigger> {
        &self.compiled
    }
}

fn load_triggers_from_dir(dir: &Path, allow_missing: bool) -> ConfigResult<Option<TriggerSet>> {
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
