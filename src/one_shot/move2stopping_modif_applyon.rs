use crate::types::running_status::{self, ProcessStatus, ProcessWatched};

impl super::OneShot {
    pub(super) fn move2stopping_modif_applyon(mut self) -> Self {
        for (proc_id, proc_info) in self.processes.iter_mut() {
            match (
                proc_id,
                proc_info.process_config.clone(),
                proc_info.process_running.clone(),
            ) {
                (_, Some(process_cfg), Some(proc_watched)) => match proc_watched.status {
                    ProcessStatus::Stopped
                    | ProcessStatus::ShouldBeRunning
                    | ProcessStatus::Stopping { .. } => {}
                    ProcessStatus::Running {
                        pid, procrust_uid, ..
                    } => {
                        if process_cfg.apply_on != proc_watched.apply_on {
                            eprintln!(
                                "[{}] Stopping process different apply on  {} != {}",
                                proc_id.0, process_cfg.apply_on, proc_watched.apply_on
                            );
                            proc_info.process_running = Some(ProcessWatched {
                                id: proc_id.clone(),
                                apply_on: proc_watched.apply_on,
                                status: running_status::ProcessStatus::Stopping {
                                    pid,
                                    procrust_uid,
                                    retries: 0,
                                    last_attempt: chrono::Local::now().naive_local(),
                                },
                                applied_on: chrono::Local::now().naive_local(),
                            });
                        }
                    }
                },
                (_, _, _) => {}
            }
        }
        self.save()
    }
}
