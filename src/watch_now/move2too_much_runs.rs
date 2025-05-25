use crate::types::running_status::ProcessStatus;

impl super::WatchNow {
    pub(super) fn move2too_much_runs(mut self) -> Result<Self, String> {
        for (proc_id, proc_info) in self.processes.iter_mut() {
            match (
                proc_id,
                proc_info.process_config.clone(),
                proc_info.process_watched.clone(),
            ) {
                (_, _, Some(proc_watched)) => {
                    if proc_watched.status == ProcessStatus::TooMuchRuns {
                        if let Some(proc_watched) = proc_info.process_watched.as_mut() {
                            proc_watched.last_runs.retain(|&timestamp| {
                                timestamp
                                    .signed_duration_since(chrono::Local::now().naive_local())
                                    .num_minutes()
                                    .abs()
                                    <= 10 //  10 minuts
                            });

                            if proc_watched.last_runs.len() < 5 {
                                println!(
                                    "[{}] has too few runs, moving back from TooMuchRuns: {:?}",
                                    proc_id.0, proc_watched
                                );
                                proc_watched.status = ProcessStatus::Stopped;
                            }
                        }
                    }
                }
                (_, _, _) => {}
            }
        }
        self.save()
    }
}
