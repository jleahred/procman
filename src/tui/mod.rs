//  experimental!!!

use crate::{
    read_config_file,
    types::config::{Config, ProcessConfig, ProcessId},
};
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::{
    event::{self},
    terminal::LeaveAlternateScreen,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table, Wrap},
    Terminal,
};
use ratatui::{
    layout::{Direction, Layout},
    text::Spans,
    widgets::Paragraph,
};
use std::{
    collections::{BTreeMap, HashMap},
    fs, io,
    path::Path,
};

pub(crate) fn run(cfg_file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout(); //  todo:
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    crossterm::terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    // terminal.clear()?;

    let log_lines: Vec<String> = vec!["test line1".to_string(), "test line2".to_string()];
    let scroll_offset = 0;

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        use std::time::Duration;

        loop {
            let status = get_status(&cfg_file_name);

            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Length(5),
                        Constraint::Length(1),
                    ])
                    .split(f.size());

                // tabla de procesos
                f.render_widget(
                    render_table(&status.cfg_file_name, &status.merged_process_info),
                    chunks[0],
                );

                // zona de texto con scroll
                let paragraph = Paragraph::new(
                    log_lines
                        .iter()
                        .map(|line| Spans::from(Span::raw(line.clone())))
                        .collect::<Vec<Spans>>(),
                )
                .block(Block::default().borders(Borders::ALL).title("out/err"))
                .scroll((scroll_offset as u16, 0))
                .wrap(Wrap { trim: false });
                f.render_widget(paragraph, chunks[1]);

                // footer con teclas
                let footer = Paragraph::new(Spans::from(vec![
                    Span::raw("Press "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" or "),
                    Span::styled("Ctrl+C", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to exit."),
                ]));
                f.render_widget(footer, chunks[2]);
            })?;
            // terminal.draw(|f| {
            //     f.render_widget(render_table(&processes), f.size());
            // })?;

            // wait for 1 second before refreshing the screen
            if crossterm::event::poll(Duration::from_secs(1))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.code == event::KeyCode::Char('q')
                        || key.code == event::KeyCode::Char('c')
                            && key.modifiers.contains(event::KeyModifiers::CONTROL)
                    {
                        break;
                    }
                }
            }
        }

        Ok(())
    })();

    crossterm::terminal::disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    // terminal.clear()?;

    result
}

fn render_table<'a>(
    full_file_name: &str,
    info: &'a BTreeMap<ProcessId, MergedProcessInfoPerProcess>,
) -> Table<'a> {
    let render_rows: Vec<Row> = render_rows(&info);

    Table::new(render_rows)
        .header(
            Row::new(vec!["Process ID", "Status", "Command (in config)"])
                .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .block(
            Block::default()
                .title(format!(" Process Table for   [{}] ", full_file_name))
                .borders(Borders::ALL),
        )
        .widths(&[
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Percentage(60),
        ])
}

fn render_rows<'a>(info: &'a BTreeMap<ProcessId, MergedProcessInfoPerProcess>) -> Vec<Row<'a>> {
    info.iter()
        .map(|(proc_id, merged_info)| render_row(proc_id, merged_info))
        .collect()
}

fn render_row<'a>(_proc_id: &ProcessId, merged_info: &'a MergedProcessInfoPerProcess) -> Row<'a> {
    Row::new(vec![
        Cell::from(merged_info.process_id.0.clone()),
        Cell::from(render_status(merged_info)),
        Cell::from(render_command(merged_info)),
    ])
}

