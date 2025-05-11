use crate::types::config::ConfigUid;
use crate::types::running_status::RunningStatus;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn load_running_status(
    file_path: &PathBuf,
    file_uid: &ConfigUid,
) -> Result<RunningStatus, String> {
    let full_path = file_path.join(format!("{}.toml", file_uid.0));

    if Path::new(&full_path).exists() {
        let content = fs::read_to_string(&full_path).map_err(|err| {
            format!(
                "Failed to read file {}: {}",
                full_path.to_str().unwrap_or("?"),
                err
            )
        })?;
        toml::from_str(&content).map_err(|err| {
            format!(
                "Failed to parse TOML from file {}: {}",
                full_path.to_str().unwrap_or("?"),
                err
            )
        })
    } else {
        // println!(
        //     "File {} does not exist. Returning default RunningStatus.",
        //     full_path
        // );
        Ok(RunningStatus {
            // persist_path: file_path.to_owned(),
            file_uid: file_uid.clone(),
            original_file_full_path: file_path.to_owned(),
            _file_format: String::from("0"),
            last_update: chrono::Local::now().naive_local(),
            processes: HashMap::new(),
        })
    }
}
