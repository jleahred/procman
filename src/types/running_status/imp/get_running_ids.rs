use crate::types::config::ProcessId;
use crate::types::running_status::{ProcessStatus, RunningStatus};

pub(crate) fn get_running_ids(runnstatus: &RunningStatus) -> Vec<ProcessId> {
    runnstatus
        .processes
        .iter()
        .filter_map(|(id, process)| {
            if matches!(process.status, ProcessStatus::Running { .. }) {
                Some(id.clone())
            } else {
                None
            }
        })
        .collect()
}
