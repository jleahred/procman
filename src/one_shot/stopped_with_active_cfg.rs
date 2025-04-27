use crate::types::running_status::{self, ProcessStatus, ProcessWatched};

impl super::OneShot {
    pub(super) fn stopped_with_active_cfg(mut self) -> Self {
        for (proc_id, process) in self.processes.iter_mut() {
            match (
                proc_id,
                process.process_config.clone(),
                process.process_running.clone(),
            ) {
                (proc_id, Some(_), Some(proc_watched)) => match proc_watched.status {
                    ProcessStatus::Stopped => {
                        println!("[{}] Stopped should be running", proc_id.0);

                        process.process_running = Some(ProcessWatched {
                            id: proc_id.clone(),
                            apply_on: proc_watched.apply_on,
                            status: running_status::ProcessStatus::ShouldBeRunning,
                            applied_on: chrono::Local::now().naive_local(),
                        });
                    }
                    _ => {}
                },
                (_, _, _) => {}
            }
        }
        self.save()
    }
}
