use crate::types::running_status::ProcessStatus;
use crate::types::running_status::ProcessWatched;
use std::process::Command;
use std::thread;
use std::time;

impl super::WatchNow {
    pub(super) fn run_before_cmd(mut self) -> Result<Self, String> {
        for (proc_id, proc_info) in self.processes.iter_mut() {
            match (
                proc_info.process_config.as_ref(),
                proc_info.process_watched.as_ref(),
            ) {
                (Some(process_config), Some(running)) => match running.status.clone() {
                    ProcessStatus::PendingBeforeCmd => match &process_config.before {
                        Some(before) => {
                            println!(
                                "[{}] running before command  {}",
                                proc_id.0,
                                before.command().0
                            );
                            let timeout = before.timeout();
                            match run_command_with_timeout(&before.command().0, timeout) {
                                Ok(()) => {
                                    println!(
                                        "[{}] Before command succeeded for process",
                                        proc_id.0
                                    );
                                    proc_info.process_watched = Some(ProcessWatched {
                                        id: proc_id.clone(),
                                        apply_on: process_config.apply_on,
                                        status: ProcessStatus::ShouldBeRunning,
                                        applied_on: chrono::Local::now().naive_local(),
                                    });
                                }
                                Err(err) => {
                                    eprintln!("[{}] Before command failed.  {}", proc_id.0, err);
                                    eprintln!("[{}] Moving to stopped", proc_id.0);
                                    proc_info.process_watched = Some(ProcessWatched {
                                        id: proc_id.clone(),
                                        apply_on: process_config.apply_on,
                                        status: ProcessStatus::Stopped,
                                        applied_on: chrono::Local::now().naive_local(),
                                    });
                                }
                            }
                        }
                        None => {
                            println!("[{}] not before command", proc_id.0);
                            proc_info.process_watched = Some(ProcessWatched {
                                id: proc_id.clone(),
                                apply_on: process_config.apply_on,
                                status: ProcessStatus::ShouldBeRunning,
                                applied_on: chrono::Local::now().naive_local(),
                            });
                        }
                    },
                    //  ---------
                    ProcessStatus::ShouldBeRunning
                    | ProcessStatus::PendingInitCmd { .. }
                    | ProcessStatus::Running { .. }
                    | ProcessStatus::Stopping { .. }
                    | ProcessStatus::Stopped { .. } => {}
                },
                _ => {}
            }
        }
        self.save()
    }
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
