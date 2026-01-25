//! Error handling and panic hook setup.

use std::io;

/// Set up a panic hook that restores terminal state before panicking.
///
/// This ensures the terminal is usable even if the application panics.
pub fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal state
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            io::stderr(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );
        original_hook(panic_info);
    }));
}

/// Initialize error handling with color-eyre.
pub fn init() -> color_eyre::Result<()> {
    color_eyre::install()?;
    setup_panic_hook();
    Ok(())
}
