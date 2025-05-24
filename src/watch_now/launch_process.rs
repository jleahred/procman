use crate::types::config::{self, CommandType};
use crate::types::running_status::{ProcessStatus, ProcessWatched};
use std::path::PathBuf;
use std::process::Stdio;
use std::{
    io,
    process::{Child, Command},
    thread,
    time::Duration,
};
// use wait_timeout::ChildExt;

impl super::WatchNow {
    pub(super) fn launch_process(mut self) -> Result<Self, String> {
        for (proc_id, proc_info) in self.processes.iter_mut() {
            let process_config = proc_info.process_config.clone();
            let process_watched = proc_info.process_watched.clone();
            match (process_config, process_watched) {
                (Some(process_config), Some(running)) => {
                    if process_config.fake {
                        continue;
                    }

                    match running.status {
                        ProcessStatus::ShouldBeRunning => match process_config.command.cmd_type() {
                            CommandType::Simple => {
                                println!(
                                    "[{}] Running process     apply_on: {}",
                                    proc_id.0, process_config.apply_on
                                );
                                run_process(run_process_simple, &process_config, proc_info);
                            }
                            CommandType::Expression => {
                                println!(
                                    "[{}] Running process     apply_on: {}",
                                    proc_id.0, process_config.apply_on
                                );
                                run_process(run_process_expression, &process_config, proc_info);
                            }
                            CommandType::PidFile => {
                                println!(
                                    "[{}] Running process     apply_on: {}",
                                    proc_id.0, process_config.apply_on
                                );
                                run_process(run_process_pid_file, &process_config, proc_info);
                            }
                            CommandType::PodmanCid => {
                                println!(
                                    "[{}] Running process     apply_on: {}",
                                    proc_id.0, process_config.apply_on
                                );
                                run_process(run_process_podman, &process_config, proc_info);
                            }
                        },
                        ProcessStatus::Running { .. }
                        | ProcessStatus::PendingBeforeCmd
                        | ProcessStatus::PendingInitCmd { .. }
                        | ProcessStatus::Stopping { .. }
                        | ProcessStatus::Stopped { .. }
                        | ProcessStatus::WaittingPidFile { .. }
                        | ProcessStatus::StoppingWaittingPidFile { .. } => {}
                    }
                }
                _ => {}
            }
        }
        self.save()
    }
}

fn run_process(
    runproc: fn(&config::Command, &Option<PathBuf>) -> Result<(u32, Option<PathBuf>), io::Error>,
    process_config: &config::ProcessConfig,
    proc_info: &mut WatchNowProcInfo,
) {
    match runproc(
        &process_config.command,
        &process_config.work_dir,
        // &procman_uid,
    ) {
        Ok((pid, pid_file)) => {
            let procman_uid = get_cmdline(pid);
            // TODO:2 Mejorar con un UUID único
            // let procman_uid = uuid::Uuid::new_v4().to_string();
            match procman_uid {
                Ok(procman_uid) => match pid_file {
                    Some(pid_file) => {
                        println!(
                            "[{}] Launched process  waitting pid_file: {}   apply_on: {}",
                            process_config.id.0, pid, process_config.apply_on
                        );
                        proc_info.process_watched = Some(ProcessWatched {
                            id: process_config.id.clone(),
                            apply_on: process_config.apply_on,
                            status: ProcessStatus::WaittingPidFile {
                                pid_file,
                                pid,
                                procman_uid,
                                stop_command: process_config.stop.clone(),
                                health_check: process_config.health_check.clone(),
                            },
                            applied_on: chrono::Local::now().naive_local(),
                        });
                    }
                    None => {
                        println!(
                            "[{}] Launched process  with PID: {}   apply_on: {}",
                            process_config.id.0, pid, process_config.apply_on
                        );
                        proc_info.process_watched = Some(ProcessWatched {
                            id: process_config.id.clone(),
                            apply_on: process_config.apply_on,
                            status: ProcessStatus::PendingInitCmd {
                                pid,
                                procman_uid,
                                stop_command: process_config.stop.clone(),
                                health_check: process_config.health_check.clone(),
                            },
                            applied_on: chrono::Local::now().naive_local(),
                        });
                    }
                },
                Err(error) => {
                    eprintln!(
                        "[{}] Failed to launch process (getting cmd line): {}",
                        process_config.id.0, error
                    )
                }
            }
        }
        Err(e) => {
            eprintln!("[{}] Failed to launch process: {}", process_config.id.0, e)
        }
    }
}

