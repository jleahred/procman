//  experimental!!!

use crossterm::event::{self};
use ratatui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
    Terminal,
};
use ratatui::{
    layout::{Direction, Layout},
    text::Spans,
    widgets::Paragraph,
};
use std::{
    collections::{BTreeMap, HashMap},
    io,
};

use crate::{
    read_config_file,
    types::config::{Config, ProcessConfig, ProcessId},
};

pub(crate) fn run() -> Result<(), Box<dyn std::error::Error>> {
    // configure
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    crossterm::terminal::enable_raw_mode()?;
    terminal.clear()?;

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        use std::time::Duration;

        loop {
            let processes = get_process_info();

            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Min(1), Constraint::Length(1)])
                    .split(f.size());

                // tabla de procesos
                f.render_widget(render_table(&processes), chunks[0]);

                // footer con teclas
                let footer = Paragraph::new(Spans::from(vec![
                    Span::raw("Press "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" or "),
                    Span::styled("Ctrl+C", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to exit."),
                ]));
                f.render_widget(footer, chunks[1]);
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
    terminal.clear()?;
    result
}

fn render_table(info: &BTreeMap<ProcessId, MergedProcessInfo>) -> Table {
    let render_rows: Vec<Row> = render_rows(&info);

    Table::new(render_rows)
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
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Percentage(60),
        ])
}

fn render_rows(info: &BTreeMap<ProcessId, MergedProcessInfo>) -> Vec<Row> {
    info.iter()
        .map(|(proc_id, merged_info)| render_row(proc_id, merged_info))
        .collect()
}

fn render_row<'a>(proc_id: &ProcessId, merged_info: &MergedProcessInfo) -> Row<'a> {
    Row::new(vec![
        Cell::from(merged_info.process_id.0.clone()),
        Cell::from(render_status(merged_info)),
        Cell::from(render_command(merged_info)),
    ])
}

fn render_status<'a>(merged_info: &MergedProcessInfo) -> Cell<'a> {
    let (st_color, st_text) = match merged_info.running {
        Some(ref running) => match running.status {
            crate::types::running_status::ProcessStatus::Running { .. } => {
                (Color::Green, "running")
            }
            crate::types::running_status::ProcessStatus::Ready2Start { .. } => {
                (Color::Yellow, "ready")
            }
            crate::types::running_status::ProcessStatus::PendingHealthStartCheck { .. } => {
                (Color::Yellow, "pend health start")
            }
            crate::types::running_status::ProcessStatus::Stopping { .. } => {
                (Color::Yellow, "stopping")
            }
            crate::types::running_status::ProcessStatus::ScheduledStop { .. } => {
                (Color::Yellow, "scheduled stop")
            }
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

fn render_command<'a>(merged_info: &MergedProcessInfo) -> Cell<'a> {
    match merged_info.config_active {
        Some(ref config) => {
            return Cell::from(Span::styled(
                config.command.0.clone(),
                Style::default().fg(Color::White),
            ));
        }
        None => {
            return Cell::from(Span::styled(
                "not actived",
                Style::default().fg(Color::Yellow),
            ));
        }
    }
}

struct MergedProcessInfo {
    process_id: ProcessId,
    config_active: Option<crate::types::config::ProcessConfig>,
    running: Option<crate::types::running_status::ProcessWatched>,
}

fn get_process_info() -> BTreeMap<ProcessId, MergedProcessInfo> {
    let cfg = read_config_file::read_config_file_or_panic("processes.toml"); //  todo:0
    let running_status =
        crate::types::running_status::load_running_status("/tmp/procman/", &cfg.uid);
    get_process_info_merged(&cfg, &running_status.processes)
}

fn get_process_info_merged(
    config: &Config,
    running_status: &HashMap<ProcessId, crate::types::running_status::ProcessWatched>,
) -> BTreeMap<ProcessId, MergedProcessInfo> {
    let mut result: BTreeMap<ProcessId, MergedProcessInfo> = config
        .process
        .iter()
        .map(|process| {
            (
                process.id.clone(),
                MergedProcessInfo {
                    process_id: process.id.clone(),
                    config_active: None,
                    running: None,
                },
            )
        })
        .collect();

    let map_proc_id_active_cfg: BTreeMap<ProcessId, ProcessConfig> = config
        .get_active_procs_by_config()
        .into_iter()
        .map(|process| (process.id.clone(), process))
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
