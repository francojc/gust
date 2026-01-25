//! gust - A terminal-based weather dashboard.

mod app;
mod config;
mod error;

mod api;
mod cache;
mod ui;

use std::io;
use std::time::Duration;

use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::{AppState, Message};
use config::AppConfig;

fn main() -> Result<()> {
    // Initialize error handling
    error::init()?;

    // Load configuration
    let config = AppConfig::load()?;

    // Initialize terminal
    let mut terminal = setup_terminal()?;

    // Run the application
    let result = run(&mut terminal, config);

    // Restore terminal state
    restore_terminal(&mut terminal)?;

    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, config: AppConfig) -> Result<()> {
    let mut state = AppState::new(config);

    loop {
        // Render
        terminal.draw(|f| state.view(f))?;

        // Handle events with timeout for tick
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events (not release)
                if key.kind == KeyEventKind::Press {
                    state.update(Message::Input(key));
                }
            }
        } else {
            // Tick for auto-refresh check
            state.update(Message::Tick);
        }

        if state.should_quit {
            break;
        }
    }

    Ok(())
}
