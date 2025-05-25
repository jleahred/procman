use std::collections::VecDeque;

use crate::types::running_status::{self, ProcessWatched};

impl super::WatchNow {
    pub(super) fn cfg_actived_not_in_watched(mut self) -> Result<Self, String> {
        for (proc_id, process) in self.processes.iter_mut() {
            match (
                proc_id,
                process.process_config.clone(),
                process.process_watched.clone(),
            ) {
                (proc_id, Some(proc_cfg), None) => {
                    process.process_watched = Some(ProcessWatched {
                        id: proc_id.clone(),
                        apply_on: proc_cfg.apply_on,
                        status: running_status::ProcessStatus::PendingBeforeCmd,
                        applied_on: chrono::Local::now().naive_local(),
                        last_runs: VecDeque::new(),
                    });

                    println!(
                        "[{}] Process is not watched, status moved to should be running",
                        proc_id.0
                    );
                }
                (_proc_id_, _, _) => {
                    // println!("[{}]  Process is already watched", proc_id.0);
                }
            }
        }
        self.save()
    }
}
