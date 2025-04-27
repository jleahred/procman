mod cfg_actived_not_in_watched;
mod filter_config_by_dependencies;
mod get_running_status;
mod launch_process;
mod move2stop_modif_signature;
mod move2stop_pid_missing_on_system;
mod move2stopping_modif_applyon;
mod not_actived_config;
mod save;
mod stopped_with_active_cfg;
mod try_stop;

use crate::read_config_file::read_config_file_or_panic;
use crate::types::config::Config;
use crate::types::config::ProcessId;
use crate::types::config::{ConfigUid, ProcessConfig};
use crate::types::running_status::load_running_status;
use crate::types::running_status::ProcessWatched;
use std::collections::HashMap;

const RUNNING_STATUS_FOLDER: &str = "/tmp/procman";

pub(crate) fn one_shot(full_config_filename: &str) {
    println!("\n--------------------------------------------------------------------------------");
    println!("Checking... {}\n", chrono::Local::now());

    OneShot::create(&full_config_filename)
    .filter_config_by_dependencies()
    .cfg_actived_not_in_watched()
    .stopped_with_active_cfg()
    .launch_process()
    .not_actived_config()
    .try_stop()
    .move2stop_pid_missing_on_system()
    .move2stop_modif_signature()
    .move2stopping_modif_applyon()
    // .run_init()  todo:0
    // .save()
    ;
}

#[derive(Debug)]
pub(crate) struct OneShot {
    persist_path: String,
    pub(crate) file_uid: ConfigUid,
    pub(crate) _file_format: String,
    pub(crate) processes: HashMap<ProcessId, OneShotProcInfo>,
}

#[derive(Clone, Debug)]
pub(crate) struct OneShotProcInfo {
    process_config: Option<ProcessConfig>,
    process_running: Option<ProcessWatched>,
}

impl OneShot {
    fn create(full_config_filename: &str) -> Self {
        let config: Config = read_config_file_or_panic(full_config_filename);
        let running_status = load_running_status(RUNNING_STATUS_FOLDER, &config.uid);
        let active_procs_by_config = config.get_active_procs_by_config();

        let mut result = Self {
            persist_path: RUNNING_STATUS_FOLDER.to_string(),
            file_uid: config.uid,
            _file_format: "0".to_string(),
            processes: HashMap::new(),
        };

        for (process_id, process_config) in active_procs_by_config.0 {
            result.processes.insert(
                process_id,
                OneShotProcInfo {
                    process_config: Some(process_config),
                    process_running: None,
                },
            );
        }

        for (process_id, process_watched) in running_status.processes {
            if let Some(proc_info) = result.processes.get_mut(&process_id) {
                proc_info.process_running = Some(process_watched);
            } else {
                result.processes.insert(
                    process_id,
                    OneShotProcInfo {
                        process_config: None,
                        process_running: Some(process_watched),
                    },
                );
            }
        }

        result
    }
}
