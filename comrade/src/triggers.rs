use std::collections::BTreeMap;
use std::fs;
use std::io::prelude::*;
use std::path::Path;

use log::{debug, error};
use regex::RegexSet;
use serde::Deserialize;

use crate::errors::TriggerError;

const LOGNAME: &str = "comrade.triggers";
const TRIGGER_FILENAME: &str = "Triggers.toml";

type Result<T, E = TriggerError> = core::result::Result<T, E>;

#[derive(Debug, Deserialize)]
pub(crate) struct Trigger {
    #[serde(rename = "name")]
    pub(crate) _name: String,
    #[serde(default, rename = "comment")]
    pub(crate) _comment: String,
    pub(crate) search_text: String,
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

#[derive(Default)]
pub(crate) struct Triggers {
    triggers: BTreeMap<TriggerSource, TriggerSet>,
}

impl Triggers {
    pub(crate) fn load(data_dir: &Path) -> Result<Triggers> {
        let mut triggers = BTreeMap::new();

        // Load our local triggers
        match load_triggers_from_dir(data_dir.join("local").as_path(), true)? {
            Some(trg) => {
                triggers.insert(trg.meta.source.clone(), trg);
            }
            None => {}
        }

        // TODO: Load Remote Triggers

        Ok(Triggers { triggers })
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
}

fn load_triggers_from_dir(dir: &Path, allow_missing: bool) -> Result<Option<TriggerSet>> {
    debug!(target: LOGNAME, "loading triggers from {}", dir.display());

    let path = dir.join(TRIGGER_FILENAME);
    let file = fs::OpenOptions::new().read(true).open(path.as_path());

    match file {
        Ok(mut f) => {
            let mut buffer = String::new();
            f.read_to_string(&mut buffer)?;

            Ok(toml_edit::de::from_str(buffer.as_str()).map_err(|source| {
                TriggerError::DeserializationError {
                    source,
                    filename: path.clone(),
                }
            })?)
        }
        Err(e) => {
            error!(
                target: LOGNAME,
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
