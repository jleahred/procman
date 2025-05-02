use crate::types::config::{self, ProcessType};
use crate::types::running_status::{ProcessStatus, ProcessWatched};
use std::fs;
use std::process::Stdio;
use std::{
    io,
    process::{Child, Command},
    thread,
    time::Duration,
};

impl super::OneShot {
    pub(super) fn launch_process(mut self) -> Result<Self, String> {
        for (proc_id, proc_info) in self.processes.iter_mut() {
            match (
                proc_info.process_config.as_ref(),
                proc_info.process_watched.as_ref(),
            ) {
                (Some(process_config), Some(running)) => {
                    match running.status {
                        ProcessStatus::ShouldBeRunning {} => {
                            println!(
                                "[{}] Running process     apply_on: {}",
                                proc_id.0, process_config.apply_on
                            );

                            let runproc = match process_config.process_type {
                                ProcessType::Fake => run_process,
                                ProcessType::Normal => run_process,
                                ProcessType::PodmanCid => run_process_podman,
                            };

                            match runproc(&process_config.command) {
                                Ok(pid) => {
                                    println!(
                                        "[{}] Launched process  with PID: {}   apply_on: {}",
                                        process_config.id.0, pid, process_config.apply_on
                                    );

                                    let procman_uid = get_cmdline(pid); // TODO: Mejorar con un UUID único

                                    match procman_uid {
                                        Ok(procman_uid) => {
                                            proc_info.process_watched = Some(ProcessWatched {
                                                id: proc_id.clone(),
                                                apply_on: process_config.apply_on,
                                                status: ProcessStatus::PendingInitCmd {
                                                    pid,
                                                    procman_uid,
                                                    stop_command: process_config.stop.clone(),
                                                    health_check: process_config
                                                        .health_check
                                                        .clone(),
                                                },
                                                applied_on: chrono::Local::now().naive_local(),
                                            });
                                        }
                                        Err(error) => {
                                            eprintln!(
                                                "[{}] Failed to launch process (getting cmd line): {}",
                                                process_config.id.0, error
                                            )
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[{}] Failed to launch process: {}",
                                        process_config.id.0, e
                                    )
                                }
                            }
                        }
                        ProcessStatus::Running { .. }
                        | ProcessStatus::PendingInitCmd { .. }
                        | ProcessStatus::Stopping { .. }
                        | ProcessStatus::Stopped { .. } => {}
                    }
                }
                _ => {}
            }
        }
        self.save()
    }
}

fn run_process(command: &config::Command) -> Result<u32, io::Error> {
    let child: Child = Command::new("sh")
        .arg("-c")
        .arg(&command.0)
        // .env("PROCMAN", &command.0)
        // .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .spawn()?;
    //  todo: convendría desconectar la salida de error y la salida estándar para evitar zombis?

    thread::sleep(Duration::from_secs(1));

    Ok(child.id())
}

fn run_process_podman(command: &config::Command) -> Result<u32, io::Error> {
    let child: Child = Command::new("sh")
        .arg("-c")
        .arg(&command.0)
        .stdout(Stdio::piped())
        // .env("PROCMAN", &command.0)
        .spawn()?;
    //  todo: convendría desconectar la salida de error y la salida estándar para evitar zombis?
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Process failed with status: {:?}", output.status.code()),
        ));
    }

    println!(
        "Process output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    thread::sleep(Duration::from_secs(1));

    get_real_pid_podman_detach(String::from_utf8_lossy(&output.stdout).as_ref())
}

use std::str;

fn get_real_pid_podman_detach(container_id: &str) -> std::io::Result<u32> {
    println!(
        "podman inspect --format {} {}",
        "'{{.State.Pid}}'", container_id
    );
    let output = Command::new("podman")
        .args([
            "inspect",
            "--format",
            &format!("{}", "{{.State.Pid}}"),
            container_id.trim(),
        ])
        .output()?;

    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to inspect container",
        ));
    }

    let pid_str = str::from_utf8(&output.stdout)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
        .trim();
    let pid = pid_str.parse::<u32>().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("Invalid PID: {}", e))
    })?;

    Ok(pid)
}

fn get_cmdline(pid: u32) -> std::io::Result<String> {
    let raw =
        String::from_utf8_lossy(&fs::read(format!("/proc/{}/cmdline", pid))?).replace('\0', " ");
    Ok(raw)
}
