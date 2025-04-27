use crate::read_config_file::read_config_file_or_panic;
use crate::types::config::Config;
use crate::types::running_status::load_running_status;
use std::collections::HashMap;

use super::OneShotProcInfo;

const RUNNING_STATUS_FOLDER: &str = "/tmp/procman";

impl super::OneShot {
    pub(super) fn create(full_config_filename: &str) -> Self {
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
