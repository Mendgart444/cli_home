use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, List, ListItem, ListState},
};

use std::process::Command;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,

    // update sucsess?
    last_status: Vec<RunStatus>,

    // menu items vec 
    menu_items: Vec<String>,
    selected: usize,
}

#[derive(Debug, Clone)]
enum RunStatus {
    Never,
    Success,
    Failed(String),
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self { 
            running: true, 
            menu_items: vec![
                "Check for updates".into(),
                "Weather".into(),
                "Check Repo (Git only)".into(),
                "quit".into(),
            ], 
            selected: 0,
            last_status: vec![RunStatus::Never; 4],
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events().await?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, frame: &mut Frame) {
        // title
        let title = Line::from("CLI Home")
            .bold()
            .blue()
            .centered();

        // list with options
        let items: Vec<ListItem> = self
            .menu_items
            .iter()
            .zip(&self.last_status)
            .enumerate()
            .map(|(i, (name, status))| {
                let status_text = match status {
                RunStatus::Never => " (never run)".gray(),
                RunStatus::Success => " (success)".green().bold(),
                RunStatus::Failed(message) => format!(" (failed: {})", message).red().bold(),
            };

            if i == self.menu_items.len() - 1 {
                ListItem::new(name.as_str())
            } else {
                ListItem::new(Line::from(vec![
                    name.clone().into(),
                    status_text,
                ]))
            }
            })
            .collect();
        
        let mut state = ListState::default();
        state.select(Some(self.selected)); // improtaint

        let list = List::new(items)
            .block(Block::bordered().title(title))
            .highlight_style(Style::new().yellow().bold())
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, frame.area(), &mut state);
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    async fn handle_crossterm_events(&mut self) -> color_eyre::Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key).await,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    async fn on_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.quit(),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => self.quit(),

            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down => {
            if self.selected < self.menu_items.len() - 1 {
                self.selected += 1;
            }
        }

        KeyCode::Enter => {
            // Beispiel-Reaktionen:
            match self.selected {
                0 => self.check_for_updates(),
                1 => println!("Retrieve Weather info..."),
                2 => println!("Checking"),
                3 => self.quit(), // Beenden
                _ => {}
            }
        }
        _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }

    fn check_for_updates(&mut self) {
        match Command::new("sudo").args(["apt", "update"]).output() {
        Ok(output) => {
            if output.status.success() {
                self.last_status[0] = RunStatus::Success;
            } else {
                let err = String::from_utf8_lossy(&output.stderr);
                self.last_status[0] = RunStatus::Failed(err.lines().next().unwrap_or("unknown error").to_string());
            }
        }
        Err(e) => {
            self.last_status[0] = RunStatus::Failed(e.to_string());
        }
    }
    }
}
