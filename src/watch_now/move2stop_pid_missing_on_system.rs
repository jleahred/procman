use crate::types::running_status::{self, ProcessStatus, ProcessWatched};

impl super::WatchNow {
    pub(super) fn move2stop_pid_missing_on_system(mut self) -> Result<Self, String> {
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
                    | ProcessStatus::TooMuchRuns => {}
                    //  ----
                    ProcessStatus::Running {
                        pid, health_check, ..
                    }
                    | ProcessStatus::Stopping {
                        pid, health_check, ..
                    }
                    | ProcessStatus::PendingInitCmd {
                        pid, health_check, ..
                    } => {
                        if health_check == None {
                            if !is_process_running(pid) {
                                proc_info.process_watched = Some(ProcessWatched {
                                    id: proc_id.clone(),
                                    apply_on: proc_watched.apply_on,
                                    status: running_status::ProcessStatus::Stopped,
                                    applied_on: chrono::Local::now().naive_local(),
                                    last_runs: proc_watched.last_runs.clone(),
                                });

                                println!(
                                    "[{}] Register Stopped process with PID {}: Not running on system",
                                    proc_id.0, pid
                                );
                            }
                        }
                    }
                    ProcessStatus::WaittingPidFile { .. }
                    | ProcessStatus::StoppingWaittingPidFile { .. } => {}
                },
                (_, _, _) => {}
            }
        }
        self.save()
    }
}

fn is_process_running(pid: u32) -> bool {
    use nix::sys::signal::kill;
    use nix::unistd::Pid;

    match kill(Pid::from_raw(pid as i32), None) {
        Ok(_) => true,
        Err(nix::Error::EPERM) => true, // No tienes permiso, pero existe
        Err(nix::Error::ESRCH) => false, // No existe
        _ => false,
    }
}
