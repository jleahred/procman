use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, List, ListState},
    Frame,
};
use std::fs;

pub(super) struct ChooseFileState {
    files: Vec<String>,
    wstate: ListState,
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

impl ChooseFileState {
    pub(super) fn update_data(&mut self) {
        self.files = available_files();
        if self.files.is_empty() {
            self.wstate.select(None);
        } else if self.wstate.selected().is_none() {
            self.wstate.select(Some(0));
        }
    }

    pub(super) fn render(&mut self, frame: &mut Frame, area: Rect) {
        let area = center(area, Constraint::Length(80), Constraint::Length(20));

        let table = List::new(self.files.clone())
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

        frame.render_stateful_widget(table, area, &mut self.wstate);
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

//  -------------

fn handle_key_event(key_event: &KeyEvent, choose_file: &mut ChooseFileState) -> super::Command {
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

fn available_files() -> Vec<String> {
    let files = match fs::read_dir("/tmp/procman") {
        Ok(entries) => {
            let mut result = entries
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
                .collect::<Vec<String>>();
            result.sort();
            result
        }
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
