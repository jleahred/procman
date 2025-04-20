mod one_shot;
mod read_config_file;
mod types;
mod cli_params;

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
            let _locked = try_acquire_lock(&format!("/tmp/procman/{}.lock", config.uid.0)).unwrap_or_else(|err| {
                eprintln!("CRITIC: {}", err);
                std::process::exit(1);
            });
            println!("Running: {}", processes_filename);
            run_in_loop(&processes_filename);
        }
        cli_params::Commands::Check { filename } => {
            println!("Checking: {}", filename);
            let _ = read_config_file::read_config_file_or_panic(&filename);
        }
        cli_params::Commands::Uid => {
            println!("uid you can use on processes config file:   {}", uuid::Uuid::new_v4().to_string());
            return;
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


use std::fs::File;
use std::path::Path;
use fs2::FileExt;

fn try_acquire_lock(lock_path: &str) -> Result<File, String> {
    let path = Path::new(lock_path);

    // open or create the lock file
    let file = File::create(&path)
        .map_err(|e| format!("Cannot create lock file: {}", e))?;

    // get exclusive lock
    file.try_lock_exclusive()
        .map_err(|_| "There are another instance execution".to_string())?;

    Ok(file) // keep the lock until the file is dropped
}
