use std::{collections::BTreeMap, path::PathBuf};

use crate::{
    types::{
        config::{ProcessConfig, ProcessId},
        running_status::ProcessStatus,
    },
    watch_now::WatchNow,
    Config,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Cell, HighlightSpacing, Row, Table, TableState},
    Frame,
};

const PAGE_STEP: usize = 10;

pub(super) struct Processes {
    pub(super) full_config_filename: PathBuf,
    watched: WatchNow,
    pub(super) only_in_config: BTreeMap<ProcessId, ProcessConfig>,

    table_state: TableState,
}

impl Processes {
    fn len(&self) -> usize {
        self.watched.processes.len() + self.only_in_config.len()
    }

    pub(super) fn update_data(&mut self) -> Result<(), String> {
        self.watched = WatchNow::create(&self.full_config_filename)?;
        self.only_in_config = get_only_in_config(&self.full_config_filename, &self.watched)?;
        Ok(())
    }

    pub(super) fn create(full_config_filename: &PathBuf) -> Result<Self, String> {
        let watched = WatchNow::create(full_config_filename)?;
        let only_in_config = get_only_in_config(&full_config_filename, &watched)?;
        Ok(Self {
            full_config_filename: full_config_filename.clone(),
            watched,
            only_in_config,
            table_state: TableState::default(),
        })
    }

    pub(super) fn render(&mut self, frame: &mut Frame, area: Rect) {
        let title = Line::from(format!(
            "  [ {} ]  ",
            self.full_config_filename.display().to_string()
        ))
        .centered();
        frame.render_widget(
            Block::bordered()
                .border_set(symbols::border::ROUNDED)
                .title(title),
            area,
        );

        let area = frame.area().inner(Margin::new(1, 1));

        render_processes(frame, area, self);
    }

    pub(super) fn handle_events(&mut self, event: &Event) -> super::Command {
        match event {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                handle_key_event(key_event, self)
            }
            _ => super::Command::None,
        }
    }
}

pub(super) fn render_processes(frame: &mut Frame, area: Rect, processes: &Processes) {
    let header_style = Style::default().bold();
    let selected_row_style = Style::default().add_modifier(Modifier::REVERSED);
    let selected_col_style = Style::default();
    let selected_cell_style = Style::default()
        // .add_modifier(Modifier::REVERSED)
        // .fg(self.colors.selected_cell_style_fg)
        ;

    // let area = frame.area().inner(Margin::new(1, 1));
    let header = ["id", "status", "Command"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);

    let rows = processes
        .watched
        .processes
        .iter()
        .map(|(proc_id, proc_info)| {
            let process_config = proc_info.process_config.clone();
            let process_watched = proc_info.process_watched.clone();

            let status = match (&process_config, &process_watched) {
                (Some(_config), Some(running)) => match running.status {
                    ProcessStatus::ShouldBeRunning => {
                        Cell::from("sould be running").fg(Color::Red).bold()
                    }
                    ProcessStatus::Running { .. } => Cell::from("running").fg(Color::Green),
                    ProcessStatus::PendingBeforeCmd => {
                        Cell::from("pend before cmd").fg(Color::Yellow)
                    }
                    ProcessStatus::PendingInitCmd { .. } => {
                        Cell::from("pend init cmd").fg(Color::Yellow)
                    }
                    ProcessStatus::Stopping { .. } => Cell::from("stopping").fg(Color::Red).bold(),
                    ProcessStatus::Stopped { .. } => Cell::from("stopped").fg(Color::Red).bold(),
                    ProcessStatus::WaittingPidFile { .. } => {
                        Cell::from("waitting pid file").fg(Color::Red).bold()
                    }
                    ProcessStatus::StoppingWaittingPidFile { .. } => {
                        Cell::from("stopping waitting pidfile")
                            .fg(Color::Red)
                            .bold()
                    }
                },
                (None, Some(running)) => match running.status {
                    ProcessStatus::ShouldBeRunning => {
                        Cell::from("sould be running").fg(Color::Red).bold()
                    }
                    ProcessStatus::Running { .. } => Cell::from("running").fg(Color::Red).bold(),
                    ProcessStatus::PendingBeforeCmd => {
                        Cell::from("pend before cmd").fg(Color::Yellow)
                    }
                    ProcessStatus::PendingInitCmd { .. } => {
                        Cell::from("pend init cmd").fg(Color::Red).bold()
                    }
                    ProcessStatus::Stopping { .. } => Cell::from("stopping").fg(Color::Red).bold(),
                    ProcessStatus::Stopped { .. } => Cell::from("stopped").fg(Color::Yellow),
                    ProcessStatus::WaittingPidFile { .. } => {
                        Cell::from("waitting pid file").fg(Color::Red).bold()
                    }
                    ProcessStatus::StoppingWaittingPidFile { .. } => {
                        Cell::from("stopping waitting pidfile")
                            .fg(Color::Red)
                            .bold()
                    }
                },
                (Some(_), None) => Cell::from("PENDING").fg(Color::Red).bold(),
                (_, _) => Cell::from("INCONSISTENCY").fg(Color::Red).bold(),
            };

            let command = match process_config {
                Some(config) => config.command.str().to_string(),
                None => String::from("No command"),
            };
            Row::new(vec![
                Cell::from(proc_id.0.to_string()),
                status,
                Cell::from(command),
            ])
        });
    let config_only_rows = processes
        .only_in_config
        .iter()
        .filter(|(proc_id, _)| !processes.watched.processes.contains_key(proc_id))
        .map(|(proc_id, process_config)| {
            let command = process_config.command.str().to_string();
            Row::new(vec![
                Cell::from(proc_id.0.to_string()),
                Cell::from("cfg inactive").fg(Color::Yellow).italic(),
                Cell::from(command),
            ])
        });

    let rows = rows.chain(config_only_rows);

    let widths = [
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(6),
    ];

    let t = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(selected_row_style)
        .column_highlight_style(selected_col_style)
        .cell_highlight_style(selected_cell_style)
        // .highlight_symbol(Text::from(vec![
        //      "▶ ".into(),
        // ]))
        // .bg(self.colors.buffer_bg)
        .highlight_spacing(HighlightSpacing::Always);
    frame.render_stateful_widget(t, area, &mut processes.table_state.clone());
}

