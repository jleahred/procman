pub(crate) mod imp;

use crate::types::config::{CommandStop, ConfigUid, ProcessId};
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
    pub(crate) last_update: NaiveDateTime,
    pub(crate) processes: HashMap<ProcessId, ProcessWatched>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(crate) struct ProcessWatched {
    pub(crate) id: ProcessId,
    pub(crate) apply_on: NaiveDateTime,
    pub(crate) status: ProcessStatus,
    pub(crate) applied_on: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(crate) enum ProcessStatus {
    ShouldBeRunning,
    PendingInitCmd {
        pid: u32,
        procman_uid: String,
        stop_command: Option<CommandStop>,
    },
    Running {
        pid: u32,
        procman_uid: String,
        stop_command: Option<CommandStop>,
    },
    Stopping {
        pid: u32,
        procman_uid: String,
        retries: u32,
        last_attempt: NaiveDateTime,
        stop_command: Option<CommandStop>,
    },
    Stopped,
}
