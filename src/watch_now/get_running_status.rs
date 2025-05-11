use crate::types::running_status::RunningStatus;
use std::collections::HashMap;

impl super::WatchNow {
    pub(super) fn get_running_status(&self) -> RunningStatus {
        let mut running_status = RunningStatus {
            file_uid: self.file_uid.clone(),
            original_file_full_path: self.original_file_full_path.clone(),
            _file_format: "0".to_string(),
            processes: HashMap::new(),
            last_update: chrono::Local::now().naive_local(),
        };

        for (process_id, proc_info) in &self.processes {
            if let Some(process_watched) = &proc_info.process_watched {
                running_status
                    .processes
                    .insert(process_id.clone(), process_watched.clone());
            }
        }

        running_status
    }
}
