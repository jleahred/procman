use crate::types::running_status::{self, ProcessWatched};

impl super::OneShot {
    pub(super) fn cfg_actived_not_in_watched(mut self) -> Self {
        for (proc_id, process) in self.processes.iter_mut() {
            match (
                proc_id,
                process.process_config.clone(),
                process.process_running.clone(),
            ) {
                (proc_id, Some(proc_cfg), None) => {
                    process.process_running = Some(ProcessWatched {
                        id: proc_id.clone(),
                        apply_on: proc_cfg.apply_on,
                        status: running_status::ProcessStatus::ShouldBeRunning {},
                        applied_on: chrono::Local::now().naive_local(),
                    });

                    println!(
                        "[{}] Process is not watched, adding to running status should be running",
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
