use std::{process::Command, thread, time};

use crate::types::{
    config::CheckHealth,
    running_status::{self, ProcessStatus, ProcessWatched},
};

impl super::WatchNow {
    pub(super) fn move2stop_check_health(mut self) -> Result<Self, String> {
        for (proc_id, proc_info) in self.processes.iter_mut() {
            match (
                proc_id,
                proc_info.process_config.clone(),
                proc_info.process_watched.clone(),
            ) {
                (_, _, Some(proc_watched)) => match proc_watched.status {
                    ProcessStatus::Stopped
                    | ProcessStatus::ShouldBeRunning
                    | ProcessStatus::PendingBeforeCmd => {}
                    //  ----
                    ProcessStatus::Stopping { health_check, .. } => {
                        if let Some(health_check) = health_check {
                            // Check if the process is running using the health_check command
                            if !is_process_running(&health_check) {
                                proc_info.process_watched = Some(ProcessWatched {
                                    id: proc_id.clone(),
                                    apply_on: proc_watched.apply_on,
                                    status: running_status::ProcessStatus::Stopped,
                                    applied_on: chrono::Local::now().naive_local(),
                                });

                                println!(
                                    "[{}] Register Stopped (in stopping state) by health_check {:?}",
                                    proc_id.0,
                                    health_check
                                );
                            }
                        }
                    }
                    ProcessStatus::Running {
                        pid,
                        procman_uid,
                        stop_command,
                        health_check,
                    }
                    | ProcessStatus::PendingInitCmd {
                        pid,
                        procman_uid,
                        stop_command,
                        health_check,
                    } => {
                        if let Some(health_check) = health_check {
                            // Check if the process is running using the health_check command
                            if !is_process_running(&health_check) {
                                proc_info.process_watched = Some(ProcessWatched {
                                    id: proc_id.clone(),
                                    apply_on: proc_watched.apply_on,
                                    status: running_status::ProcessStatus::Stopping {
                                        pid,
                                        health_check: Some(health_check.clone()),
                                        procman_uid,
                                        stop_command: stop_command.clone(),
                                        retries: 0,
                                        last_attempt: chrono::Local::now().naive_local(),
                                    },
                                    applied_on: chrono::Local::now().naive_local(),
                                });

                                println!(
                                    "[{}] Move to stopping by health_check command {:?}",
                                    proc_id.0, health_check
                                );
                            }
                        }
                    }
                },
                (_, _, _) => {}
            }
        }
        self.save()
    }
}

fn is_process_running(health_check: &CheckHealth) -> bool {
    match health_check {
        CheckHealth::Command(health_check) => {
            match run_command_with_timeout(&health_check.command().str(), health_check.timeout()) {
                Ok(()) => true,
                Err(_err) => false,
            }
        }
        CheckHealth::FolderActivity(folder_activity) => {
            let most_recent_update = get_most_recent_update_folder(&folder_activity.folder);
            let inactive_time = folder_activity.inactive_time();
            if let Some(most_recent) = most_recent_update {
                if let Ok(elapsed) = most_recent.elapsed() {
                    return elapsed < inactive_time;
                }
            }
            false
        }
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

use std::fs;

fn get_most_recent_update_folder(folder: &std::path::PathBuf) -> Option<std::time::SystemTime> {
    let mut most_recent: Option<std::time::SystemTime> = None;

    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    most_recent = match most_recent {
                        Some(current) if modified > current => Some(modified),
                        None => Some(modified),
                        _ => most_recent,
                    };
                }
            }
        }
    }

    most_recent
}
