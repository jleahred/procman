use super::*;

use chrono::Weekday;
use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::Deserialize;
use std::fmt;

pub(super) fn check(cfg: &Config) -> Result<(), String> {
    if cfg.uid.0.is_empty() {
        return Err("UID cannot be empty".to_string());
    }
    if cfg.process.is_empty() {
        return Err("Process list cannot be empty".to_string());
    }
    for process in &cfg.process {
        process.check_config()?;
    }
    depends_exists(cfg)?;
    circular_refs(cfg)?;
    Ok(())
}

pub(super) fn is_valid_start_stop(proc_conf: &ProcessConfig) -> Result<(), String> {
    if let Some(schedule) = &proc_conf.schedule {
        if schedule.start_time < schedule.stop_time {
            Ok(())
        } else {
            Err(format!(
                "Invalid time range: start_time ({}) is not before stop_time ({})",
                schedule.start_time, schedule.stop_time
            ))
        }
    } else {
        Ok(()) // If no schedule is defined, consider it valid
    }
}

pub(super) fn deserialize_day_selection<'de, D>(deserializer: D) -> Result<DaySelection, D::Error>
where
    D: Deserializer<'de>,
{
    struct DaySelectionVisitor;

    impl<'de> Visitor<'de> for DaySelectionVisitor {
        type Value = DaySelection;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a list of weekdays, 'mon-fri' or 'all'")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match v {
                "mon-fri" => Ok(DaySelection::Mon2Fri),
                "all" => Ok(DaySelection::All),
                _ => Err(E::custom(format!("unexpected string: {}", v))),
            }
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let days: Vec<Weekday> =
                Deserialize::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))?;
            Ok(DaySelection::Days(days))
        }
    }

    deserializer.deserialize_any(DaySelectionVisitor)
}

pub(super) fn matches(ds: &DaySelection, weekday: chrono::Weekday) -> bool {
    match ds {
        DaySelection::Days(days) => days.contains(&weekday),
        DaySelection::Mon2Fri => {
            weekday.num_days_from_monday() >= chrono::Weekday::Mon.num_days_from_monday()
                && weekday.num_days_from_monday() <= chrono::Weekday::Fri.num_days_from_monday()
        }
        DaySelection::All => true,
    }
}

pub(super) fn get_active_procs_by_config(config: &Config) -> ActiveProcessByConfig {
    let now = Local::now().naive_local();
    let mut process_map: HashMap<ProcessId, ProcessConfig> = HashMap::new();

    for process in &config.process {
        if process.apply_on > now {
            continue;
        }

        if let Some(schedule) = &process.schedule {
            let weekday = now.weekday();
            let time = now.time();

            if !schedule.week_days.matches(weekday) {
                continue;
            }

            if time < schedule.start_time || time >= schedule.stop_time {
                continue;
            }
        }

        match process.process_type {
            ProcessType::Normal | ProcessType::PodmanCid  => {}
            ProcessType::Fake => {
                // println!("[{}] Process type is fake, skipping...", process.id.0);
                continue;
            }
        }

        // keep more recent process config
        let entry = process_map
            .entry(process.id.clone())
            .or_insert_with(|| process.clone());

        if entry.apply_on < process.apply_on {
            *entry = process.clone();
        }
    }

    ActiveProcessByConfig(process_map)
}

//  --------------------
fn depends_exists(cfg: &Config) -> Result<(), String> {
    let process_ids: std::collections::HashSet<_> = cfg.process.iter().map(|p| &p.id).collect();

    for process in &cfg.process {
        for dependency in &process.depends_on {
            if !process_ids.contains(&dependency) {
                return Err(format!(
                    "Process '{}' depends on non-existent process '{}'",
                    process.id.0, dependency.0
                ));
            }
        }
    }

    Ok(())
}

fn circular_refs(cfg: &Config) -> Result<(), String> {
    fn has_cycle(
        process_id: &ProcessId,
        visited: &mut std::collections::HashSet<ProcessId>,
        stack: &mut std::collections::HashSet<ProcessId>,
        processes: &std::collections::HashMap<ProcessId, &ProcessConfig>,
    ) -> bool {
        if !visited.contains(process_id) {
            visited.insert(process_id.clone());
            stack.insert(process_id.clone());

            if let Some(process) = processes.get(process_id) {
                for dependency in &process.depends_on {
                    if !visited.contains(dependency)
                        && has_cycle(dependency, visited, stack, processes)
                    {
                        return true;
                    } else if stack.contains(dependency) {
                        return true;
                    }
                }
            }
        }

        stack.remove(process_id);
        false
    }

    let processes: std::collections::HashMap<_, _> =
        cfg.process.iter().map(|p| (p.id.clone(), p)).collect();

    let mut visited = std::collections::HashSet::new();
    let mut stack = std::collections::HashSet::new();

    for process in &cfg.process {
        if has_cycle(&process.id, &mut visited, &mut stack, &processes) {
            return Err(format!(
                "Circular dependency detected involving process '{}'",
                process.id.0
            ));
        }
    }

    Ok(())
}
