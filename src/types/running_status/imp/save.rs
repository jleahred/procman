use super::super::RunningStatus;
use std::fs::{self, rename};
use std::io::Write;
use std::thread;

pub(crate) fn save(mut run_status: RunningStatus, file_path: &str) -> RunningStatus {
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