fn render_status<'a>(merged_info: &MergedProcessInfoPerProcess) -> Cell<'a> {
    let (st_color, st_text) = match merged_info.running {
        Some(ref running) => match running.status {
            crate::types::running_status::ProcessStatus::Running { .. } => {
                if merged_info.config_active.is_none() {
                    return Cell::from(Span::styled(
                        "running",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ));
                }
                (Color::Green, "running".to_string())
            }
            crate::types::running_status::ProcessStatus::ShouldBeRunning { .. } => {
                (Color::Yellow, "should be running".to_string())
            }
            crate::types::running_status::ProcessStatus::Stopped { .. } => {
                (Color::Yellow, "stopped".to_string())
            }
            crate::types::running_status::ProcessStatus::Stopping { .. } => {
                (Color::Yellow, "stopping".to_string())
            } // crate::types::running_status::ProcessStatus::Ready2Start { .. } => {
              //     (Color::Yellow, "ready".to_string())
              // }
              // crate::types::running_status::ProcessStatus::PendingHealthStartCheck {
              //     retries,
              //     ..
              // } => (Color::Yellow, format!("health start({})", retries)),
              // crate::types::running_status::ProcessStatus::Stopping { .. } => {
              //     (Color::Yellow, "stopping".to_string())
              // }
              // crate::types::running_status::ProcessStatus::ScheduledStop { .. } => {
              //     (Color::Yellow, "scheduled stop".to_string())
              // }
              // crate::types::running_status::ProcessStatus::PendingInitCmd { .. } => {
              //     if merged_info.config_active.is_none() {
              //         return Cell::from(Span::styled(
              //             "running init",
              //             Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
              //         ));
              //     } else {
              //         (Color::Yellow, "running init".to_string())
              //     }
              // }
        },
        None => {
            if merged_info.config_active.is_none() {
                return Cell::from(Span::styled(
                    "not running",
                    Style::default().fg(Color::Yellow),
                ));
            } else {
                return Cell::from(Span::styled(
                    "not running",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ));
            }
        }
    };

    Cell::from(Span::styled(st_text, Style::default().fg(st_color)))
}

fn render_command<'a>(merged_info: &MergedProcessInfoPerProcess) -> Cell<'a> {
    match merged_info.config_active {
        Some(ref config) => {
            return Cell::from(Span::styled(
                config.command.0.clone(),
                Style::default().fg(Color::White),
            ));
        }
        None => {
            if merged_info.in_config {
                return Cell::from(Span::styled(
                    "not activated",
                    Style::default().fg(Color::Yellow),
                ));
            } else {
            }
            return Cell::from(Span::styled(
                "not in config",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ));
        }
    }
}

struct Status {
    cfg_file_name: String,
    merged_process_info: BTreeMap<ProcessId, MergedProcessInfoPerProcess>,
}
struct MergedProcessInfoPerProcess {
    process_id: ProcessId,
    in_config: bool,
    config_active: Option<crate::types::config::ProcessConfig>,
    running: Option<crate::types::running_status::ProcessWatched>,
}

fn get_status(cfg_file_name: &str) -> Status {
    let relative_path = Path::new(cfg_file_name);
    let absolute_path = fs::canonicalize(&relative_path).expect("Failed to get absolute path");
    Status {
        cfg_file_name: absolute_path.display().to_string(),
        merged_process_info: get_process_info(cfg_file_name),
    }
}

fn get_process_info(cfg_file_name: &str) -> BTreeMap<ProcessId, MergedProcessInfoPerProcess> {
    let cfg = read_config_file::read_config_file_or_panic(&cfg_file_name); //  todo:0
    let running_status =
        crate::types::running_status::load_running_status("/tmp/procman/", &cfg.uid);
    get_process_info_merged(&cfg, &running_status.processes)
}

fn get_process_info_merged(
    config: &Config,
    running_status: &HashMap<ProcessId, crate::types::running_status::ProcessWatched>,
) -> BTreeMap<ProcessId, MergedProcessInfoPerProcess> {
    let mut result: BTreeMap<ProcessId, MergedProcessInfoPerProcess> = config
        .process
        .iter()
        .map(|process| {
            (
                process.id.clone(),
                MergedProcessInfoPerProcess {
                    process_id: process.id.clone(),
                    in_config: true,
                    config_active: None,
                    running: None,
                },
            )
        })
        .collect();

    for (proc_id, process_watched) in running_status {
        result
            .entry(proc_id.clone())
            .or_insert_with(|| MergedProcessInfoPerProcess {
                process_id: proc_id.clone(),
                in_config: false,
                config_active: None,
                running: Some(process_watched.clone()),
            });
    }

    let map_proc_id_active_cfg: BTreeMap<ProcessId, ProcessConfig> = config
        .get_active_procs_by_config()
        .0
        .values()
        .map(|process| (process.id.clone(), process.clone()))
        .collect();

    for (proc_id, process) in map_proc_id_active_cfg.iter() {
        if let Some(entry) = result.get_mut(proc_id) {
            entry.config_active = Some(process.clone());
        }
    }

    for (proc_id, process_watched) in running_status {
        if let Some(entry) = result.get_mut(proc_id) {
            entry.running = Some(process_watched.clone());
        }
    }

    result
}
