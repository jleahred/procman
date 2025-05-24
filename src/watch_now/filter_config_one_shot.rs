use crate::types::running_status::ProcessStatus;

impl super::WatchNow {
    pub(super) fn filter_config_one_shot(mut self) -> Result<Self, String> {
        for (proc_id, proc_info) in self.processes.iter_mut() {
            let process_config = proc_info.process_config.clone();
            let process_watched = proc_info.process_watched.clone();
            match (process_config, process_watched) {
                (Some(process_config), Some(running)) => match running.status {
                    ProcessStatus::Stopped => {
                        if process_config.one_shot {
                            if running.applied_on.date() < chrono::Local::now().naive_local().date()
                            {
                                println!(
                                    "[{}] Prepare to run process one-shot   apply_on: {}",
                                    proc_id.0, process_config.apply_on
                                );
                            } else {
                                proc_info.process_config = None;
                            }
                        } else {
                            continue;
                        }
                    }
                    ProcessStatus::Running { .. }
                    | ProcessStatus::PendingBeforeCmd
                    | ProcessStatus::PendingInitCmd { .. }
                    | ProcessStatus::Stopping { .. }
                    | ProcessStatus::ShouldBeRunning { .. } => {}
                },

                _ => {}
            }
        }
        self.save()
    }
}
