use crate::types::running_status::ProcessStatus;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Layout;
use ratatui::{layout::Rect, style::Stylize, widgets::HighlightSpacing, Frame};
use ratatui::{
    layout::{Constraint, Flex},
    style::{Color, Modifier, Style},
    symbols,
    text::Line,
    widgets::{Block, Cell, List, ListDirection, Row, Table},
};
use std::fs;

impl super::App {
    pub(super) fn render_choose_files(&mut self, frame: &mut Frame) {
        let area = center(frame.area(), Constraint::Length(80), Constraint::Length(20));

        let table = List::new(self.choose_file.files.clone())
            .block(
                Block::bordered()
                    .title(Line::from(" Choose file: ").centered())
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_set(symbols::border::DOUBLE),
            )
            .highlight_style(Style::new().reversed())
            // .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            //.direction(ListDirection::BottomToTop)
            ;

        frame.render_stateful_widget(table, area, &mut self.choose_file.wstate);
    }

    pub(super) fn handle_key_event_choose_file(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.choose_file.wstate.select(None);
            }
            KeyCode::Down => {
                if let Some(selected) = self.choose_file.wstate.selected() {
                    let next = (selected + 1) % self.choose_file.len();
                    self.choose_file.wstate.select(Some(next));
                } else {
                    self.choose_file.wstate.select(Some(0));
                }
            }
            KeyCode::Up => {
                if let Some(selected) = self.choose_file.wstate.selected() {
                    let prev = if selected == 0 {
                        self.choose_file.len() - 1
                    } else {
                        selected - 1
                    };
                    self.choose_file.wstate.select(Some(prev));
                } else {
                    self.choose_file.wstate.select(Some(0));
                }
            }
            _ => {}
        }
    }
}

pub(super) fn available_files() -> Vec<String> {
    let mut files = match fs::read_dir("/tmp/procman") {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .map(String::from)
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => vec![],
    };
    files.push("default.toml".to_string());
    files.push("default.toml".to_string());
    files.push("default.toml".to_string());
    files.push("default.toml".to_string());
    files.push("default.toml".to_string());
    files
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

impl super::ChooseFile {
    fn len(&self) -> usize {
        self.files.len()
    }
    fn fill(mut self) -> Self {
        self.files = available_files();
        self
    }
}
