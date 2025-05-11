use fs2::FileExt;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

pub(super) fn check(lock_file_name: &str) -> Result<File, String> {
    //  todo: /tmp/procman here???
    fs::create_dir_all("/tmp/procman/")
        .map_err(|e| format!("Failed to create directory on {}: {}", "/tmp/procman/", e))?;

    let full_path = format!("/tmp/procman/{}", lock_file_name);
    let path = Path::new(&full_path);

    // open or create the lock file
    let file = File::create(&path)
        .map_err(|e| format!("Cannot create lock file:{}  error: {}", path.display(), e))?;

    // get exclusive lock
    file.try_lock_exclusive()
        .map_err(|_| "There is another instance running".to_string())?;

    // Ok((file, path.to_path_buf())) // keep the lock until the file is dropped
    Ok(file) // keep the lock until the file is dropped
}

pub(super) fn remove_lock_file(locked: &File, lock_file_name: &PathBuf) {
    if let Err(e) = FileExt::unlock(locked) {
        eprintln!("Failed to unlock file: {}", e);
    }

    let full_path = format!("/tmp/procman/{}", lock_file_name.to_str().unwrap_or("?"));
    let path = Path::new(&full_path);

    if let Err(e) = std::fs::remove_file(&path) {
        eprintln!("Failed to remove lock file: {}", e);
    }
}
