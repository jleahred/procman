use crate::types::config::ProcessId;
use crate::types::running_status::ProcessStatus;
use std::collections::HashSet;

impl super::WatchNow {
    pub(super) fn filter_config_by_dependencies(mut self) -> Result<Self, String> {
        let proc_id_watched = {
            let mut proc_id_watched = HashSet::<ProcessId>::new();

            for (proc_id, proc_info) in self.processes.iter_mut() {
                if let Some(ref proc_watched) = proc_info.process_watched {
                    match proc_watched.status {
                        ProcessStatus::Running { .. } => {
                            proc_id_watched.insert(proc_id.clone());
                        }
                        _ => {}
                    }
                }
            }
            proc_id_watched
        };

        for (proc_id, proc_info) in self.processes.iter_mut() {
            match (
                proc_id,
                proc_info.process_config.clone(),
                proc_info.process_watched.clone(),
            ) {
                (proc_id, Some(prc_cfg), _) => {
                    let all_depends_running = prc_cfg
                        .depends_on
                        .iter()
                        .all(|dep| proc_id_watched.contains(dep));
                    if !all_depends_running {
                        println!("[{}] missing dependency", proc_id.0);
                        proc_info.process_config = None;
                    }
                }
                (_, None, _) => {}
            }
        }
        self.save()
    }
}
