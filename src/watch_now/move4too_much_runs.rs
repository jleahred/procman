use crate::types::running_status::ProcessStatus;

impl super::WatchNow {
    pub(super) fn move4too_much_runs(mut self) -> Result<Self, String> {
        for (proc_id, proc_info) in self.processes.iter_mut() {
            match (
                proc_id,
                proc_info.process_config.clone(),
                proc_info.process_watched.clone(),
            ) {
                (_, _, Some(proc_watched)) => {
                    if proc_watched.status == ProcessStatus::Stopped
                        && proc_watched.last_runs.len() >= 5
                    {
                        eprintln!(
                            "[{}] has too many runs, moving to TooMuchRuns: {:?}",
                            proc_id.0, proc_watched
                        );
                        if let Some(proc_watched) = proc_info.process_watched.as_mut() {
                            proc_watched.status = ProcessStatus::TooMuchRuns;
                        }
                    }
                }
                (_, _, _) => {}
            }
        }
        self.save()
    }
}
