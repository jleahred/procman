use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Layout;
use ratatui::widgets::ListState;
use ratatui::{layout::Rect, style::Stylize, Frame};
use ratatui::{
    layout::{Constraint, Flex},
    style::Style,
    symbols,
    text::Line,
    widgets::{Block, List},
};
use std::fs;

pub(super) struct ChooseFileState {
    files: Vec<String>,
    pub(super) wstate: ListState,
}

impl Default for ChooseFileState {
    fn default() -> Self {
        let mut result = Self {
            files: available_files(),
            wstate: ListState::default(),
        };
        result.wstate.select(Some(0));
        result
    }
}

pub(super) fn render(frame: &mut Frame, choose_file: &mut ChooseFileState) {
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
}

pub(super) fn handle_key_event(
    key_event: KeyEvent,
    choose_file: &mut ChooseFileState,
) -> super::Command {
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
            super::Command::None
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
            super::Command::None
        }
        KeyCode::Enter => {
            if let Some(selected) = choose_file.wstate.selected() {
                let selected_file = choose_file.files[selected].clone();
                super::Command::ChoosedFile(std::path::PathBuf::from(selected_file))
            } else {
                super::Command::None
            }
        }
        _ => super::Command::None,
    }
}

// ------------------

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

impl ChooseFileState {
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
