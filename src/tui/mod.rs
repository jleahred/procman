//  experimental!!!
mod choose_file;
mod processes;

use std::collections::BTreeMap;
use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use ratatui::widgets::ListState;
use ratatui::{backend::CrosstermBackend, Terminal};
use ratatui::{
    layout::Margin,
    style::Stylize,
    symbols::{self},
    text::Line,
    widgets::{Block, TableState},
    DefaultTerminal, Frame,
};

use crate::types::config::{ProcessConfig, ProcessId};
use crate::watch_now::WatchNow;
use crate::Config;

// #[derive(Debug)]
struct App {
    cfg_file_name: String,
    processes: Processes,
    choose_file: ChooseFile,
    exit: bool,
    debug: Option<String>,
}

pub(crate) fn run(cfg_file_name: &str) -> io::Result<()> {
    //     let stdout = io::stdout(); //  todo:
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    crossterm::terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let processes = Processes::create(&cfg_file_name, TableState::default())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut app = App {
        cfg_file_name: cfg_file_name.to_string(),
        processes,
        choose_file: ChooseFile {
            files: choose_file::available_files(),
            wstate: ListState::default(),
        },
        exit: false,
        debug: None,
    };

    let app_result = app.run(&mut terminal);

    ratatui::restore();

    app_result
}

impl App {
    /// runs the application's main loop until the user quits
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        use std::time::{Duration, Instant};
        let mut last_update = Instant::now();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(Duration::from_secs(2))?;

            // Actualiza self.processes cada 2 segundos
            if last_update.elapsed() >= Duration::from_secs(2) {
                self.processes =
                    Processes::create(&self.cfg_file_name, self.processes.table_state.clone())
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                last_update = Instant::now();
                self.choose_file.files = choose_file::available_files();
            }

            // Pequeño sleep para evitar uso excesivo de CPU
            // std::thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let title = Line::from(format!("  [ {} ]  ", self.cfg_file_name)).centered();
        let bottom = Line::from(vec![
            " Quit ".into(),
            "<Q> ".blue().bold(),
            "<Ctrl-c> ".blue().bold(),
            // "<Esc> ".blue().bold(),
        ])
        .centered();
        frame.render_widget(
            Block::bordered()
                .border_set(symbols::border::ROUNDED)
                .title(title)
                .title_bottom(bottom),
            frame.area(),
        );

        //self.render_processes(frame, frame.area().inner(Margin::new(1, 1)));
        self.render_choose_files(frame);
    }

    fn handle_events(&mut self, timeout: Duration) -> io::Result<()> {
        match event::poll(timeout)? {
            true => {
                match event::read()? {
                    // it's important to check that the event is a key press event as
                    // crossterm also emits key release and repeat events on Windows.
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        self.handle_key_event_main(key_event)
                    }
                    _ => {}
                };
            }
            false => {}
        }
        Ok(())
    }

    fn handle_key_event_main(&mut self, key_event: KeyEvent) {
        match key_event.code {
            // KeyCode::Esc => self.exit = true,
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('c') if key_event.modifiers.contains(event::KeyModifiers::CONTROL) => {
                self.exit = true
            }
            KeyCode::Char('d') => {
                if self.debug.is_some() {
                    self.debug = None
                } else {
                    self.debug = Some(format!("{:#?}", self.processes.watched))
                }
            }
            _ => {
                // self.handle_key_event_processes(key_event),
                self.handle_key_event_choose_file(key_event);
            }
        }
    }
}

impl Processes {
    pub(crate) fn create(
        full_config_filename: &str,
        table_state: TableState,
    ) -> Result<Self, String> {
        let watched = WatchNow::create(full_config_filename)?;
        let mut only_in_config = BTreeMap::new();

        let config: Config =
            Config::read_from_file(full_config_filename).map_err(|e| e.0.to_string())?;
        for process_config in config.process.iter() {
            if !watched.processes.contains_key(&process_config.id) {
                only_in_config
                    .entry(process_config.id.clone())
                    .or_insert_with(|| process_config.clone());
            }
        }
        Ok(Self {
            watched,
            only_in_config,
            table_state,
        })
    }
}

// #[derive(Clone)]
pub(super) struct Processes {
    watched: WatchNow,
    pub(crate) only_in_config: BTreeMap<ProcessId, ProcessConfig>,

    table_state: TableState,
}

impl Processes {
    fn len(&self) -> usize {
        self.watched.processes.len() + self.only_in_config.len()
    }
}

struct ChooseFile {
    files: Vec<String>,
    wstate: ListState,
}
