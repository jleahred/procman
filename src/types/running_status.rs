pub(crate) mod imp;

use crate::types::config::{ConfigUid, ProcessId};
use chrono::NaiveDateTime;
pub(crate) use imp::load_running_status;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct RunningStatus {
    // pub(crate) persist_path: String,
    pub(crate) file_uid: ConfigUid,
    #[serde(rename = "file_format")]
    pub(crate) _file_format: String,
    pub(crate) processes: HashMap<ProcessId, ProcessWatched>,
    pub(crate) last_update: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(crate) struct ProcessWatched {
    pub(crate) id: ProcessId,
    // pub(crate) procrust_uid: String,
    pub(crate) apply_on: NaiveDateTime,
    pub(crate) status: ProcessStatus,
    pub(crate) applied_on: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(crate) enum ProcessStatus {
    ShouldBeRunning,
    Running {
        pid: u32,
        procrust_uid: String,
    },
    Stopping {
        pid: u32,
        procrust_uid: String,
        retries: u32,
        last_attempt: NaiveDateTime,
    },
    Stopped, // PendingInitCmd {
             //     pid: u32,
             //     retries: u32,
             //     last_attempt: chrono::DateTime<chrono::Local>,
             // },
}
