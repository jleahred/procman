mod imp;

use crate::read_config_file::read_config_file_or_panic;
use crate::types::config::Config;
use crate::types::running_status;

const RUNNING_STATUS_FOLDER: &str = "/tmp/procman";

pub(crate) fn one_shot(full_file_name: &str) {
    println!("\n--------------------------------------------------------------------------------");
    println!("Checking... {}\n", chrono::Local::now());

    let config: Config = read_config_file_or_panic(full_file_name);
    let running_status = running_status::load_running_status(RUNNING_STATUS_FOLDER, &config.uid);
    let active_procs_by_config = config.get_active_procs_by_config();
    let running_ids = running_status.get_running_ids();
    let whatched_ids = running_status.get_watched_ids();
    let active_procs_cfg_all_depends_running =
        imp::filter_active_procs_by_config_with_running(&active_procs_by_config, &running_ids);
    let pending2watch =
        imp::get_pending2run_processes(&active_procs_cfg_all_depends_running, &whatched_ids);

    //  --------------------

    running_status
        .del_if_missing_pid()
        .save(RUNNING_STATUS_FOLDER)
        .send_kill_on_stopping_processes()
        .save(RUNNING_STATUS_FOLDER)
        .run_init_cmds()
        .save(RUNNING_STATUS_FOLDER)
        .check_start_held_processes()
        .save(RUNNING_STATUS_FOLDER)
        .mark_stopping_not_active_in_cfg(&active_procs_cfg_all_depends_running)
        .save(RUNNING_STATUS_FOLDER)
        .mark_stopping_if_apply_on_changed(&active_procs_by_config)
        .save(RUNNING_STATUS_FOLDER)
        .ready2start_from_missing_watched(&pending2watch)
        .save(RUNNING_STATUS_FOLDER)
        .launch_ready2start()
        .save(RUNNING_STATUS_FOLDER);
}
