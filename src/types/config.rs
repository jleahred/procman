mod imp;

use chrono::{Datelike, Local, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(test)]
mod tests;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    pub(crate) uid: ConfigUid,
    #[serde(rename = "file_format")]
    pub(crate) _file_format: String,
    pub(crate) process: Vec<ProcessConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProcessConfig {
    pub(crate) id: ProcessId,
    pub(crate) command: Command,
    pub(crate) apply_on: NaiveDateTime,
    /// Command to execute when the process is running
    /// for the process to transition to the running state, the command must complete successfully
    /// it will only be attempted once
    /// if the command fails, the stop process is initiated
    pub(crate) init: Option<CommandInit>,

    /// This command will be executed before the process is started
    /// This command will be executed only once
    /// If this command fails, the process will not be started
    /// If this command is not defined, the process will be started directly
    /// If this command is defined, the process will be started only if this command succeeds
    pub(crate) before: Option<CommandBefore>,

    /// This command will be executed to stop the process
    /// If this command does not exist, a `SIGTERM` will be sent first
    /// If retries fail, a `SIGKILL` will be sent
    pub(crate) stop: Option<CommandStop>,

    /// This command will be executed to check the status of the process
    /// If this command is defined, instead of using the pid to determine if the process is running,
    /// this command will be used
    pub(crate) health_check: Option<CheckHealth>,

    #[serde(default)]
    pub(crate) schedule: Option<Schedule>,

    #[serde(default, rename = "type")]
    pub(crate) process_type: ProcessType,

    #[serde(default)]
    pub(crate) depends_on: Vec<ProcessId>,
}

pub(crate) struct ConfigError(pub(crate) String);

/// process identification
/// it's a unique string to identify a process to watch
#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Debug)]
pub(crate) struct ProcessId(pub(crate) String);

/// Each file with a process cofig has to use a unique ID
#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct ConfigUid(pub(crate) String);

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct Command(pub(crate) String);

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct CommandInit {
    pub(crate) command: Command,
    #[serde(with = "humantime_serde")]
    pub(crate) timeout: Option<std::time::Duration>,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub(crate) enum CommandBefore {
    Simple(Command),
    Detailed {
        command: Command,
        #[serde(with = "humantime_serde")]
        #[serde(default)]
        timeout: Option<std::time::Duration>,
    },
}
impl CommandBefore {
    pub(crate) fn command(&self) -> &Command {
        match self {
            CommandBefore::Simple(command) => command,
            CommandBefore::Detailed { command, .. } => command,
        }
    }
    pub(crate) fn timeout(&self) -> std::time::Duration {
        match self {
            CommandBefore::Simple(_) => std::time::Duration::from_secs(10),
            CommandBefore::Detailed { timeout, .. } => {
                timeout.unwrap_or_else(|| std::time::Duration::from_secs(10))
            }
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub(crate) enum CheckHealth {
    Command(CommandCheckHealth),
    FolderActivity(FolderActivityCheckHealth),
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub(crate) enum CommandCheckHealth {
    Simple(Command),
    Detailed {
        command: Command,
        #[serde(with = "humantime_serde")]
        #[serde(default)]
        timeout: Option<std::time::Duration>,
    },
}
impl CommandCheckHealth {
    pub(crate) fn command(&self) -> &Command {
        match self {
            CommandCheckHealth::Simple(command) => command,
            CommandCheckHealth::Detailed { command, .. } => command,
        }
    }
    pub(crate) fn timeout(&self) -> std::time::Duration {
        match self {
            CommandCheckHealth::Simple(_) => std::time::Duration::from_secs(5),
            CommandCheckHealth::Detailed { timeout, .. } => {
                timeout.unwrap_or_else(|| std::time::Duration::from_secs(5))
            }
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub(crate) enum FolderActivityCheckHealth {
    Simple(std::path::PathBuf),
    Detailed {
        folder: std::path::PathBuf,
        #[serde(with = "humantime_serde")]
        #[serde(default)]
        inactive_time: Option<std::time::Duration>,
    },
}
impl FolderActivityCheckHealth {
    pub(crate) fn folder(&self) -> &std::path::PathBuf {
        match self {
            FolderActivityCheckHealth::Simple(folder) => folder,
            FolderActivityCheckHealth::Detailed { folder, .. } => folder,
        }
    }
    pub(crate) fn inactive_time(&self) -> std::time::Duration {
        match self {
            FolderActivityCheckHealth::Simple(_) => std::time::Duration::from_secs(300),
            FolderActivityCheckHealth::Detailed { inactive_time, .. } => {
                inactive_time.unwrap_or_else(|| std::time::Duration::from_secs(300))
            }
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub(crate) enum CommandStop {
    Simple(Command),
    Detailed {
        command: Command,
        #[serde(with = "humantime_serde")]
        #[serde(default)]
        timeout: Option<std::time::Duration>,
    },
}

impl CommandStop {
    pub(crate) fn command(&self) -> &Command {
        match self {
            CommandStop::Simple(command) => command,
            CommandStop::Detailed { command, .. } => command,
        }
    }
    pub(crate) fn timeout(&self) -> std::time::Duration {
        match self {
            CommandStop::Simple(_) => std::time::Duration::from_secs(5),
            CommandStop::Detailed { timeout, .. } => {
                timeout.unwrap_or_else(|| std::time::Duration::from_secs(5))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct Schedule {
    #[serde(default)]
    pub(crate) week_days: DaySelection,

    pub(crate) start_time: NaiveTime,
    pub(crate) stop_time: NaiveTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ProcessType {
    Normal,
    Fake,
    /// When the process ends, it does not restart
    #[serde(rename = "one-shot")]
    OneShot,
    /// podman running detached and returning the cid
    /// podman run -d
    /// With cid, we will look for real process pid on system
    #[serde(rename = "podman_cid")]
    PodmanCid,
}

pub(crate) struct ActiveProcessByConfig(pub(crate) HashMap<ProcessId, ProcessConfig>);

impl Config {
    pub(crate) fn check(self) -> Result<Self, ConfigError> {
        imp::check(self)
    }

    pub(crate) fn get_active_procs_by_config(&self) -> ActiveProcessByConfig {
        imp::get_active_procs_by_config(&self)
    }
}

impl Default for ProcessType {
    fn default() -> Self {
        ProcessType::Normal
    }
}

impl ProcessConfig {
    pub(crate) fn check_config(&self) -> Result<(), ConfigError> {
        if self.command.0.is_empty() {
            return Err(ConfigError("Command cannot be empty".to_string()));
        }
        imp::is_valid_start_stop(self)
    }
}

//  ----------------

#[derive(Debug, Clone, Serialize)]
pub(crate) enum DaySelection {
    Days(Vec<chrono::Weekday>),
    #[serde(rename = "mon-fri")]
    Mon2Fri,
    #[serde(rename = "all")]
    All,
}

impl Default for DaySelection {
    fn default() -> Self {
        DaySelection::All
    }
}

impl DaySelection {
    pub(crate) fn matches(&self, weekday: chrono::Weekday) -> bool {
        imp::matches(self, weekday)
    }
}

impl<'de> Deserialize<'de> for DaySelection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        imp::deserialize_day_selection(deserializer)
    }
}
