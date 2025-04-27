pub(crate) mod save;

use nix::errno::Errno;

use crate::types::config::{self, ProcessId};
use crate::types::running_status::{self, ProcessStatus, ProcessWatched, RunningStatus};
use chrono::NaiveDateTime;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::process::Stdio;
use std::{
    io,
    process::{Child, Command},
    thread,
    time::Duration,
};

impl super::OneShot {
    fn save(self) -> Self {
        save::save(self)
    }

    pub(super) fn filter_config_by_dependencies(mut self) -> Self {
        let proc_id_running = {
            let mut proc_id_running = HashSet::<ProcessId>::new();

            for (proc_id, proc_info) in self.processes.iter_mut() {
                if let Some(ref proc_watched) = proc_info.process_running {
                    match proc_watched.status {
                        ProcessStatus::Running { .. } => {
                            proc_id_running.insert(proc_id.clone());
                        }
                        _ => {}
                    }
                }
            }
            proc_id_running
        };

        for (proc_id, proc_info) in self.processes.iter_mut() {
            match (
                proc_id,
                proc_info.process_config.clone(),
                proc_info.process_running.clone(),
            ) {
                (proc_id, Some(prc_cfg), _) => {
                    let all_depends_running = prc_cfg
                        .depends_on
                        .iter()
                        .all(|dep| proc_id_running.contains(dep));
                    if !all_depends_running {
                        println!("[{}] missing dependency", proc_id.0);
                        proc_info.process_config = None;
                    }
                }
                (_, None, _) => {}
            }
        }
        self.save()
    }

    pub(super) fn move2stopping_modif_applyon(mut self) -> Self {
        for (proc_id, proc_info) in self.processes.iter_mut() {
            match (
                proc_id,
                proc_info.process_config.clone(),
                proc_info.process_running.clone(),
            ) {
                (_, Some(process_cfg), Some(proc_watched)) => match proc_watched.status {
                    ProcessStatus::Stopped
                    | ProcessStatus::ShouldBeRunning
                    | ProcessStatus::Stopping { .. } => {}
                    ProcessStatus::Running {
                        pid, procrust_uid, ..
                    } => {
                        if process_cfg.apply_on != proc_watched.apply_on {
                            eprintln!(
                                "[{}] Stopping process different apply on  {} != {}",
                                proc_id.0, process_cfg.apply_on, proc_watched.apply_on
                            );
                            proc_info.process_running = Some(ProcessWatched {
                                id: proc_id.clone(),
                                apply_on: proc_watched.apply_on,
                                status: running_status::ProcessStatus::Stopping {
                                    pid,
                                    procrust_uid,
                                    retries: 0,
                                    last_attempt: chrono::Local::now().naive_local(),
                                },
                                applied_on: chrono::Local::now().naive_local(),
                            });
                        }
                    }
                },
                (_, _, _) => {}
            }
        }
        self.save()
    }

    pub(super) fn move2stop_modif_signature(mut self) -> Self {
        for (proc_id, proc_info) in self.processes.iter_mut() {
            match (
                proc_id,
                proc_info.process_config.clone(),
                proc_info.process_running.clone(),
            ) {
                (_, _, Some(proc_watched)) => match proc_watched.status {
                    ProcessStatus::Stopped | ProcessStatus::ShouldBeRunning => {}
                    ProcessStatus::Running {
                        pid, procrust_uid, ..
                    }
                    | ProcessStatus::Stopping {
                        pid, procrust_uid, ..
                    } => match get_signature(pid) {
                        Ok(signature) => {
                            if signature != procrust_uid {
                                eprintln!(
                                        "[{}] Register Stopped process different signature (not stopping process)",
                                        proc_id.0
                                    );
                                proc_info.process_running = Some(ProcessWatched {
                                    id: proc_id.clone(),
                                    apply_on: proc_watched.apply_on,
                                    status: running_status::ProcessStatus::Stopped,
                                    applied_on: chrono::Local::now().naive_local(),
                                });
                            }
                        }
                        Err(err) => {
                            eprintln!(
                                "[{}] Error getting signature for {}  {}",
                                proc_id.0, pid, err
                            );
                        }
                    },
                },
                (_, _, _) => {}
            }
        }
        self.save()
    }

