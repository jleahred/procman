use std::path::PathBuf;

pub(super) fn create(filename: Option<&PathBuf>) -> Result<(), String> {
    use std::fs::File;
    use std::io::Write;

    let binding = PathBuf::from("./processes.toml");
    let filename = filename.unwrap_or(&binding);

    let content = format!(
        r#"
# This is a simple process configuration file for procman
# It contains a single process with a simple command
uid = "{}"
file-format = "0"

[[process]]
id = "example_process"
apply-on = "2025-05-12T21:00:00"
command = "sleep 5555"

#optionals  -----------------------------------------
fake = false
depends-on = []
work-dir = "/tmp"
before = "echo 'Preparing to start process...'"
init = "echo 'Process is starting...'"
stop = "echo 'Stopping process...'"

[process.schedule]
start-time = "00:00:00"
stop-time = "23:59:00"
week-days = ["mon", "wed", "thu", "sun"]



# ---


[[process]]
id = "example_process 2"
apply-on = "2023-10-01T12:00:00"
command = "echo 'Hello, World!'"


"#,
        uuid::Uuid::new_v4()
    );

    if filename.exists() {
        return Err(format!("File '{}' already exists", filename.display()));
    }

    let mut file = File::create(filename).map_err(|e| e.to_string())?;
    file.write_all(content.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(())
}
