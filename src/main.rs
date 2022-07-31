use crossterm::{
    cursor::Hide, cursor::Show, event::read, event::Event, event::KeyCode, event::KeyEvent,
    execute, style::Print, terminal, terminal::EnterAlternateScreen,
    terminal::LeaveAlternateScreen, Result,
};

fn main() -> Result<()> {
    terminal::enable_raw_mode()?;
    execute!(std::io::stderr(), Hide, EnterAlternateScreen)?;
    loop {
        match read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => break,
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                ..
            }) => {
                execute!(std::io::stderr(), Print("a"))?;
            }
            _ => continue,
        }
    }
    execute!(std::io::stderr(), Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    return Ok(());
}
