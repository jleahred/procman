pub(crate) mod imp;

use crate::types::config::{CommandStop, ConfigUid, ProcessId};
use chrono::NaiveDateTime;
pub(crate) use imp::load_running_status;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, collections::VecDeque, path::PathBuf};

use super::config::CheckHealth;

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct RunningStatus {
    // pub(crate) persist_path: String,
    pub(crate) file_uid: ConfigUid,
    pub(crate) original_file_full_path: PathBuf,
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
    #[serde(default)]
    pub(crate) last_runs: VecDeque<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub(crate) enum ProcessStatus {
    PendingBeforeCmd,
    ShouldBeRunning,
    PendingInitCmd {
        pid: u32,
        procman_uid: String,
        health_check: Option<CheckHealth>,
        stop_command: Option<CommandStop>,
    },
    WaittingPidFile {
        pid_file: PathBuf,
        pid: u32,
        procman_uid: String,
        health_check: Option<CheckHealth>,
        stop_command: Option<CommandStop>,
    },
    Running {
        pid: u32,
        procman_uid: String,
        health_check: Option<CheckHealth>,
        stop_command: Option<CommandStop>,
    },
    Stopping {
        pid: u32,
        procman_uid: String,
        health_check: Option<CheckHealth>,
        retries: u32,
        last_attempt: NaiveDateTime,
        stop_command: Option<CommandStop>,
    },
    Stopped,

    StoppingWaittingPidFile {
        pid_file: PathBuf,
        pid: u32,
        procman_uid: String,
        health_check: Option<CheckHealth>,
        retries: u32,
        last_attempt: NaiveDateTime,
        stop_command: Option<CommandStop>,
    },
}
