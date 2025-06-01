use crate::types::running_status::ProcessStatus;
use chrono::TimeZone;
use chrono::{Duration, Local};

impl super::WatchNow {
    pub(super) fn delete_old_stopped(mut self) -> Result<Self, String> {
        for (proc_id, proc_info) in self.processes.iter_mut() {
            match (
                proc_id,
                proc_info.process_config.clone(),
                proc_info.process_watched.clone(),
            ) {
                (_proc_id, _, Some(process_watched)) => {
                    if ProcessStatus::Stopped == process_watched.status {
                        if Local::now().signed_duration_since(
                            Local
                                .from_local_datetime(&process_watched.applied_on)
                                .unwrap(),
                        ) > Duration::days(4)
                        {
                            println!("[{}] Deleting old stopped process", proc_id.0);
                            proc_info.process_watched = None;
                        }
                    }
                }
                _ => {}
            }
        }
        self.save()
    }
}
