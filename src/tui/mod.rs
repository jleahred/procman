//  experimental!!!
mod choose_file;
mod processes;

use choose_file::ChooseFileState;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, EnterAlternateScreen},
};
use once_cell::sync::Lazy;
use processes::Processes;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::Block,
    DefaultTerminal, Frame, Terminal,
};
use std::io;
use std::sync::Mutex;
use std::time::{Duration, Instant};

macro_rules! handle_events {
    (  $( $cmd:expr ),+ $(,)? ) => {{
        let mut commands = Vec::<Command>::new();
        $(
            commands.push($cmd);
        )+
        commands
    }};
}

static LAST_UPDATE: Lazy<Mutex<Instant>> = Lazy::new(|| Mutex::new(Instant::now()));
const UPDATE_INTERVAL: Duration = Duration::from_millis(500);

struct App {
    processes: Option<Processes>,
    choose_file: Option<ChooseFileState>,
    exit: bool,
}

enum Command {
    None,
    ChooseFile,
    ChoosedFile(std::path::PathBuf),
}

pub(crate) fn run() -> io::Result<()> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    crossterm::terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let mut app = App {
        processes: None,
        choose_file: Some(ChooseFileState::default()),
        exit: false,
    };

    let app_result = app.run(&mut terminal);
    disable_raw_mode()?;
    ratatui::restore();

    app_result
}

impl App {
    /// runs the application's main loop until the user quits
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            self.handle_events(Duration::from_secs(2))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            update_info_app(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [main, bottom] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
            // .flex(Flex::Center)
            .areas(frame.area());

        draw_main(&self, frame, bottom);

        self.processes
            .as_mut()
            .map(|processes| processes.render(frame, main));
        self.choose_file.as_mut().map(|w| w.render(frame, main));
    }

    fn handle_events(&mut self, timeout: Duration) -> Result<(), String> {
        match event::poll(timeout).map_err(|e| e.to_string())? {
            true => {
                let event = event::read().map_err(|e| e.to_string())?;

                let commands = handle_events! {
                    self.handle_events_main(&event),
                    // self.choose_file.handle_events(&event),
                    self.choose_file.as_mut()
                    .map_or(Command::None, |w| w.handle_events(&event)),
                    self.processes
                        .as_mut()
                        .map_or(Command::None, |w| w.handle_events(&event))
                };
                // let commands = handle_events2! {
                //     &event,
                //     self, App::handle_events_main,
                //     &mut self.choose_file, ChooseFileState::handle_events,
                // };

                self.process_commands(&commands)?;
            }
            false => {}
        }
        Ok(())
    }

    fn handle_events_main(&mut self, event: &Event) -> Command {
        match event {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                handle_key_event_main(self, &key_event)
            }
            _ => {}
        };
        Command::None
    }

    fn process_commands(&mut self, commands: &[Command]) -> Result<(), String> {
        for command in commands {
            match command {
                Command::None => {}
                Command::ChooseFile => {
                    self.choose_file = Some(ChooseFileState::default());
                    self.processes = None;
                }
                Command::ChoosedFile(path) => {
                    self.processes = Some(Processes::create(&path)?);
                    self.choose_file = None;
                }
            }
        }
        Ok(())
    }
}

fn draw_main(_app: &App, frame: &mut Frame, area: Rect) {
    let bottom = Line::from(vec![
        " Quit ".into(),
        "<Q> ".blue().bold(),
        "<Ctrl-c> ".blue().bold(),
    ])
    .centered();
    frame.render_widget(
        Block::new()
            // Block::bordered()
            // .border_set(symbols::border::ROUNDED)
            // .title(title)
            .title_bottom(bottom),
        area,
    );
}

fn handle_key_event_main(app: &mut App, key_event: &KeyEvent) {
    match key_event.code {
        // KeyCode::Esc => self.exit = true,
        KeyCode::Char('q') => app.exit = true,
        KeyCode::Char('c') if key_event.modifiers.contains(event::KeyModifiers::CONTROL) => {
            app.exit = true
        }
        _ => {}
    }
}

fn update_info_app(app: &mut App) -> Result<(), String> {
    let mut last_update = LAST_UPDATE
        .lock()
        .map_err(|_| "Failed to lock LAST_UPDATE")?;

    if last_update.elapsed() >= UPDATE_INTERVAL {
        app.processes
            .as_mut()
            .and_then(|processes| Some(processes.update_data()));
        app.choose_file
            .as_mut()
            .and_then(|cf| Some(cf.update_data()));

        *last_update = Instant::now();
    }
    Ok(())
}

// -------------------------
