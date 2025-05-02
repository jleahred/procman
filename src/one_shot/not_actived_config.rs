use crate::types::running_status::{self, ProcessStatus, ProcessWatched};

impl super::OneShot {
    pub(super) fn not_actived_config(mut self) -> Result<Self, String> {
        for (proc_id, process) in self.processes.iter_mut() {
            match (
                proc_id,
                process.process_config.clone(),
                process.process_watched.clone(),
            ) {
                (proc_id, None, Some(proc_watched)) => match proc_watched.status {
                    ProcessStatus::Running {
                        pid,
                        procman_uid,
                        stop_command,
                    } => {
                        println!("[{}] Stopping from running", proc_id.0);

                        process.process_watched = Some(ProcessWatched {
                            id: proc_id.clone(),
                            apply_on: proc_watched.apply_on,
                            status: running_status::ProcessStatus::Stopping {
                                pid,
                                procman_uid,
                                retries: 0,
                                last_attempt: chrono::Local::now().naive_local(),
                                stop_command,
                            },
                            applied_on: chrono::Local::now().naive_local(),
                        });
                    }
                    ProcessStatus::ShouldBeRunning {} => {
                        println!("[{}] Stopped from ShouldBeRunning", proc_id.0);

                        process.process_watched = Some(ProcessWatched {
                            id: proc_id.clone(),
                            apply_on: proc_watched.apply_on,
                            status: running_status::ProcessStatus::Stopped,
                            applied_on: chrono::Local::now().naive_local(),
                        });
                    }
                    ProcessStatus::PendingInitCmd {
                        pid,
                        procman_uid,
                        stop_command,
                    } => {
                        println!("[{}] Stopping from init cmd", proc_id.0);

                        process.process_watched = Some(ProcessWatched {
                            id: proc_id.clone(),
                            apply_on: proc_watched.apply_on,
                            status: running_status::ProcessStatus::Stopping {
                                pid,
                                procman_uid,
                                retries: 0,
                                last_attempt: chrono::Local::now().naive_local(),
                                stop_command,
                            },
                            applied_on: chrono::Local::now().naive_local(),
                        });
                    }
                    ProcessStatus::Stopped => {}
                    ProcessStatus::Stopping {
                        pid: _,
                        procman_uid: _,
                        retries: _,
                        last_attempt: _,
                        stop_command: _,
                    } => {}
                },
                (_proc_id, _, _) => {}
            }
        }
        self.save()
    }
}
