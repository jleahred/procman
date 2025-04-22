use super::super::{ProcessStatus, RunningStatus};
use crate::types::config::{ProcessConfig, ProcessId};

pub(crate) fn mark_stopping_not_active_in_cfg(
    mut rs: RunningStatus,
    conf_proc_active: &[ProcessConfig],
) -> RunningStatus {
    let active_ids: Vec<ProcessId> = conf_proc_active
        .iter()
        .map(|conf| conf.id.clone())
        .collect();

    for (id, process) in rs.processes.iter_mut() {
        match process.status {
            ProcessStatus::Stopping { .. } => {
                println!("[{}] Process is already stopping", id.0);
                continue;
            }
            ProcessStatus::ScheduledStop { pid } => {
                println!("[{}] Marking process with PID {} as stopping", id.0, pid);
                process.status = ProcessStatus::Stopping {
                    pid,
                    retries: 0,
                    last_attempt: chrono::Local::now(),
                };
            }
            ProcessStatus::Running { pid } | ProcessStatus::PendingHealthStartCheck { pid, .. }
            | ProcessStatus::PendingInitCmd { pid, .. } => {
                if !active_ids.contains(id) {
                    println!(
                        "[{}] Marking process with PID {} as stopping due to not active in config",
                        id.0, pid
                    );
                    process.status = ProcessStatus::Stopping {
                        pid,
                        retries: 0,
                        last_attempt: chrono::Local::now(),
                    };
                }
            }
            ProcessStatus::Ready2Start { .. } => {}
        }
    }

    rs
}