fn handle_key_event(key_event: &KeyEvent, proc: &mut Processes) -> super::Command {
    match key_event.code {
        KeyCode::Down => {
            if let Some(selected) = proc.table_state.selected() {
                let next = (selected + 1) % proc.len();
                proc.table_state.select(Some(next));
            } else {
                proc.table_state.select(Some(0));
            }
            super::Command::None
        }
        KeyCode::Up => {
            if let Some(selected) = proc.table_state.selected() {
                let prev = if selected == 0 {
                    proc.len() - 1
                } else {
                    selected - 1
                };
                proc.table_state.select(Some(prev));
            } else {
                proc.table_state.select(Some(0));
            }
            super::Command::None
        }
        KeyCode::PageDown => {
            let next = PAGE_STEP + *proc.table_state.selected().get_or_insert(0);
            if next >= proc.len() {
                proc.table_state.select(Some(proc.len() - 1));
            } else {
                proc.table_state.select(Some(next));
            }
            super::Command::None
        }
        KeyCode::PageUp => {
            let current = *proc.table_state.selected().get_or_insert(0);
            if current > PAGE_STEP {
                let next = current - PAGE_STEP;
                proc.table_state.select(Some(next));
            } else {
                proc.table_state.select(Some(0));
            }
            super::Command::None
        }
        // KeyCode::Esc => {
        //     proc.table_state.select(None);
        //     super::Command::None
        // }
        KeyCode::Esc => {
            if proc.table_state.selected().is_some() {
                proc.table_state.select(None);
                super::Command::None
            } else {
                super::Command::ChooseFile
            }
        }

        _ => super::Command::None,
    }
}

fn get_only_in_config(
    full_config_filename: &PathBuf,
    watched: &WatchNow,
) -> Result<BTreeMap<ProcessId, ProcessConfig>, String> {
    let mut result = BTreeMap::new();

    let config: Config =
        Config::read_from_file(&full_config_filename).map_err(|e| e.0.to_string())?;
    for process_config in config.process.iter() {
        if !watched.processes.contains_key(&process_config.id) {
            result
                .entry(process_config.id.clone())
                .or_insert_with(|| process_config.clone());
        }
    }
    Ok(result)
}
