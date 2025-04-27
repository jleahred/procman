use crate::types::running_status::{self, ProcessStatus, ProcessWatched};

impl super::OneShot {
    pub(super) fn not_actived_config(mut self) -> Self {
        for (proc_id, process) in self.processes.iter_mut() {
            match (
                proc_id,
                process.process_config.clone(),
                process.process_running.clone(),
            ) {
                (proc_id, None, Some(proc_watched)) => match proc_watched.status {
                    ProcessStatus::Stopped => {}
                    ProcessStatus::Running { pid, procrust_uid } => {
                        println!("[{}] Stopping from running", proc_id.0);

                        process.process_running = Some(ProcessWatched {
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
                    ProcessStatus::ShouldBeRunning {} => {
                        println!("[{}] Stopped from ShouldBeRunning", proc_id.0);

                        process.process_running = Some(ProcessWatched {
                            id: proc_id.clone(),
                            apply_on: proc_watched.apply_on,
                            status: running_status::ProcessStatus::Stopped,
                            applied_on: chrono::Local::now().naive_local(),
                        });
                    }
                    ProcessStatus::Stopping {
                        pid: _,
                        procrust_uid: _,
                        retries: _,
                        last_attempt: _,
                    } => {}
                },
                (_proc_id, _, _) => {}
            }
        }
        self.save()
    }
}
