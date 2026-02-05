mod app;
mod ui;
mod tree;
mod terminal;
mod event;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::path::PathBuf;

use app::App;
use event::EventHandler;

#[derive(Parser, Debug)]
#[command(name = "claude-explorer")]
#[command(author, version, about = "A TUI file explorer for Claude Code CLI", long_about = None)]
struct Args {
    /// Working directory path
    #[arg(short, long, default_value = ".")]
    path: PathBuf,

    /// Tree panel width percentage (10-50)
    #[arg(short = 'w', long, default_value = "30")]
    tree_width: u16,

    /// Show hidden files
    #[arg(short = 'a', long)]
    show_hidden: bool,

    /// Max tree depth
    #[arg(short, long, default_value = "10")]
    depth: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(args.path, args.tree_width, args.show_hidden, args.depth)?;

    // Create event handler
    let event_handler = EventHandler::new(250);

    // Run the app
    let result = run_app(&mut terminal, &mut app, event_handler).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    mut event_handler: EventHandler,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Handle events
        match event_handler.next().await? {
            event::Event::Tick => {
                app.tick();
            }
            event::Event::Key(key_event) => {
                if app.handle_key(key_event) {
                    return Ok(());
                }
            }
            event::Event::Mouse(mouse_event) => {
                app.handle_mouse(mouse_event);
            }
            event::Event::Resize(width, height) => {
                app.handle_resize(width, height);
            }
            event::Event::FileChange(path) => {
                app.handle_file_change(path);
            }
        }
    }
}
