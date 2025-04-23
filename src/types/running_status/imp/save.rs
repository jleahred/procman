use crate::types::config::ProcessId;
use crate::types::running_status::ProcessWatched;

use super::super::RunningStatus;
use std::collections::HashMap;
use std::fs::{self, rename};
use std::io::Write;
use std::sync::Mutex;
use std::thread;

lazy_static::lazy_static! {
    static ref LAST_SAVED_PROCESSES: Mutex<(HashMap<ProcessId, ProcessWatched>, Option<chrono::NaiveDateTime>)> = Mutex::new((HashMap::new(), None));
}

pub(crate) fn save(mut run_status: RunningStatus, file_path: &str) -> RunningStatus {
    let mut last_saved = LAST_SAVED_PROCESSES.lock().unwrap();

    let now = chrono::Local::now().naive_utc();
    if last_saved.0 == run_status.processes {
        if let Some(last_saved_time) = last_saved.1 {
            if (now - last_saved_time).num_seconds() <= 5 {
                println!(
                    "No changes detected in RunningStatus and last save was recent, skipping save."
                );
                return run_status;
            }
        }
    }
    last_saved.0 = run_status.processes.clone();
    last_saved.1 = Some(now);

    run_status.last_update = chrono::Local::now().naive_utc();

    fs::create_dir_all(&file_path).expect(&format!("Failed to create directory on {}", file_path));
    let full_path = format!("{}/{}.toml", file_path, run_status.file_uid.0);
    let full_path_tmp = format!("{}.tmp", full_path);

    let toml_content = toml::to_string(&run_status)
        .unwrap_or_else(|err| panic!("Failed to serialize RunningStatus to TOML: {}", err));

    let mut file = fs::File::create(&full_path_tmp)
        .unwrap_or_else(|err| panic!("Failed to create file {}: {}", full_path_tmp, err));

    file.write_all(toml_content.as_bytes())
        .unwrap_or_else(|err| panic!("Failed to write to file {}: {}", full_path_tmp, err));

    rename(full_path_tmp.clone(), &full_path)
        .unwrap_or_else(|err| panic!("Failed to rename to file {}: {}", full_path_tmp, err));
    println!("RunningStatus saved to {}", full_path);

    // not in a hurry, keep calm and cooperate
    thread::sleep(std::time::Duration::from_millis(100));
    run_status
}
