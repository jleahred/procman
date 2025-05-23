// use crate::read_config_file::read_config_file_or_panic;
use crate::types::config::Config;
use crate::types::running_status::load_running_status;
use once_cell::sync::Lazy;
use std::collections::BTreeMap;
use std::path::PathBuf;

use super::WatchNowProcInfo;
static RUNNING_STATUS_FOLDER: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("/tmp/procman"));

impl super::WatchNow {
    pub(crate) fn create(full_config_filename: &PathBuf) -> Result<Self, String> {
        let config: Config =
            Config::read_from_file(full_config_filename).map_err(|e| e.0.to_string())?;
        let running_status = load_running_status(&RUNNING_STATUS_FOLDER, &config.uid)?;
        let active_procs_by_config = config.get_active_procs_by_config();

        let mut result = Self {
            persist_path: RUNNING_STATUS_FOLDER.clone(),
            file_uid: config.uid,
            original_file_full_path: full_config_filename.clone(),
            _file_format: "0".to_string(),
            processes: BTreeMap::new(),
        };

        for (process_id, process_config) in active_procs_by_config.0 {
            result.processes.insert(
                process_id,
                WatchNowProcInfo {
                    process_config: Some(process_config),
                    process_watched: None,
                },
            );
        }

        for (process_id, process_watched) in running_status.processes {
            if let Some(proc_info) = result.processes.get_mut(&process_id) {
                proc_info.process_watched = Some(process_watched);
            } else {
                result.processes.insert(
                    process_id,
                    WatchNowProcInfo {
                        process_config: None,
                        process_watched: Some(process_watched),
                    },
                );
            }
        }

        Ok(result)
    }
}
