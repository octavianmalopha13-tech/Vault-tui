mod app;
mod ui;


use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, AppState};

fn main() -> io::Result<()> {
    // ---- terminal setup ----
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Restore the terminal on ANY exit path — normal return, `?`, or panic.
    // Without this, a panic leaves the shell in raw mode and it looks broken.
    let _guard = TerminalGuard;

    // ---- app + loop ----
    let vault_path=std::env::args()
       .nth(1)
       .unwrap_or_else(||"vault.enc".into());
    let mut app = App::new(vault_path);
    run(&mut terminal, &mut app)
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        // render
        terminal.draw(|f| ui::draw(f, app))?;

        // poll for input (100ms tick lets us redraw periodically if needed)
        
    if event::poll(Duration::from_millis(100))?
        && let Event::Key(key) = event::read()?
        && key.kind == KeyEventKind::Press
    {
        app.handle_key(key);
    }


        if app.state == AppState::Quit {
            return Ok(());
        }
    }
}
