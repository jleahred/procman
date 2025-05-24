use crate::types::running_status::{self, ProcessStatus, ProcessWatched};
use std::fs;

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
                    | ProcessStatus::PendingBeforeCmd => {}
                    ProcessStatus::Running {
                        pid, procman_uid, ..
                    }
                    | ProcessStatus::Stopping {
                        pid, procman_uid, ..
                    }
                    | ProcessStatus::PendingInitCmd {
                        pid, procman_uid, ..
                    } => match get_signature(pid) {
                        Ok(signature) => {
                            if signature != procman_uid {
                                eprintln!(
                                        "[{}] Register Stopped process different signature (not stopping process)",
                                        proc_id.0
                                    );
                                proc_info.process_watched = Some(ProcessWatched {
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
    let raw =
        String::from_utf8_lossy(&fs::read(format!("/proc/{}/cmdline", pid))?).replace('\0', " ");
    Ok(raw)
}