    pub(super) fn move2stop_pid_missing_on_system(mut self) -> Self {
        for (proc_id, proc_info) in self.processes.iter_mut() {
            match (
                proc_id,
                proc_info.process_config.clone(),
                proc_info.process_running.clone(),
            ) {
                (_, _, Some(proc_watched)) => match proc_watched.status {
                    ProcessStatus::Stopped | ProcessStatus::ShouldBeRunning => {}
                    ProcessStatus::Running { pid, .. } | ProcessStatus::Stopping { pid, .. } => {
                        if !is_process_running(pid) {
                            proc_info.process_running = Some(ProcessWatched {
                                id: proc_id.clone(),
                                apply_on: proc_watched.apply_on,
                                status: running_status::ProcessStatus::Stopped,
                                applied_on: chrono::Local::now().naive_local(),
                            });

                            println!(
                                "[{}] Register Stopped process with PID {}: Not running on system",
                                proc_id.0, pid
                            );
                        }
                    }
                },
                (_, _, _) => {}
            }
        }
        self.save()
    }

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

    pub(super) fn not_actived_config(mut self) -> Self {
        for (proc_id, process) in self.processes.iter_mut() {
            match (
                proc_id,
                process.process_config.clone(),
                process.process_running.clone(),
            ) {
                (proc_id, None, Some(proc_watched)) => match proc_watched.status {
                    ProcessStatus::Stopped => {}
                    ProcessStatus::Running { pid, procrust_uid } => {
                        println!("[{}] Stopping from running", proc_id.0);

                        process.process_running = Some(ProcessWatched {
                            id: proc_id.clone(),
                            apply_on: proc_watched.apply_on,
                            status: running_status::ProcessStatus::Stopping {
                                pid,
                                procrust_uid,
                                retries: 0,
                                last_attempt: chrono::Local::now().naive_local(),
                            },
                            applied_on: chrono::Local::now().naive_local(),
                        });
                    }
                    ProcessStatus::ShouldBeRunning {} => {
                        println!("[{}] Stopped from ShouldBeRunning", proc_id.0);

                        process.process_running = Some(ProcessWatched {
                            id: proc_id.clone(),
                            apply_on: proc_watched.apply_on,
                            status: running_status::ProcessStatus::Stopped,
                            applied_on: chrono::Local::now().naive_local(),
                        });
                    }
                    ProcessStatus::Stopping {
                        pid: _,
                        procrust_uid: _,
                        retries: _,
                        last_attempt: _,
                    } => {}
                },
                (_proc_id, _, _) => {}
            }
        }
        self.save()
    }

    pub(super) fn stopped_with_active_cfg(mut self) -> Self {
        for (proc_id, process) in self.processes.iter_mut() {
            match (
                proc_id,
                process.process_config.clone(),
                process.process_running.clone(),
            ) {
                (proc_id, Some(_), Some(proc_watched)) => match proc_watched.status {
                    ProcessStatus::Stopped => {
                        println!("[{}] Stopped should be running", proc_id.0);

                        process.process_running = Some(ProcessWatched {
                            id: proc_id.clone(),
                            apply_on: proc_watched.apply_on,
                            status: running_status::ProcessStatus::ShouldBeRunning,
                            applied_on: chrono::Local::now().naive_local(),
                        });
                    }
                    _ => {}
                },
                (_, _, _) => {}
            }
        }
        self.save()
    }
    pub(super) fn cfg_actived_not_in_watched(mut self) -> Self {
        for (proc_id, process) in self.processes.iter_mut() {
            match (
                proc_id,
                process.process_config.clone(),
                process.process_running.clone(),
            ) {
                (proc_id, Some(proc_cfg), None) => {
                    process.process_running = Some(ProcessWatched {
                        id: proc_id.clone(),
                        apply_on: proc_cfg.apply_on,
                        status: running_status::ProcessStatus::ShouldBeRunning {},
                        applied_on: chrono::Local::now().naive_local(),
                    });

                    println!(
                        "[{}] Process is not watched, adding to running status should be running",
                        proc_id.0
                    );
                }
                (_proc_id_, _, _) => {
                    // println!("[{}]  Process is already watched", proc_id.0);
                }
            }
        }
        self.save()
    }

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

    fn get_running_status(&self) -> RunningStatus {
        let mut running_status = RunningStatus {
            file_uid: self.file_uid.clone(),
            _file_format: "0".to_string(),
            processes: HashMap::new(),
            last_update: chrono::Local::now().naive_local(),
        };

        for (process_id, proc_info) in &self.processes {
            if let Some(process_running) = &proc_info.process_running {
                running_status
                    .processes
                    .insert(process_id.clone(), process_running.clone());
            }
        }

        running_status
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

fn get_signature(pid: u32) -> std::io::Result<String> {
    let raw =
        String::from_utf8_lossy(&fs::read(format!("/proc/{}/cmdline", pid))?).replace('\0', " ");
    Ok(raw)
}
