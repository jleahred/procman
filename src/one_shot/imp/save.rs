use super::super::OneShot;
use crate::types::config::ProcessId;
use crate::types::running_status::{ProcessWatched, RunningStatus};
use std::collections::HashMap;
use std::fs::{self, rename};
use std::io::Write;
use std::sync::Mutex;
use std::thread;

lazy_static::lazy_static! {
    static ref PREVIOUS_SAVED: Mutex<(HashMap<ProcessId, ProcessWatched>, Option<chrono::NaiveDateTime>)> = Mutex::new((HashMap::new(), None));
}

pub(crate) fn save(one_shot: OneShot) -> OneShot {
    let mut previous_saved = PREVIOUS_SAVED.lock().unwrap();

    let now = chrono::Local::now().naive_local();

    let mut running_status: RunningStatus = one_shot.get_running_status();

    if previous_saved.0 == running_status.processes {
        if let Some(last_saved_time) = previous_saved.1 {
            if (now - last_saved_time).num_seconds() <= 5 {
                // println!(
                //     "No changes detected in RunningStatus and last save was recent, skipping save."
                // );
                return one_shot;
            }
        }
    }
    previous_saved.0 = running_status.processes.clone();
    previous_saved.1 = Some(now);

    running_status.last_update = chrono::Local::now().naive_local();

    let file_path = one_shot.persist_path.clone();

    fs::create_dir_all(&file_path).expect(&format!("Failed to create directory on {}", &file_path));
    let full_path = format!("{}/{}.toml", file_path, running_status.file_uid.0);
    let full_path_tmp = format!("{}.tmp", full_path);

    let toml_content = toml::to_string(&running_status)
        .unwrap_or_else(|err| panic!("Failed to serialize RunningStatus to TOML: {}", err));

    let mut file = fs::File::create(&full_path_tmp)
        .unwrap_or_else(|err| panic!("Failed to create file {}: {}", full_path_tmp, err));

    file.write_all(toml_content.as_bytes())
        .unwrap_or_else(|err| panic!("Failed to write to file {}: {}", full_path_tmp, err));

    rename(full_path_tmp.clone(), &full_path)
        .unwrap_or_else(|err| panic!("Failed to rename to file {}: {}", full_path_tmp, err));
    // println!("RunningStatus saved to {}", full_path);

    // not in a hurry, keep calm and cooperate
    thread::sleep(std::time::Duration::from_millis(100));
    one_shot
}
