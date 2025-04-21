//  experimental!!!

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
    Terminal,
};
use std::io;

pub(crate) fn run(processes_uid: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Configurar el backend y el terminal
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    crossterm::terminal::enable_raw_mode()?;
    terminal.clear()?;

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        use std::time::{Duration, Instant};

        loop {
            let binding = get_process_info(processes_uid);
            terminal.draw(|f| {
                let rows: Vec<Row> = binding
                    .iter()
                    .map(|(id, status, command)| {
                        let color = match status.as_str() {
                            "running" => Color::Green,
                            "starting" => Color::Yellow,
                            "stopped" => Color::Red,
                            _ => Color::White,
                        };

                        Row::new(vec![
                            Cell::from(id.clone()),
                            Cell::from(Span::styled(status, Style::default().fg(color))),
                            Cell::from(command.clone()),
                        ])
                    })
                    .collect();

                let table = Table::new(rows)
                    .header(
                        Row::new(vec!["Process ID", "Status", "Command"])
                            .style(Style::default().add_modifier(Modifier::BOLD)),
                    )
                    .block(
                        Block::default()
                            .title("Process Table")
                            .borders(Borders::ALL),
                    )
                    .widths(&[
                        Constraint::Length(15),
                        Constraint::Length(15),
                        Constraint::Percentage(70),
                    ]);

                f.render_widget(table, f.size());
            })?;

            // Espera eventos por 1 segundo o hasta que haya input
            if crossterm::event::poll(Duration::from_secs(1))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.code == event::KeyCode::Char('q')
                        || key.code == event::KeyCode::Char('z')
                            && key.modifiers.contains(event::KeyModifiers::CONTROL)
                    {
                        break;
                    }
                }
            }

            // Aquí podrías actualizar el vector `processes` si lo hicieras mutable o lo recargases desde fichero
        }
        Ok(())
    })();

    crossterm::terminal::disable_raw_mode()?;
    terminal.clear()?;
    result
}

fn get_process_info(processes_uid: &str) -> Vec<(String, String, String)> {
    let running_status = load_running_status("/tmp/procman/", &ConfigUid(processes_uid.to_owned()));

    let mut processes: Vec<(String, String, String)> = running_status
        .processes
        .iter()
        .map(|(id, process)| {
            let status = match &process.status {
                ProcessStatus::Running { .. } => "running".to_string(),
                ProcessStatus::Ready2Start { .. } => "ready".to_string(),
                ProcessStatus::PendingHealthStartCheck { .. } => "starting".to_string(),
                ProcessStatus::Stopping { .. } => "stopping".to_string(),
                ProcessStatus::ScheduledStop { .. } => "scheduled_stop".to_string(),
            };

            let command = match &process.status {
                ProcessStatus::Ready2Start { command, .. } => command.0.to_string(),
                _ => "N/A".to_string(),
            };

            (id.0.to_string(), status, command)
        })
        .collect();

    processes.sort_by(|a, b| a.0.cmp(&b.0)); // Ordenar alfabéticamente por el primer campo
    processes
}

use crate::types::config::{Command, CommandStartHealthCheck, ConfigUid, ProcessConfig, ProcessId};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct RunningStatus {
    pub(crate) file_uid: ConfigUid,
    #[serde(rename = "file_format")]
    pub(crate) _file_format: String,
    pub(crate) processes: HashMap<ProcessId, ProcessWatched>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ProcessWatched {
    pub(crate) id: ProcessId,
    pub(crate) procrust_uid: String,
    pub(crate) apply_on: NaiveDateTime,
    pub(crate) status: ProcessStatus,
    pub(crate) applied_on: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) enum ProcessStatus {
    Ready2Start {
        command: Command,
        process_id: ProcessId,
        start_health_check: Option<CommandStartHealthCheck>,
        apply_on: NaiveDateTime,
    },
    PendingHealthStartCheck {
        pid: u32,
        start_health_check: Option<CommandStartHealthCheck>,
        retries: u32,
        last_attempt: chrono::DateTime<chrono::Local>,
    },
    Running {
        pid: u32,
    },
    Stopping {
        pid: u32,
        retries: u32,
        last_attempt: chrono::DateTime<chrono::Local>,
    },
    ScheduledStop {
        pid: u32,
    },
}

// use crate::types::config::ConfigUid;
// use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub(crate) fn load_running_status(file_path: &str, file_uid: &ConfigUid) -> RunningStatus {
    let full_path = format!("{}/{}.toml", file_path, file_uid.0); // Construir la ruta completa

    if Path::new(&full_path).exists() {
        let content = fs::read_to_string(&full_path)
            .unwrap_or_else(|err| panic!("Failed to read file {}: {}", full_path, err));
        toml::from_str(&content)
            .unwrap_or_else(|err| panic!("Failed to parse TOML from file {}: {}", full_path, err))
    } else {
        println!(
            "File {} does not exist. Returning default RunningStatus.",
            full_path
        );
        RunningStatus {
            file_uid: file_uid.clone(),
            _file_format: String::from("0"),
            processes: HashMap::new(),
        }
    }
}
