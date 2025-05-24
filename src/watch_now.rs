mod cfg_actived_not_in_watched;
mod check_waitting_pid_file;
mod create;
mod filter_config_by_dependencies;
mod filter_config_one_shot;
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
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(crate) fn watch_now(full_config_filename: &PathBuf) -> Result<(), String> {
    println!("\n--------------------------------------------------------------------------------");
    println!("Checking... {}\n", chrono::Local::now());

    WatchNow::create(&full_config_filename)?
        .filter_config_by_dependencies()?
        .filter_config_one_shot()?
        .cfg_actived_not_in_watched()?
        .stopped_with_active_cfg()?
        .check_waitting_pid_file()?
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
pub(super) struct WatchNow {
    persist_path: PathBuf,
    pub(crate) file_uid: ConfigUid,
    pub(crate) original_file_full_path: PathBuf,
    pub(crate) _file_format: String,
    pub(crate) processes: BTreeMap<ProcessId, WatchNowProcInfo>,
}

#[derive(Clone, Debug)]
pub(crate) struct WatchNowProcInfo {
    pub(crate) process_config: Option<ProcessConfig>,
    pub(crate) process_watched: Option<ProcessWatched>,
}
