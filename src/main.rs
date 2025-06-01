mod check_run_once;
mod cli_params;
mod gen_simple_process_toml;
mod remove_old_status;
mod tui;
mod types;
mod watch_now;

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::{env, thread};
use types::config::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 && args[1] == "--watch-now" {
        let full_proc_file_name = &PathBuf::from(&args[2])
            .canonicalize()
            .unwrap_or_else(|err| {
                eprintln!("CRITIC: Failed to get absolute path: {}", err);
                std::process::exit(1);
            });

        if let Err(err) = watch_now::watch_now(full_proc_file_name) {
            eprintln!("CRITIC: canceling check   {}", err);
            std::process::exit(1);
        }
        return;
        // } else if args.len() == 3 && args[1] == "--supervise" {
        //     run_supervised(&args[2]).unwrap_or_else(|err| {
        //         eprintln!("CRITIC: {}", err);
        //         std::process::exit(1);
        //     });
        //     return;
    }

    let cli_params = cli_params::parse();

    match cli_params.command {
        cli_params::Commands::Run { processes_filename } => {
            let config = Config::read_from_file(&processes_filename).unwrap_or_else(|err| {
                eprintln!("CRITIC: Failed to read config file: {}", err.0);
                std::process::exit(1);
            });
            // read_config_file::read_config_file_or_panic(&processes_filename);
            let locked =
                check_run_once::check(&format!("{}.lock", config.uid.0)).unwrap_or_else(|err| {
                    eprintln!("CRITIC: {}", err);
                    std::process::exit(1);
                });
            run_in_loop(&processes_filename);

            check_run_once::remove_lock_file(&locked, &processes_filename); //  higienic, not critic
        }
        cli_params::Commands::Check { processes_filename } => {
            println!("Checking: {}", processes_filename.to_str().unwrap_or("?"));
            let _ = Config::read_from_file(&processes_filename).unwrap_or_else(|err| {
                eprintln!("CRITIC: Failed to read config file: {}", err.0);
                std::process::exit(1);
            });
            println!(
                "Check config OK: {}",
                processes_filename.to_str().unwrap_or("?")
            );
        }
        cli_params::Commands::Uid => {
            println!(
                "uid you can use on processes config file:   {}",
                uuid::Uuid::new_v4().to_string()
            );
            return;
        }
        cli_params::Commands::Tui => {
            remove_old_status::rename_old_status();
            tui::run().unwrap();
        }
        cli_params::Commands::ExpandTemplates { processes_filename } => {
            println!(
                "Expanding templates: {}",
                processes_filename.to_str().unwrap_or("?")
            );
            let expanded = Config::read_and_expand(&processes_filename).unwrap_or_else(|err| {
                eprintln!("CRITIC: Failed to read config file: {}", err.0);
                std::process::exit(1);
            });
            println!(
                "Expanded config: {} .....................................",
                processes_filename.to_str().unwrap_or("?")
            );
            println!("{}", expanded.0);
        }
        cli_params::Commands::GenFile { filename } => {
            if let Err(err) = gen_simple_process_toml::create(filename.as_ref()) {
                eprintln!("CRITIC: {}", err);
                std::process::exit(1);
            }
            println!("Generated simple process file successfully.");
        }
        cli_params::Commands::DeleteOldStatusFiles => {
            remove_old_status::rename_old_status();
            println!("Old status files deleted if any.");
        }
    }
}

fn run_in_loop(full_proc_file_name: &PathBuf) {
    loop {
        let current_exe = env::current_exe().expect("CRITIC: Can't get current executable path");

        // Create a new process
        let mut child =
            //Command::new(current_exe)
            Command::new("setsid")
            .arg(current_exe)
            .args(["--watch-now", full_proc_file_name.to_str().unwrap_or("?")])
            .spawn()
            .expect("CRITIC: Can't spawn child process");

        thread::sleep(Duration::from_secs(2));
        // println!("New process created with PID: {}", child.id());
        if child.wait().is_err() {
            eprintln!("Error waiting for child process");
        } else {
            println!("Watch finished");
        }
    }
}

// fn run_supervised(command: &str) -> Result<u32, io::Error> {
//     let mut child: Child = Command::new("sh")
//         .arg("-c")
//         .arg(&command)
//         .env("PROCMAN", &command)
//         .spawn()?;
//     //  todo convendría desconectar la salida de error y la salida estándar para evitar zombis?
//     println!("New SUPERVISED process created with PID: {}", child.id());
//     if child.wait().is_err() {
//         eprintln!("Error waiting for child process");
//     } else {
//         println!("supervised finished OK");
//     }
//     Ok(child.id())
//     // }
// }
