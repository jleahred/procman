use crate::types::running_status::{self, ProcessStatus, ProcessWatched};

impl super::WatchNow {
    pub(super) fn move2stopping_modif_applyon(mut self) -> Result<Self, String> {
        for (proc_id, proc_info) in self.processes.iter_mut() {
            match (
                proc_id,
                proc_info.process_config.clone(),
                proc_info.process_watched.clone(),
            ) {
                (_, Some(process_cfg), Some(proc_watched)) => match proc_watched.status {
                    ProcessStatus::Stopped
                    | ProcessStatus::ShouldBeRunning
                    | ProcessStatus::PendingBeforeCmd
                    | ProcessStatus::Stopping { .. }
                    | ProcessStatus::StoppingWaittingPidFile { .. }
                    | ProcessStatus::TooMuchRuns => {}
                    ProcessStatus::Running {
                        pid,
                        procman_uid,
                        stop_command,
                        health_check,
                    } => {
                        if process_cfg.apply_on != proc_watched.apply_on {
                            eprintln!(
                                "[{}] Stopping running process different apply on  {} != {}",
                                proc_id.0, process_cfg.apply_on, proc_watched.apply_on
                            );
                            proc_info.process_watched = Some(ProcessWatched {
                                id: proc_id.clone(),
                                apply_on: proc_watched.apply_on,
                                status: running_status::ProcessStatus::Stopping {
                                    pid,
                                    procman_uid,
                                    retries: 0,
                                    last_attempt: chrono::Local::now().naive_local(),
                                    stop_command,
                                    health_check,
                                },
                                applied_on: chrono::Local::now().naive_local(),
                                last_runs: proc_watched.last_runs.clone(),
                            });
                        }
                    }
                    ProcessStatus::PendingInitCmd {
                        pid,
                        procman_uid,
                        stop_command,
                        health_check,
                    } => {
                        if process_cfg.apply_on != proc_watched.apply_on {
                            eprintln!(
                                "[{}] Stopping initializing process different apply on  {} != {}",
                                proc_id.0, process_cfg.apply_on, proc_watched.apply_on
                            );
                            proc_info.process_watched = Some(ProcessWatched {
                                id: proc_id.clone(),
                                apply_on: proc_watched.apply_on,
                                status: running_status::ProcessStatus::Stopping {
                                    pid,
                                    procman_uid,
                                    retries: 0,
                                    last_attempt: chrono::Local::now().naive_local(),
                                    stop_command,
                                    health_check,
                                },
                                applied_on: chrono::Local::now().naive_local(),
                                last_runs: proc_watched.last_runs.clone(),
                            });
                        }
                    }
                    ProcessStatus::WaittingPidFile {
                        pid_file,
                        pid,
                        procman_uid,
                        stop_command,
                        health_check,
                    } => {
                        if process_cfg.apply_on != proc_watched.apply_on {
                            eprintln!(
                                "[{}] Stopping waitting-pid process different apply on  {} != {}",
                                proc_id.0, process_cfg.apply_on, proc_watched.apply_on
                            );

                            proc_info.process_watched = Some(ProcessWatched {
                                id: proc_id.clone(),
                                apply_on: proc_watched.apply_on,
                                status: running_status::ProcessStatus::StoppingWaittingPidFile {
                                    pid_file,
                                    pid,
                                    procman_uid,
                                    retries: 0,
                                    last_attempt: chrono::Local::now().naive_local(),
                                    stop_command,
                                    health_check,
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
