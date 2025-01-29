use std::io::{self, stderr, Stderr};

use crossterm::{execute, terminal::*};
use ratatui::prelude::*;

/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stderr>>;

/// Initialize the terminal
pub fn init() -> io::Result<Tui> {
    execute!(stderr(), EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stderr()))
}

/// Restore the terminal to its original state
pub fn restore() -> io::Result<()> {
    execute!(stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
