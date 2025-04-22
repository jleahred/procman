use chrono::Local;
use std::thread;
use std::time::Duration;
use std::{process::Command, time};
// use std::sync::mpsc;

use super::super::{ProcessStatus, RunningStatus};

pub(crate) fn run_init_cmds(mut rs: RunningStatus) -> RunningStatus {
    for (id, process) in rs.processes.iter_mut() {
        if let ProcessStatus::PendingInitCmd {
            pid,
            init_command,
            retries,
            last_attempt,
        } = process.status.clone()
        {
            if retries > 0 && last_attempt + Duration::from_secs(20) > Local::now() {
                println!(
                    "[{}] Skipping init command. Last attempt was at {}",
                    id.0, last_attempt
                );
                continue;
            }

            match init_command {
                Some(ref cmd) => {
                    println!("[{}] running init command", id.0);
                    let timeout = cmd.timeout.unwrap_or_else(|| Duration::from_secs(120));
                    match run_command_with_timeout(&cmd.command.0, timeout) {
                        Ok(()) => {
                            println!("[{}] Init command succeeded for process", id.0);
                            process.status = ProcessStatus::Running { pid };
                        }
                        Err(err) => {
                            eprintln!(
                                "[{}] Init command failed: {}. Retries: {}",
                                id.0, err, retries
                            );
                            eprintln!("[{}] Program process restart", id.0);
                            if retries > 10 {
                                process.status = ProcessStatus::ScheduledStop { pid };
                                process.applied_on = Local::now().naive_local();
                                process.status = ProcessStatus::Stopping {
                                    pid,
                                    retries: 0,
                                    last_attempt: Local::now(),
                                };
                            } else {
                                process.status = ProcessStatus::PendingInitCmd {
                                    pid,
                                    init_command,
                                    retries: retries + 1,
                                    last_attempt: Local::now(),
                                };
                            }
                        }
                    }
                }
                None => {
                    println!(
                        "[{}] No init command provided for process",
                        id.0
                    );
                    process.status = ProcessStatus::Running { pid };
                }
            }
        }
    }
    rs
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
                thread::sleep(time::Duration::from_millis(50));
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
