use crate::types::config::{self, ProcessId};
use crate::types::running_status::{ProcessStatus, ProcessWatched};
use chrono::NaiveDateTime;
use std::fs;
use std::process::Stdio;
use std::{
    io,
    process::{Child, Command},
    thread,
    time::Duration,
};

impl super::OneShot {
    pub(super) fn launch_process(mut self) -> Self {
        for (proc_id, proc_info) in self.processes.iter_mut() {
            match (
                proc_info.process_config.as_ref(),
                proc_info.process_running.as_ref(),
            ) {
                (Some(process_config), Some(running)) => {
                    match running.status {
                        ProcessStatus::ShouldBeRunning {} => {
                            println!(
                                "[{}] Running process     apply_on: {}",
                                proc_id.0, process_config.apply_on
                            );
                            match run_process(
                                &process_config.command,
                                &process_config.id,
                                &process_config.apply_on,
                            ) {
                                Ok(pid) => {
                                    println!(
                                        "[{}] Launched process  with PID: {}   apply_on: {}",
                                        process_config.id.0, pid, process_config.apply_on
                                    );

                                    proc_info.process_running = Some(ProcessWatched {
                                        id: proc_id.clone(),
                                        apply_on: process_config.apply_on,
                                        status: ProcessStatus::Running {
                                            pid,
                                            procrust_uid: get_cmdline(pid).unwrap(), // TODO: Mejorar con un UUID único
                                        },
                                        applied_on: chrono::Local::now().naive_local(),
                                    });
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

fn run_process(
    command: &config::Command,
    _process_id: &ProcessId,
    _apply_on: &NaiveDateTime,
) -> Result<u32, io::Error> {
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

fn run_process_podman(
    command: &config::Command,
    _process_id: &ProcessId,
    _apply_on: &NaiveDateTime,
) -> Result<u32, io::Error> {
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
            format!(
                "Process failed with status: {}",
                output.status.code().unwrap_or(-1)
            ),
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

use super::OneShot;

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
