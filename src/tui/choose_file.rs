use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Layout;
use ratatui::{layout::Rect, style::Stylize, Frame};
use ratatui::{
    layout::{Constraint, Flex},
    style::Style,
    symbols,
    text::Line,
    widgets::{Block, List},
};
use std::fs;

impl super::App {
    pub(super) fn render_choose_files(
        &mut self,
        frame: &mut Frame,
        mut choose_file: super::ChooseFile,
    ) -> super::ChooseFile {
        let area = center(frame.area(), Constraint::Length(80), Constraint::Length(20));

        let table = List::new(choose_file.files.clone())
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

        frame.render_stateful_widget(table, area, &mut choose_file.wstate);
        choose_file
    }

    pub(super) fn handle_key_event_choose_file(
        &mut self,
        key_event: KeyEvent,
        mut choose_file: super::ChooseFile,
    ) -> super::ChooseFile {
        match key_event.code {
            // KeyCode::Esc => {
            //     choose_file.wstate.select(None);
            // }
            KeyCode::Down => {
                if let Some(selected) = choose_file.wstate.selected() {
                    let next = (selected + 1) % choose_file.len();
                    choose_file.wstate.select(Some(next));
                } else {
                    choose_file.wstate.select(Some(0));
                }
            }
            KeyCode::Up => {
                if let Some(selected) = choose_file.wstate.selected() {
                    let prev = if selected == 0 {
                        choose_file.len() - 1
                    } else {
                        selected - 1
                    };
                    choose_file.wstate.select(Some(prev));
                } else {
                    choose_file.wstate.select(Some(0));
                }
            }
            KeyCode::Enter => {
                if let Some(selected) = choose_file.wstate.selected() {
                    let selected_file = choose_file.files[selected].clone();
                    self.full_config_filename = Some(std::path::PathBuf::from(selected_file));
                }
            }
            _ => {}
        }
        choose_file
    }
}

pub(super) fn available_files() -> Vec<String> {
    let files = match fs::read_dir("/tmp/procman") {
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
            .map(|file| get_cfg_file_from_running(&file))
            .collect(),
        Err(_) => vec![],
    };
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
    // fn fill(mut self) -> Self {
    //     self.files = available_files();
    //     self
    // }
}

fn get_cfg_file_from_running(selected_file: &str) -> String {
    let selected_file = format!("/tmp/procman/{}", selected_file);
    let content = fs::read_to_string(selected_file).unwrap_or_else(|_| String::new());
    let original_file_full_path = content
        .lines()
        .find(|line| line.starts_with("original_file_full_path"))
        .and_then(|line| line.split('=').nth(1))
        .map(|value| value.trim().trim_matches('"'))
        .unwrap_or("unknown");

    original_file_full_path.to_string()
}
