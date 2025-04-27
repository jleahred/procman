use super::super::{ProcessStatus, RunningStatus};
use std::fs;
use std::io::{self};

pub(crate) fn del_if_missing_pid(mut rs: RunningStatus) -> RunningStatus {
    // Iterate over the processes in the running status
    let mut to_remove = Vec::new();

    for (id, process) in rs.processes.iter_mut() {
        let opid = match process.status {
            ProcessStatus::ScheduledStop { pid } => Some(pid),
            ProcessStatus::Stopping { pid, .. } => Some(pid),
            ProcessStatus::Running { pid } => Some(pid),
            ProcessStatus::PendingHealthStartCheck { pid, .. } => Some(pid),
            ProcessStatus::PendingInitCmd { pid, .. } => Some(pid),
            ProcessStatus::Ready2Start { .. } => None,
        };

        if let Some(pid) = opid {
            // Check if the process is running
            if !is_process_running(pid) {
                // If not running, mark the process for removal
                println!("[{}] Removing process  with PID {}: Not running", id.0, pid);
                to_remove.push(id.clone());
            } else {
                match get_cmdline(pid) {
                    Ok(cmdline) => {
                        if cmdline != process.procrust_uid {
                            println!(
                                "[{}] Removing process  with PID {}: Different command line",
                                id.0, pid
                            );
                            to_remove.push(id.clone());
                        } else {
                            // println!(
                            //     "Keeping process {} with PID {}: Still running   {} == {}",
                            //     id.0, pid, cmdline, process.procrust_uid
                            // );
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "[{}] Failed to get command line for PID {}: {}",
                            id.0, pid, e
                        );
                        to_remove.push(id.clone());
                    }
                }
            }
        }
    }

    for id in to_remove {
        rs.processes.remove(&id);
    }
    rs
}

fn is_process_running(pid: u32) -> bool {
    let path = format!("/proc/{}", pid);
    fs::metadata(path).is_ok()
}

// fn get_process_env_var(pid: u32, var_name: &str) -> Result<Option<String>, io::Error> {
//     // Build the path to the environ file
//     let environ_path = format!("/proc/{}/environ", pid);

//     // Read the content of the file
//     let content = fs::read(environ_path)?;

//     // Environment variables are separated by null characters
//     for var in content.split(|&b| b == 0) {
//         // Convert to String, ignoring invalid UTF-8 bytes
//         let Ok(env_var) = String::from_utf8_lossy(var).into_owned().parse::<String>();
//         // Look for the format NAME=VALUE
//         if let Some(pos) = env_var.find('=') {
//             let (name, value) = env_var.split_at(pos);
//             if name == var_name {
//                 // Remove the '=' character at the start of the value
//                 return Ok(Some(value[1..].to_string()));
//             }
//         }
//     }

//     // Variable not found
//     Ok(None)
// }

fn get_cmdline(pid: u32) -> std::io::Result<String> {
    let raw =
        String::from_utf8_lossy(&fs::read(format!("/proc/{}/cmdline", pid))?).replace('\0', " ");
    Ok(raw)
    // let parts = raw
    //     .split(|b| *b == 0) // argumentos separados por \0
    //     .filter(|s| !s.is_empty())
    //     .map(|s| String::from_utf8_lossy(s).into_owned())
    //     .collect();
    // Ok(parts)
}
