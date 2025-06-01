use crate::types::running_status::{self, ProcessStatus, ProcessWatched};

impl super::WatchNow {
    pub(super) fn move2stop_modif_signature(mut self) -> Result<Self, String> {
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
                    | ProcessStatus::WaittingPidFile { .. }
                    | ProcessStatus::StoppingWaittingPidFile { .. }
                    | ProcessStatus::TooMuchRuns
                    | ProcessStatus::Stopping { .. } => {}
                    // ----
                    ProcessStatus::Running {
                        pid, procman_uid, ..
                    }
                    | ProcessStatus::PendingInitCmd {
                        pid, procman_uid, ..
                    } => match get_signature(pid) {
                        Ok(signature) => {
                            let compatible_signature = procman_uid.contains(signature.trim());

                            // if signature != procman_uid {
                            if !compatible_signature {
                                eprintln!(
                                        "[{}] Register Stopped process different signature (not stopping process) <{}> not contained in  <{}>",
                                        proc_id.0, signature.trim(), procman_uid
                                    );
                                proc_info.process_watched = Some(ProcessWatched {
                                    id: proc_id.clone(),
                                    apply_on: proc_watched.apply_on,
                                    status: running_status::ProcessStatus::Stopped,
                                    applied_on: chrono::Local::now().naive_local(),
                                    last_runs: proc_watched.last_runs.clone(),
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
}

// fn get_signature(pid: u32) -> std::io::Result<String> {
//     let path = format!("/proc/{}/environ", pid);
//     let environ = fs::read_to_string(&path)?;
//     for entry in environ.split('\0') {
//         if let Some((key, value)) = entry.split_once('=') {
//             if key == "PROCMAN_PUID" {
//                 return Ok(value.to_string());
//             }
//         }
//     }
//     Err(std::io::Error::new(
//         std::io::ErrorKind::NotFound,
//         format!(
//             "Environment variable PROCMAN_PUID not found for process {}",
//             pid
//         ),
//     ))
// }

fn get_signature(pid: u32) -> std::io::Result<String> {
    use std::{fs, io, thread, time::Duration};

    let comm_path = format!("/proc/{}/comm", pid);
    let cmdline_path = format!("/proc/{}/cmdline", pid);

    // Leer el nombre del ejecutable desde /comm
    let executable = fs::read_to_string(&comm_path)?.trim().to_string();

    // Intentar leer cmdline hasta 3 veces
    for _ in 0..3 {
        let bytes = fs::read(&cmdline_path)?;

        if !bytes.is_empty() {
            let args = String::from_utf8_lossy(&bytes).replace('\0', " ");
            if !args.trim().is_empty() {
                // Anteponer el nombre del ejecutable
                return Ok(format!("{} {}", executable, args.trim()));
            }
        }

        thread::sleep(Duration::from_millis(20));
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("Failed to read non-empty cmdline for PID {}", pid),
    ))

    // let raw =
    //     String::from_utf8_lossy(&fs::read(format!("/proc/{}/cmdline", pid))?).replace('\0', " ");
    // Ok(raw)
}
