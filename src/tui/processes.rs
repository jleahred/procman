use crate::types::running_status::ProcessStatus;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Cell, Row, Table},
};
use ratatui::{layout::Rect, style::Stylize, widgets::HighlightSpacing, Frame};

impl super::App {
    pub(super) fn render_processes(&mut self, frame: &mut Frame, area: Rect) {
        let header_style = Style::default();
        let selected_row_style = Style::default().add_modifier(Modifier::REVERSED);
        let selected_col_style = Style::default();
        let selected_cell_style = Style::default()
            // .add_modifier(Modifier::REVERSED)
            // .fg(self.colors.selected_cell_style_fg)
            ;

        let header = ["id", "status", "Command"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);

        let rows = self
            .processes
            .watched
            .processes
            .iter()
            .map(|(proc_id, proc_info)| {
                let process_config = proc_info.process_config.clone();
                let process_watched = proc_info.process_watched.clone();

                let status = match (&process_config, &process_watched) {
                    (Some(_config), Some(running)) => match running.status {
                        ProcessStatus::ShouldBeRunning => {
                            Cell::from("sould be running").fg(Color::Yellow)
                        }
                        ProcessStatus::Running { .. } => Cell::from("running").fg(Color::Green),
                        ProcessStatus::PendingBeforeCmd => {
                            Cell::from("pend before cmd").fg(Color::Yellow)
                        }
                        ProcessStatus::PendingInitCmd { .. } => {
                            Cell::from("pend init cmd").fg(Color::Yellow)
                        }
                        ProcessStatus::Stopping { .. } => Cell::from("stopping").fg(Color::Yellow),
                        ProcessStatus::Stopped { .. } => Cell::from("stopped").fg(Color::Yellow),
                    },
                    (None, Some(running)) => match running.status {
                        ProcessStatus::ShouldBeRunning => {
                            Cell::from("sould be running").fg(Color::Red).bold()
                        }
                        ProcessStatus::Running { .. } => {
                            Cell::from("running").fg(Color::Red).bold()
                        }
                        ProcessStatus::PendingBeforeCmd => {
                            Cell::from("pend before cmd").fg(Color::Red).bold()
                        }
                        ProcessStatus::PendingInitCmd { .. } => {
                            Cell::from("pend init cmd").fg(Color::Red).bold()
                        }
                        ProcessStatus::Stopping { .. } => {
                            Cell::from("stopping").fg(Color::Red).bold()
                        }
                        ProcessStatus::Stopped { .. } => Cell::from("stopped").fg(Color::Yellow),
                    },
                    (Some(_), None) => Cell::from("PENDING").fg(Color::Red).bold(),
                    (_, _) => Cell::from("INCONSISTENCY").fg(Color::Red).bold(),
                };

                let command = match process_config {
                    Some(config) => config.command.0.to_string(),
                    None => String::from("No command"),
                };
                Row::new(vec![
                    Cell::from(proc_id.0.to_string()),
                    status,
                    Cell::from(command),
                ])
            });
        let config_only_rows = self
            .processes
            .only_in_config
            .iter()
            .filter(|(proc_id, _)| !self.processes.watched.processes.contains_key(proc_id))
            .map(|(proc_id, process_config)| {
                let command = process_config.command.0.to_string();
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
        frame.render_stateful_widget(t, area, &mut self.processes.table_state);
    }

    pub(super) fn handle_key_event_processes(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.processes.table_state.select(None);
            }
            KeyCode::Down => {
                if let Some(selected) = self.processes.table_state.selected() {
                    let next = (selected + 1) % self.processes.len();
                    self.processes.table_state.select(Some(next));
                } else {
                    self.processes.table_state.select(Some(0));
                }
            }
            KeyCode::Up => {
                if let Some(selected) = self.processes.table_state.selected() {
                    let prev = if selected == 0 {
                        self.processes.len() - 1
                    } else {
                        selected - 1
                    };
                    self.processes.table_state.select(Some(prev));
                } else {
                    self.processes.table_state.select(Some(0));
                }
            }
            KeyCode::PageDown => {
                if let Some(selected) = self.processes.table_state.selected() {
                    let next = (selected + 10) % self.processes.len();
                    self.processes.table_state.select(Some(next));
                } else {
                    self.processes.table_state.select(Some(0));
                }
            }
            KeyCode::PageUp => {
                if let Some(selected) = self.processes.table_state.selected() {
                    let next = (selected - 10) % self.processes.len();
                    self.processes.table_state.select(Some(next));
                } else {
                    self.processes.table_state.select(Some(0));
                }
            }
            _ => {}
        }
    }
}
