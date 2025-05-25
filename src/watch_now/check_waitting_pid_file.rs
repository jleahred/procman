use crate::types::running_status::{self, ProcessStatus, ProcessWatched};
use std::fs;

impl super::WatchNow {
    pub(super) fn check_waitting_pid_file(mut self) -> Result<Self, String> {
        for (proc_id, proc_info) in self.processes.iter_mut() {
            match (
                proc_id,
                proc_info.process_config.clone(),
                proc_info.process_watched.clone(),
            ) {
                (_, _, Some(proc_watched)) => match proc_watched.status {
                    ProcessStatus::Stopped
                    | ProcessStatus::ShouldBeRunning
                    | ProcessStatus::PendingBeforeCmd
                    | ProcessStatus::Stopping { .. }
                    | ProcessStatus::Running { .. }
                    | ProcessStatus::PendingInitCmd { .. }
                    | ProcessStatus::TooMuchRuns => {}
                    //  ----
                    ProcessStatus::WaittingPidFile {
                        pid_file,
                        pid: _,
                        procman_uid,
                        health_check,
                        stop_command,
                    } => {
                        if let Ok(content) = fs::read_to_string(&pid_file) {
                            fs::remove_file(&pid_file).map_err(|e| {
                                format!("Error deleting PID file {}: {}", pid_file.display(), e)
                            })?;
                            if let Ok(pid) = content.trim().parse::<u32>() {
                                proc_info.process_watched = Some(ProcessWatched {
                                    id: proc_id.clone(),
                                    apply_on: proc_watched.apply_on,
                                    status: running_status::ProcessStatus::PendingInitCmd {
                                        pid,
                                        procman_uid,
                                        health_check,
                                        stop_command,
                                    },
                                    applied_on: chrono::Local::now().naive_local(),
                                    last_runs: proc_watched.last_runs.clone(),
                                });
                            }
                        } else {
                            if proc_watched.apply_on
                                < chrono::Local::now().naive_local() - chrono::Duration::minutes(1)
                            {
                                eprintln!(
                                    "[{}] ERROR Too much time waiting for PID file: {}",
                                    proc_watched.id.0,
                                    pid_file.display()
                                );
                            }
                        }
                    }
                    ProcessStatus::StoppingWaittingPidFile {
                        pid_file,
                        pid,
                        procman_uid,
                        health_check,
                        retries,
                        last_attempt: _,
                        stop_command,
                    } => {
                        if let Ok(content) = fs::read_to_string(&pid_file) {
                            fs::remove_file(&pid_file).map_err(|e| {
                                format!("Error deleting PID file {}: {}", pid_file.display(), e)
                            })?;
                            if let Ok(pid) = content.trim().parse::<u32>() {
                                proc_info.process_watched = Some(ProcessWatched {
                                    id: proc_id.clone(),
                                    apply_on: proc_watched.apply_on,
                                    status: running_status::ProcessStatus::Stopping {
                                        pid,
                                        procman_uid,
                                        health_check,
                                        retries: 0,
                                        last_attempt: chrono::Local::now().naive_local(),
                                        stop_command,
                                    },
                                    applied_on: chrono::Local::now().naive_local(),
                                    last_runs: proc_watched.last_runs.clone(),
                                });
                            }
                        } else {
                            if proc_watched.apply_on
                                < chrono::Local::now().naive_local() - chrono::Duration::minutes(1)
                            {
                                if proc_watched.apply_on
                                    < chrono::Local::now().naive_local()
                                        - chrono::Duration::minutes(1)
                                {
                                    eprintln!(
                                        "[{}] ERROR Too much time waiting for PID file: {}",
                                        proc_watched.id.0,
                                        pid_file.display()
                                    );
                                }
                            }

                            proc_info.process_watched = Some(ProcessWatched {
                                id: proc_id.clone(),
                                apply_on: proc_watched.apply_on,
                                status: running_status::ProcessStatus::StoppingWaittingPidFile {
                                    pid_file: pid_file.clone(),
                                    pid,
                                    procman_uid,
                                    health_check,
                                    retries: retries + 1,
                                    last_attempt: chrono::Local::now().naive_local(),
                                    stop_command,
                                },
                                applied_on: chrono::Local::now().naive_local(),
                                last_runs: proc_watched.last_runs.clone(),
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
