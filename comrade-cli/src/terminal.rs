use std::io;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use tui::backend::CrosstermBackend;
use tui::Terminal;

use crate::errors::TerminalError;

type Result<T, E = TerminalError> = core::result::Result<T, E>;

pub(crate) type ComradeTerminal = Terminal<CrosstermBackend<io::Stdout>>;

pub(crate) fn setup_terminal() -> Result<ComradeTerminal> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

pub(crate) fn restore_terminal(mut term: ComradeTerminal) -> Result<()> {
    disable_raw_mode()?;

    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()?;

    Ok(())
}
