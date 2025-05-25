use std::{process::Command, thread, time};

use crate::types::{
    config::{CommandStop, ProcessId},
    running_status::{self, ProcessStatus, ProcessWatched},
};
use nix::errno::Errno;

impl super::WatchNow {
    pub(super) fn try_stop(mut self) -> Result<Self, String> {
        for (proc_id, process) in self.processes.iter_mut() {
            match (
                proc_id,
                process.process_config.clone(),
                process.process_watched.clone(),
            ) {
                (_, _, Some(proc_watched)) => match proc_watched.status {
                    //  --------------
                    ProcessStatus::Stopping {
                        pid,
                        procman_uid,
                        retries,
                        last_attempt: _,
                        stop_command,
                        health_check,
                    } => {
                        process.process_watched = Some(ProcessWatched {
                            id: proc_id.clone(),
                            apply_on: proc_watched.apply_on,
                            status: running_status::ProcessStatus::Stopping {
                                pid,
                                procman_uid,
                                retries: retries + 1,
                                last_attempt: chrono::Local::now().naive_local(),
                                stop_command: stop_command.clone(),
                                health_check: health_check.clone(),
                            },
                            applied_on: chrono::Local::now().naive_local(),
                            last_runs: proc_watched.last_runs.clone(),
                        });

                        send_kill_or_command_stop(retries, proc_id, pid, stop_command)?;
                    }
                    //  ------
                    ProcessStatus::PendingBeforeCmd => {}
                    ProcessStatus::Stopped
                    | ProcessStatus::Running {
                        pid: _,
                        procman_uid: _,
                        stop_command: _,
                        health_check: _,
                    }
                    | ProcessStatus::WaittingPidFile {
                        pid_file: _,
                        pid: _,
                        procman_uid: _,
                        stop_command: _,
                        health_check: _,
                    }
                    | ProcessStatus::ShouldBeRunning
                    | ProcessStatus::PendingInitCmd {
                        pid: _,
                        procman_uid: _,
                        stop_command: _,
                        health_check: _,
                    }
                    | ProcessStatus::StoppingWaittingPidFile {
                        pid_file: _,
                        pid: _,
                        procman_uid: _,
                        health_check: _,
                        retries: _,
                        last_attempt: _,
                        stop_command: _,
                    }
                    | ProcessStatus::TooMuchRuns => {}
                },
                (_, _, None) => {}
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

fn run_command_with_timeout(command: &str, timeout: time::Duration) -> Result<(), String> {
    let command = command.to_string();
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
        .map_err(|e| format!("Failed to spawn command: {}", e))?;

    // let pid = child.id();

    let start = time::Instant::now();
    while start.elapsed() < timeout {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    Ok(())
                } else {
                    Err(format!("Command failed with status: {}", status))
                };
            }
            Ok(None) => {
                thread::sleep(time::Duration::from_millis(100));
                continue;
            }
            Err(e) => return Err(format!("Error checking child process: {}", e)),
        }
    }

    // Timeout: kill the process
    let _ = child.kill();
    let _ = child.wait(); // Important to avoid zombie processes
    Err("Command timed out".to_string())
}

fn send_kill_or_command_stop(
    retries: u32,
    proc_id: &ProcessId,
    pid: u32,
    stop_command: Option<CommandStop>,
) -> Result<(), String> {
    {
        let force = if retries < 5 { false } else { true };

        match (force, &stop_command) {
            (true, _) | (false, None) => match kill_process(pid, force) {
                Ok(()) => {
                    println!(
                        "[{}] Sent kill signal to stop  forcekill: {}",
                        proc_id.0, force
                    );
                }
                Err(err) => {
                    eprintln!(
                        "[{}] Failed to send kill signal for pid: {}  {}",
                        proc_id.0, pid, err
                    );
                }
            },
            (false, Some(command_stop)) => {
                let timeout = command_stop.timeout();
                if timeout.as_secs() > 0 {
                    println!(
                        "[{}] executing command stop {}",
                        proc_id.0,
                        &command_stop.command().str()
                    );
                    let result =
                        match run_command_with_timeout(&command_stop.command().str(), timeout) {
                            Ok(()) => {
                                println!("[{}] Command stop succeeded for process", proc_id.0);
                            }
                            Err(err) => {
                                eprintln!("[{}] Command stop failed.  {}", proc_id.0, err);
                            }
                        };
                    let _ = kill_process(pid, force);
                    result
                } else {
                    return Err(format!(
                        "[{}] INCORRECT timeout in configuration: {}",
                        proc_id.0,
                        timeout.as_secs()
                    ));
                }
            }
        }
    }
    Ok(())
}
