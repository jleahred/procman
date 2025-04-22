use procfs::process;

use super::super::{ProcessStatus, RunningStatus};
use crate::types::config::{ProcessConfig, ProcessId};
use std::collections::HashMap;

pub(crate) fn mark_stopping_if_apply_on_changed(
    mut rs: RunningStatus,
    conf_proc_active: &[ProcessConfig],
) -> RunningStatus {
    let config_map: HashMap<ProcessId, &ProcessConfig> = conf_proc_active
        .iter()
        .map(|cfg| (cfg.id.clone(), cfg))
        .collect();

    for (procid, proc_watched) in rs.processes.iter_mut() {
        match proc_watched.status {
            ProcessStatus::Stopping { .. } => {
                continue;
            }
            ProcessStatus::ScheduledStop { .. } => {
                continue;
            }
            ProcessStatus::Running { pid } | ProcessStatus::PendingHealthStartCheck { pid, .. } 
            | ProcessStatus::PendingInitCmd { pid, .. }=> {
                if let Some(proc_cfg) = config_map.get(procid) {
                    if proc_cfg.apply_on != proc_watched.apply_on {
                        println!(
                        "[{}] Marking process with PID {} as stopping due to modifcation on   apply_on  {} -> {}",
                        procid.0, pid, proc_watched.apply_on, proc_cfg.apply_on
                    );
                        proc_watched.status = ProcessStatus::Stopping {
                            pid,
                            retries: 0,
                            last_attempt: chrono::Local::now(),
                        };
                    }
                }
            }
            ProcessStatus::Ready2Start { .. } => {}
        }
    }

    rs
}
