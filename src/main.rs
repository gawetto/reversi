use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, SetBackgroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};

#[derive(Copy, Clone, PartialEq)]
enum Masu {
    Empty,
    Black,
    White,
}

fn main() -> Result<()> {
    let mut field = [[Masu::Empty; 8]; 8];
    let mut cursor = (0, 0);
    enable_raw_mode()?;
    execute!(std::io::stderr(), Hide, EnterAlternateScreen)?;
    loop {
        execute!(std::io::stderr(), MoveTo(0, 0),)?;
        for i in 0..8 {
            for j in 0..8 {
                if i == cursor.0 && j == cursor.1 {
                    execute!(std::io::stderr(), SetBackgroundColor(Color::Grey))?;
                } else {
                    execute!(std::io::stderr(), SetBackgroundColor(Color::DarkGreen))?;
                }
                match field[i][j] {
                    Masu::Empty => {
                        execute!(std::io::stderr(), Print('　'))?;
                    }
                    Masu::Black => {
                        execute!(std::io::stderr(), Print('⚫'))?;
                    }
                    Masu::White => {
                        execute!(std::io::stderr(), Print('⚪'))?;
                    }
                }
            }
            execute!(std::io::stderr(), Print("\n"))?;
        }
        match read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => break,
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            }) => {
                if cursor.1 != 0 {
                    cursor.1 -= 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) => {
                if cursor.0 != 0 {
                    cursor.0 -= 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            }) => {
                if cursor.1 != 7 {
                    cursor.1 += 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) => {
                if cursor.0 != 7 {
                    cursor.0 += 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('w'),
                ..
            }) => {
                field[cursor.0][cursor.1] = Masu::White;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('b'),
                ..
            }) => {
                field[cursor.0][cursor.1] = Masu::Black;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) => {
                field[cursor.0][cursor.1] = Masu::Empty;
            }
            _ => continue,
        }
    }
    execute!(std::io::stderr(), Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    return Ok(());
}
