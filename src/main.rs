mod app;
mod audio;
mod ui;
mod metadata;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    
    let path = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    app.scan_directory(&path);

    let tick_rate = Duration::from_millis(16);

    while !app.quit {
        app.update_visualizer();
        terminal.draw(|f| ui::render(f, &mut app))?;

        if app.player.player.empty() && app.current_track_name != "Ninguna pista cargada" {
            if app.loop_track {
                let _ = app.play_selected();
            } else {
                let _ = app.play_next();
            }
        }

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => app.quit = true,
                        KeyCode::Char(' ') => app.player.toggle_playback(),
                        KeyCode::Up => app.previous(),
                        KeyCode::Down => app.next(),
                        KeyCode::Enter => {
                            let _ = app.play_selected();
                        }
                        KeyCode::Char('n') | KeyCode::Right => {
                            let _ = app.play_next();
                        }
                        KeyCode::Char('p') | KeyCode::Left => {
                            let _ = app.play_previous();
                        }
                        KeyCode::Char('l') => app.toggle_loop(),
                        KeyCode::Char('v') => app.toggle_visualizer(),
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
