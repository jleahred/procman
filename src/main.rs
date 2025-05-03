mod check_run_once;
mod cli_params;
mod watch_now;
mod tui;
mod types;

use std::process::Command;
use std::time::Duration;
use std::{env, thread};
use types::config::Config;

// run supervised
// use std::process::{Child};
// use std::{io};
// use nix::libc;
// use std::os::unix::process::CommandExt;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 && args[1] == "--watch-now" {
        if let Err(err) = watch_now::watch_now(&args[2]) {
            eprintln!("CRITIC: canceling check   {}", err);
            std::process::exit(1);
        }
        return;
    }
    // else if args.len() == 3 && args[1] == "--supervise" {
    //     run_supervised(&args[2]).unwrap_or_else(|err| {
    //         eprintln!("CRITIC: {}", err);
    //         std::process::exit(1);
    //     });
    //     return;
    // }

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
            println!("Running: {}", processes_filename);
            run_in_loop(&processes_filename);

            check_run_once::remove_lock_file(&locked, &processes_filename); //  higienic, not critic
        }
        cli_params::Commands::Check { processes_filename } => {
            println!("Checking: {}", processes_filename);
            let _ = Config::read_from_file(&processes_filename).unwrap_or_else(|err| {
                eprintln!("CRITIC: Failed to read config file: {}", err.0);
                std::process::exit(1);
            });
            println!("Check config OK: {}", processes_filename);
        }
        cli_params::Commands::Uid => {
            println!(
                "uid you can use on processes config file:   {}",
                uuid::Uuid::new_v4().to_string()
            );
            return;
        }
        cli_params::Commands::Tui { processes_filename } => {
            tui::run(&processes_filename).unwrap();
        }
        cli_params::Commands::ExpandTemplates { processes_filename } => {
            println!("Expanding templates: {}", processes_filename);
            let expanded = Config::read_and_expand(&processes_filename).unwrap_or_else(|err| {
                eprintln!("CRITIC: Failed to read config file: {}", err.0);
                std::process::exit(1);
            });
            println!("Expanded config: {} .....................................", processes_filename);
            println!("{}", expanded.0);
        }
    }
}

fn run_in_loop(prc_cfg_file_name: &str) {
    loop {
        let current_exe = env::current_exe().expect("CRITIC: Can't get current executable path");

        // Create a new process
        let mut child = 
            //Command::new(current_exe)
            Command::new("setsid")
            .arg(current_exe)
            .args(["--watch-now", prc_cfg_file_name])
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


// fn run_supervised(
//     command: &str,
// ) -> Result<u32, io::Error> {
//     unsafe {
//     let mut child: Child = Command::new("sh")
//         .arg("-c")
//         .arg(&command)
//         .pre_exec(|| {
//             // PR_SET_PDEATHSIG = 1
//             // SIGKILL = 9
//             if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0 {
//                 return Err(std::io::Error::last_os_error());
//             }
//             Ok(())
//         })
//         .env("PROCMAN", &command)
//         .spawn()?;
        
//         //  todo convendría desconectar la salida de error y la salida estándar para evitar zombis?

//         println!("New SUPERVISED process created with PID: {}", child.id());

//         if child.wait().is_err() {
//             eprintln!("Error waiting for child process");
//         } else {
//             println!("supervised finished OK");
//         }

//     Ok(child.id())
//     }
// }
