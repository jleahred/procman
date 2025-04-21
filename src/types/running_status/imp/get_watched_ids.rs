use crate::types::config::ProcessId;
use crate::types::running_status::{ProcessStatus, RunningStatus};

pub(crate) fn get_watched_ids(runnstatus: &RunningStatus) -> Vec<ProcessId> {
    runnstatus
        .processes
        .iter()
        .filter_map(|(id, process)| {
            let opid = match process.status {
                ProcessStatus::ScheduledStop { pid } => Some(pid),
                ProcessStatus::Stopping { pid, .. } => Some(pid),
                ProcessStatus::Running { pid } => Some(pid),
                ProcessStatus::PendingHealthStartCheck { pid, .. } => Some(pid),
                ProcessStatus::Ready2Start { .. } => None,
            };

            if opid.is_some() {
                Some(id.clone())
            } else {
                None
            }
        })
        .collect()
}
