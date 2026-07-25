// Terminal backend — init/restore + capability detection
//
// Cross-platform core: consistent on Windows/Linux/macOS.

use std::io::{self, Stdout};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

/// TUI terminal type alias
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize terminal (Windows / Linux / macOS)
pub fn init() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restore terminal (called on exit)
pub fn restore(terminal: &mut Tui) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Detect underline color support
///
/// Legacy conhost (CMD) doesn't support underline-color.
/// Windows Terminal (WT_SESSION) and VS Code (TERM_PROGRAM=vscode) support it.
pub fn supports_underline_color() -> bool {
    if cfg!(windows) {
        std::env::var("WT_SESSION").is_ok()
            || std::env::var("TERM_PROGRAM").is_ok_and(|v| v == "vscode")
    } else {
        true
    }
}

/// Install panic hook — restore terminal on panic
///
/// TUI crash without raw mode restore makes terminal unusable.
pub fn setup_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // best-effort terminal restore
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
        // call default hook to print panic info
        default_hook(panic_info);
    }));
}
