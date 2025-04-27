use crate::types::running_status::{self, ProcessStatus, ProcessWatched};
use nix::errno::Errno;

impl super::OneShot {
    pub(super) fn try_stop(mut self) -> Self {
        for (proc_id, process) in self.processes.iter_mut() {
            match (
                proc_id,
                process.process_config.clone(),
                process.process_running.clone(),
            ) {
                (_, _, Some(proc_watched)) => match proc_watched.status {
                    ProcessStatus::Stopped
                    | ProcessStatus::Running {
                        pid: _,
                        procrust_uid: _,
                    }
                    | ProcessStatus::ShouldBeRunning {} => {}

                    ProcessStatus::Stopping {
                        pid,
                        procrust_uid,
                        retries,
                        last_attempt: _,
                    } => {
                        let force = if retries < 5 { false } else { true };

                        match kill_process(pid, force) {
                            Ok(()) => {
                                println!(
                                    "[{}] Sent command to stop  forcekill: {}",
                                    proc_id.0, force
                                );
                            }
                            Err(err) => {
                                eprintln!(
                                    "[{}] Failed to send kill signal for pid: {}  {}",
                                    proc_id.0, pid, err
                                );
                            }
                        }

                        process.process_running = Some(ProcessWatched {
                            id: proc_id.clone(),
                            apply_on: proc_watched.apply_on,
                            status: running_status::ProcessStatus::Stopping {
                                pid,
                                procrust_uid,
                                retries: retries + 1,
                                last_attempt: chrono::Local::now().naive_local(),
                            },
                            applied_on: chrono::Local::now().naive_local(),
                        });
                    }
                },
                (_, _, _) => {}
            }
        }
        self.save()
    }
}

fn kill_process(pid: u32, force: bool) -> Result<(), Errno> {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    let signal = if force {
        Signal::SIGKILL
    } else {
        Signal::SIGTERM
    };
    kill(Pid::from_raw(pid as i32), signal)

    // if status.is_ok() {
    //     println!("Successfully sent signal {} to PID {}", signal, pid);
    // } else {
    //     eprintln!("Failed to send signal {} to PID {}", signal, pid);
    // }

    // Ok(())
}
