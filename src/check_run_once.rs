use fs2::FileExt;
use std::fs;
use std::fs::File;
use std::path::Path;

pub(crate) fn check(lock_file: &str) -> Result<File, String> {
    //  todo: /tmp/procman here???
    fs::create_dir_all("/tmp/procman/").expect(&format!(
        "Failed to create directory on {}",
        "/tmp/procman/"
    ));

    let full_path = format!("/tmp/procman/{}", lock_file);
    let path = Path::new(&full_path);

    // open or create the lock file
    let file = File::create(&path)
        .map_err(|e| format!("Cannot create lock file:{}  error: {}", path.display(), e))?;

    // get exclusive lock
    file.try_lock_exclusive()
        .map_err(|_| "There is another instance running".to_string())?;

    Ok(file) // keep the lock until the file is dropped
}
