mod cfg_actived_not_in_watched;
mod create;
mod filter_config_by_dependencies;
mod get_running_status;
mod launch_process;
mod move2stop_check_health;
mod move2stop_modif_signature;
mod move2stop_pid_missing_on_system;
mod move2stopping_modif_applyon;
mod not_actived_config;
mod run_before_cmd;
mod run_init_cmd;
mod save;
mod stopped_with_active_cfg;
mod try_stop;

use crate::types::config::ProcessId;
use crate::types::config::{ConfigUid, ProcessConfig};
use crate::types::running_status::ProcessWatched;
use std::collections::HashMap;

pub(crate) fn watch_now(full_config_filename: &str) -> Result<(), String> {
    println!("\n--------------------------------------------------------------------------------");
    println!("Checking... {}\n", chrono::Local::now());

    WatchNow::create(&full_config_filename)?
        .filter_config_by_dependencies()?
        .cfg_actived_not_in_watched()?
        .stopped_with_active_cfg()?
        .launch_process()?
        .not_actived_config()?
        .try_stop()?
        .move2stop_pid_missing_on_system()?
        .move2stop_check_health()?
        .move2stop_modif_signature()?
        .move2stopping_modif_applyon()?
        .run_init_cmd()?
        .run_before_cmd()?
        // .save()
        ;
    Ok(())
}

#[derive(Debug)]
pub(crate) struct WatchNow {
    persist_path: String,
    pub(crate) file_uid: ConfigUid,
    pub(crate) _file_format: String,
    pub(crate) processes: HashMap<ProcessId, WatchNowProcInfo>,
}

#[derive(Clone, Debug)]
pub(crate) struct WatchNowProcInfo {
    process_config: Option<ProcessConfig>,
    process_watched: Option<ProcessWatched>,
}
