mod one_shot;
mod read_config_file;
mod types;
mod cli_params;
mod check_run_once;
mod tui;//  experimental

use std::env;
use std::process::Command;
use std::time::Duration;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 && args[1] == "--one-shot" {
        one_shot::one_shot2(&args[2]);
        return;
    } 

    let cli_params = cli_params::parse();

    match cli_params.command {
        cli_params::Commands::Run { processes_filename } => {
            let config = read_config_file::read_config_file_or_panic(&processes_filename);
            let _locked = check_run_once::check(&format!("{}.lock", config.uid.0)).unwrap_or_else(|err| {
                eprintln!("CRITIC: {}", err);
                std::process::exit(1);
            });
            println!("Running: {}", processes_filename);
            run_in_loop(&processes_filename);
        }
        cli_params::Commands::Check { processes_filename } => {
            println!("Checking: {}", processes_filename);
            let _ = read_config_file::read_config_file_or_panic(&processes_filename);
            println!("Check config OK: {}", processes_filename);
        }
        cli_params::Commands::Uid => {
            println!("uid you can use on processes config file:   {}", uuid::Uuid::new_v4().to_string());
            return;
            }
            cli_params::Commands::Tui { processes_filename } => {
                println!("TUI");
                let config = read_config_file::read_config_file_or_panic(&processes_filename);
                tui::run(&config.uid.0).unwrap();
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
            .args(["--one-shot", prc_cfg_file_name])
            .spawn()
            .expect("CRITIC: Can't spawn child process");

        // println!("New process created with PID: {}", child.id());
        if child.wait().is_err() {
            eprintln!("Error waiting for child process");
        } else {
            println!("Shot finished");
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}


