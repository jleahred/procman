use crate::types::config::ConfigUid;
use crate::types::running_status::RunningStatus;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub(crate) fn load_running_status(
    file_path: &str,
    file_uid: &ConfigUid,
) -> Result<RunningStatus, String> {
    let full_path = format!("{}/{}.toml", file_path, file_uid.0); // Construir la ruta completa

    if Path::new(&full_path).exists() {
        let content = fs::read_to_string(&full_path)
            .map_err(|err| format!("Failed to read file {}: {}", full_path, err))?;
        toml::from_str(&content)
            .map_err(|err| format!("Failed to parse TOML from file {}: {}", full_path, err))?
    } else {
        // println!(
        //     "File {} does not exist. Returning default RunningStatus.",
        //     full_path
        // );
        Ok(RunningStatus {
            // persist_path: file_path.to_owned(),
            file_uid: file_uid.clone(),
            _file_format: String::from("0"),
            processes: HashMap::new(),
            last_update: chrono::Local::now().naive_local(),
        })
    }
}
