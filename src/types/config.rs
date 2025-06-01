mod imp;

use chrono::{Datelike, Local, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(test)]
mod tests;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Config {
    pub(crate) uid: ConfigUid,
    #[serde(rename = "file-format")]
    pub(crate) _file_format: String,
    pub(crate) process: Vec<ProcessConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct ProcessConfig {
    pub(crate) id: ProcessId,
    pub(crate) command: Command,
    pub(crate) apply_on: NaiveDateTime,
    pub(crate) work_dir: Option<std::path::PathBuf>,

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

    #[serde(default)]
    pub(crate) one_shot: bool,

    #[serde(default)]
    pub(crate) depends_on: Vec<ProcessId>,

    #[serde(default)]
    pub(crate) fake: bool,
}

pub(crate) struct ConfigError(pub(crate) String);

/// process identification
/// it's a unique string to identify a process to watch
#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Debug)]
pub(crate) struct ProcessId(pub(crate) String);

/// Each file with a process cofig has to use a unique ID
#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct ConfigUid(pub(crate) String);

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(deny_unknown_fields, untagged)]
pub(crate) enum Command {
    Simple(String),
    Detailed(CommandDetail),
}

impl Command {
    /// Returns a reference to the command line string
    pub(crate) fn str(&self) -> &str {
        match self {
            Command::Simple(s) => s,
            Command::Detailed(d) => &d.line,
        }
    }

    /// Returns the command type
    pub(crate) fn cmd_type(&self) -> &CommandType {
        match self {
            Command::Simple(_) => &CommandType::Simple,
            Command::Detailed(d) => &d.cmd_type,
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct CommandDetail {
    line: String,
    #[serde(default, rename = "type")]
    cmd_type: CommandType,
}

/// Command type selection for detailed commands
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) enum CommandType {
    Simple,
    Expression,
    PidFile,
    /// podman running detached and returning the cid
    /// podman run -d
    /// With cid, we will look for real process pid on system
    PodmanCid,
}

// --- Command wrappers with optional timeout ---

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(deny_unknown_fields, untagged)]
pub(crate) enum CommandInit {
    Simple(Command),
    Detailed {
        command: Command,
        #[serde(with = "humantime_serde")]
        #[serde(default)]
        timeout: Option<std::time::Duration>,
    },
}

impl CommandInit {
    pub(crate) fn command(&self) -> &Command {
        match self {
            CommandInit::Simple(command) => command,
            CommandInit::Detailed { command, .. } => command,
        }
    }
    pub(crate) fn timeout(&self) -> std::time::Duration {
        match self {
            CommandInit::Simple(_) => std::time::Duration::from_secs(10),
            CommandInit::Detailed { timeout, .. } => {
                timeout.unwrap_or_else(|| std::time::Duration::from_secs(10))
            }
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(deny_unknown_fields, untagged)]
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

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(deny_unknown_fields, untagged)]
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

// --- Health check commands ---

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(deny_unknown_fields, untagged)]
pub(crate) enum CheckHealth {
    Command(CommandCheckHealth),
    FolderActivity(FolderActivityCheckHealth),
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(deny_unknown_fields, untagged)]
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
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct FolderActivityCheckHealth {
    pub(crate) folder: std::path::PathBuf,
    #[serde(with = "humantime_serde")]
    #[serde(default)]
    inactive_time: Option<std::time::Duration>,
}

impl FolderActivityCheckHealth {
    pub(crate) fn inactive_time(&self) -> std::time::Duration {
        match self.inactive_time {
            Some(inactive_time) => inactive_time,
            None => std::time::Duration::from_secs(300),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Schedule {
    #[serde(default)]
    pub(crate) week_days: DaySelection,
    pub(crate) start_time: NaiveTime,
    pub(crate) stop_time: NaiveTime,
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

impl Default for CommandType {
    fn default() -> Self {
        CommandType::Simple
    }
}

impl ProcessConfig {
    pub(crate) fn check_config(&self) -> Result<(), ConfigError> {
        if self.command.str().is_empty() {
            return Err(ConfigError("Command cannot be empty".to_string()));
        }
        imp::is_valid_start_stop(self)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
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
