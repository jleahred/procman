use chrono::{DateTime, Duration, Local};
use std::fs::File;
use std::fs::{self};
use std::io::{self, BufRead};

pub(super) fn rename_old_status() {
    let dir_path = "/tmp/procman";
    let entries = match fs::read_dir(dir_path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error reading directory {}: {}", dir_path, e);
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_e) => continue,
        };
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }

        let file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let reader = io::BufReader::new(file);

        let mut last_update_str = None;
        let mut original_file_full_path = None;

        for line in reader.lines().flatten() {
            if line.starts_with("last_update") {
                if let Some(idx) = line.find('=') {
                    last_update_str = Some(line[idx + 1..].trim().trim_matches('"').to_string());
                }
            }
            if line.starts_with("original_file_full_path") {
                if let Some(idx) = line.find('=') {
                    original_file_full_path =
                        Some(line[idx + 1..].trim().trim_matches('"').to_string());
                }
            }
            if last_update_str.is_some() && original_file_full_path.is_some() {
                break;
            }
        }

        if let (Some(mut last_update), Some(orig_path)) = (last_update_str, original_file_full_path)
        {
            // Trim fractional seconds to 9 digits if necessary
            if let Some(dot_pos) = last_update.find('.') {
                let after_dot = &last_update[dot_pos + 1..];
                if let Some(t_pos) = after_dot.find(|c: char| !c.is_ascii_digit()) {
                    let frac_digits = &after_dot[..t_pos];
                    if frac_digits.len() > 9 {
                        let trimmed = format!(
                            "{}.{}{}",
                            &last_update[..dot_pos],
                            &frac_digits[..9],
                            &after_dot[t_pos..]
                        );
                        last_update = trimmed;
                    }
                } else if after_dot.len() > 9 {
                    let trimmed = format!("{}.{}", &last_update[..dot_pos], &after_dot[..9]);
                    last_update = trimmed;
                }
            }
            dbg!(&last_update);
            use chrono::{NaiveDateTime, TimeZone};

            let parsed_dt = DateTime::parse_from_rfc3339(&last_update)
                .map(|dt| dt.with_timezone(&Local))
                .or_else(|_| {
                    NaiveDateTime::parse_from_str(&last_update, "%Y-%m-%dT%H:%M:%S%.f")
                        .map(|naive| Local.from_local_datetime(&naive).unwrap())
                });

            if let Ok(dt_local) = parsed_dt {
                if Local::now() - dt_local > Duration::days(7) {
                    if let Err(e) = fs::remove_file(&path) {
                        eprintln!("Error deleting file {:?}: {}", path, e);
                    } else {
                        println!(
                            "File too long not running deleting the status file: {} for {}",
                            path.display(),
                            orig_path
                        );
                    }
                }
            } else {
                eprintln!("Error parsing date from file: {}", path.display());
            }
        }
    }
}