fn run_process_expression(
    command: &config::Command,
    work_dir: &Option<PathBuf>,
    // _proc_uuid: &str,
) -> Result<(u32, Option<PathBuf>), io::Error> {
    let mut cmd = Command::new("sh");
    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }
    let child: Child = cmd
        .arg("-c")
        .arg(&command.str())
        // .env("PROCMAN_PUID", proc_uuid)
        // .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .spawn()?;
    //  todo:2 convendría desconectar la salida de error y la salida estándar para evitar zombis?

    thread::sleep(Duration::from_secs(1));

    Ok((child.id(), None))
}

fn run_process_pid_file(
    command: &config::Command,
    work_dir: &Option<PathBuf>,
) -> Result<(u32, Option<PathBuf>), io::Error> {
    let mut cmd = Command::new("sh");
    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }

    let uid = uuid::Uuid::new_v4().to_string();
    let file = format!("/tmp/procman/{}.pid", uid);

    let child: Child = cmd
        .arg("-c")
        .arg(&command.str().replace("$$PROCMAN_PID_FILE$$", file.as_str()))
        // .env("PROCMAN_PID_FILE", "proc_uuid")
        .spawn()?;

    // let start_time = std::time::Instant::now();
    // let pid = loop {
    //     if start_time.elapsed() > Duration::from_secs(30) {
    //         return Err(io::Error::new(
    //             io::ErrorKind::TimedOut,
    //             "Timed out waiting for PID file",
    //         ));
    //     }

    //     if let Ok(content) = fs::read_to_string(&file) {
    //         fs::remove_file(&file).map_err(|e| {
    //             io::Error::new(
    //                 io::ErrorKind::Other,
    //                 format!("Failed to remove PID file: {}", e),
    //             )
    //         })?;
    //         if let Ok(pid) = content.trim().parse::<u32>() {
    //             break pid;
    //         }
    //     }

    //     thread::sleep(Duration::from_millis(100));
    // };
    thread::sleep(Duration::from_secs(1));

    Ok((child.id(), Some(PathBuf::from(file))))
}

fn run_process_simple(
    command: &config::Command,
    work_dir: &Option<PathBuf>,
    // _proc_uuid: &str,
) -> Result<(u32, Option<PathBuf>), io::Error> {
    let mut parts = command.str().split_whitespace();
    let cmd = parts
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Command string is empty"))?;
    let args: Vec<&str> = parts.collect();

    let mut cmd_builder = Command::new(cmd);
    if let Some(dir) = work_dir {
        cmd_builder.current_dir(dir);
    }

    let child: Child = cmd_builder
        .args(&args)
        // .env("PROCMAN_PUID", proc_uuid)
        .spawn()?;

    // // Wait for up to 2 seconds for the process to finish
    // let result = child.wait_timeout(Duration::from_secs(2)).map_err(|e| {
    //     io::Error::new(
    //         io::ErrorKind::Other,
    //         format!("Failed launching process: {}   error: {}", command.str(), e),
    //     )
    // })?;

    // if let Some(status) = result {
    //     dbg!(status);
    //     if !status.success() {
    //         return Err(io::Error::new(
    //             io::ErrorKind::Other,
    //             format!("Process failed with status: {:?}", status.code()),
    //         ));
    //     }
    // }
    // If the process is still running after 2 seconds, return its PID

    Ok((child.id(), None))
}
// )
// -> Result<u32, io::Error> {
//     let mut parts = command.str().split_whitespace();
//     let cmd = parts
//         .next()
//         .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Command string is empty"))?;
//     let args: Vec<&str> = parts.collect();

//     let mut cmd_builder = Command::new(cmd);
//     if let Some(dir) = work_dir {
//         cmd_builder.current_dir(dir);
//     }

//     let child: Child = cmd_builder
//         .args(&args)
//         // .env("PROCMAN_PUID", proc_uuid)
//         .spawn()?;
//     //  todo:2 convendría desconectar la salida de error y la salida estándar para evitar zombis?

//     thread::sleep(Duration::from_secs(1));

//     Ok(child.id())
// }

fn run_process_podman(
    command: &config::Command,
    work_dir: &Option<PathBuf>,
    // _proc_uuid: &str,
) -> Result<(u32, Option<PathBuf>), io::Error> {
    let mut cmd = Command::new("sh");
    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }
    let child: Child = cmd
        .arg("-c")
        .arg(&command.str())
        // .env("PROCMAN_PUID", proc_uuid)
        .stdout(Stdio::piped())
        .spawn()?;
    //  todo:2 convendría desconectar la salida de error y la salida estándar para evitar zombis?
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
        .map(|pid| (pid, None))
}

use std::{fs, str};

use super::WatchNowProcInfo;

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
