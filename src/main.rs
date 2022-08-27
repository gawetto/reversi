use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};

#[derive(Copy, Clone, PartialEq)]
enum Masu {
    Empty,
    Black,
    White,
}

fn input(
    event: Event,
    field: &mut [[Masu; 8]; 8],
    cursor: &mut (usize, usize),
    end: &mut bool,
) -> Result<()> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Esc, ..
        }) => *end = true,
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
        _ => {}
    }
    return Ok(());
}
fn view<T: std::io::Write>(
    output: &mut T,
    field: &[[Masu; 8]; 8],
    cursor: &(usize, usize),
) -> Result<()> {
    execute!(output, MoveTo(0, 0),)?;
    for i in 0..8 {
        for j in 0..8 {
            if i == cursor.0 && j == cursor.1 {
                execute!(output, SetBackgroundColor(Color::Grey))?;
            } else {
                execute!(output, SetBackgroundColor(Color::DarkGreen))?;
            }
            match field[i][j] {
                Masu::Empty => {
                    execute!(output, Print('　'))?;
                }
                Masu::Black => {
                    execute!(output, Print('⚫'))?;
                }
                Masu::White => {
                    execute!(output, Print('⚪'))?;
                }
            }
        }
        execute!(output, Print("\n"))?;
    }
    execute!(output, ResetColor)?;
    return Ok(());
}

fn main() -> Result<()> {
    let mut field = [[Masu::Empty; 8]; 8];
    let mut cursor = (0, 0);
    let mut end = false;
    enable_raw_mode()?;
    execute!(std::io::stderr(), Hide, EnterAlternateScreen)?;
    while !end {
        view(&mut std::io::stderr(), &field, &cursor)?;
        input(read()?, &mut field, &mut cursor, &mut end)?;
    }
    execute!(std::io::stderr(), Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use std::fs::File;
    use std::io::Read;
    #[test]
    fn input_test() {
        let mut field = [[Masu::Empty; 8]; 8];
        let mut cursor = (0, 0);
        let mut end = false;
        let wkey = Event::Key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE));
        super::input(wkey, &mut field, &mut cursor, &mut end).unwrap();
        assert!(field[0][0] == Masu::White);
        let rightkey = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        super::input(rightkey, &mut field, &mut cursor, &mut end).unwrap();
        assert!(cursor.0 == 0);
        assert!(cursor.1 == 1);
        let downkey = Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        super::input(downkey, &mut field, &mut cursor, &mut end).unwrap();
        assert!(cursor.0 == 1);
        assert!(cursor.1 == 1);
        let bkey = Event::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
        super::input(bkey, &mut field, &mut cursor, &mut end).unwrap();
        assert!(field[1][1] == Masu::Black);
        let leftkey = Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        super::input(leftkey, &mut field, &mut cursor, &mut end).unwrap();
        assert!(cursor.0 == 1);
        assert!(cursor.1 == 0);
        let upkey = Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        super::input(upkey, &mut field, &mut cursor, &mut end).unwrap();
        assert!(cursor.0 == 0);
        assert!(cursor.1 == 0);
        let backspace = Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        super::input(backspace, &mut field, &mut cursor, &mut end).unwrap();
        assert!(field[0][0] == Masu::Empty);
        let esc = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        super::input(esc, &mut field, &mut cursor, &mut end).unwrap();
        assert!(end);
    }
    #[test]
    fn view_test() {
        let mut field = [[Masu::Empty; 8]; 8];
        let cursor = (0, 0);
        field[3][3] = Masu::Black;
        field[4][4] = Masu::Black;
        field[3][4] = Masu::White;
        field[4][3] = Masu::White;
        let mut buf = Vec::<u8>::new();
        let mut assert_buf = Vec::<u8>::new();
        super::view(&mut buf, &field, &cursor).unwrap();
        //let mut f = File::create("testdata/initview").unwrap();
        //use std::io::Write;
        //f.write_all(buf.into_boxed_slice().as_ref()).unwrap();
        let mut f = File::open("testdata/initview").unwrap();
        f.read_to_end(&mut assert_buf).unwrap();
        assert!(buf == assert_buf);
    }
}
