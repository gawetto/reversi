use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};

use reversi_core::*;

fn input(
    event: Event,
    field: &mut Field,
    cursor: &mut Position,
    end: &mut bool,
    turn: &mut BorW,
) -> Result<()> {
    match event {
        Event::Key(KeyEvent {
            code: KeyCode::Esc, ..
        }) => *end = true,
        Event::Key(KeyEvent {
            code: KeyCode::Char('p'),
            ..
        }) => *turn = get_another_color(*turn),
        Event::Key(KeyEvent {
            code: KeyCode::Char('r'),
            ..
        }) => (*field, *cursor, *turn) = create_initial_data(),
        Event::Key(KeyEvent {
            code: KeyCode::Left,
            ..
        }) => {
            *cursor = cursor.left().unwrap_or(*cursor);
        }
        Event::Key(KeyEvent {
            code: KeyCode::Up, ..
        }) => {
            *cursor = cursor.up().unwrap_or(*cursor);
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            ..
        }) => {
            *cursor = cursor.right().unwrap_or(*cursor);
        }
        Event::Key(KeyEvent {
            code: KeyCode::Down,
            ..
        }) => {
            *cursor = cursor.down().unwrap_or(*cursor);
        }
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            ..
        }) => {
            if check_putable(&field, *cursor, *turn) {
                field.set(*cursor, Masu::Putted(*turn));
                auto_reverse(field, *cursor, *turn);
                *turn = get_another_color(*turn);
                if !field.puttable(*turn) {
                    *turn = get_another_color(*turn);
                }
            }
        }
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            ..
        }) => {
            field.set(*cursor, Masu::Empty);
        }
        _ => {}
    }
    return Ok(());
}
fn view<T: std::io::Write>(
    output: &mut T,
    field: &Field,
    cursor: Position,
    turn: BorW,
) -> Result<()> {
    execute!(output, MoveTo(0, 0),)?;
    match turn {
        BorW::Black => {
            execute!(output, SetForegroundColor(Color::Black))?;
        }
        BorW::White => {
            execute!(output, SetForegroundColor(Color::White))?;
        }
    }
    for i in 0..8 {
        for j in 0..8 {
            let p = Position::new(i, j).unwrap();
            if cursor.eq(&p) {
                execute!(output, SetBackgroundColor(Color::Grey))?;
            } else {
                if (i + j) % 2 == 0 {
                    execute!(output, SetBackgroundColor(Color::DarkGreen))?;
                } else {
                    execute!(output, SetBackgroundColor(Color::Green))?;
                }
            }
            match field.get(p) {
                Masu::Empty => {
                    if check_putable(field, p, turn) {
                        execute!(output, Print('・'))?;
                    } else {
                        execute!(output, Print('　'))?;
                    }
                }
                Masu::Putted(BorW::Black) => {
                    execute!(output, Print('⚫'))?;
                }
                Masu::Putted(BorW::White) => {
                    execute!(output, Print('⚪'))?;
                }
            }
        }
        execute!(output, Print("\n"))?;
    }
    execute!(output, ResetColor)?;
    match turn {
        BorW::Black => {
            execute!(output, Print("Black Turn\n"))?;
        }
        BorW::White => {
            execute!(output, Print("White Turn\n"))?;
        }
    }
    execute!(
        output,
        Print(format!(
            "⚫:{:>2} ⚪:{:>2}\n",
            field.count(BorW::Black),
            field.count(BorW::White),
        ))
    )?;
    match field.get_gameresult() {
        GameResult::Playing => {
            execute!(output, Print(format!("                    ",)))?;
        }
        GameResult::Draw => {
            execute!(output, Print(format!("Game Result is Draw ",)))?;
        }
        GameResult::Win(BorW::Black) => {
            execute!(output, Print(format!("Winner is Black      ")))?;
        }
        GameResult::Win(BorW::White) => {
            execute!(output, Print(format!("Winner is White      ")))?;
        }
    }
    return Ok(());
}

fn main() -> Result<()> {
    let (mut field, mut cursor, mut turn) = create_initial_data();
    let mut end = false;
    enable_raw_mode()?;
    execute!(std::io::stderr(), Hide, EnterAlternateScreen)?;
    while !end {
        view(&mut std::io::stderr(), &field, cursor, turn)?;
        input(read()?, &mut field, &mut cursor, &mut end, &mut turn)?;
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
        let mut field = Field::new();
        let mut cursor = Position::new(4, 2).unwrap();
        let mut end = false;
        let mut turn = BorW::Black;
        let enterkey = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        super::input(enterkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(field.get(Position::new(4, 2).unwrap()) == Masu::Putted(BorW::Black));
        assert!(turn == BorW::White);
        let pkey = Event::Key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE));
        super::input(pkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(turn == BorW::Black);
        super::input(pkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(turn == BorW::White);
        let rightkey = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        super::input(rightkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(cursor.x() == 4);
        assert!(cursor.y() == 3);
        let downkey = Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        super::input(downkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(cursor.x() == 5);
        assert!(cursor.y() == 3);
        let leftkey = Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        super::input(leftkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(cursor.x() == 5);
        assert!(cursor.y() == 2);
        let upkey = Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        super::input(upkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(cursor.x() == 4);
        assert!(cursor.y() == 2);
        let backspace = Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        super::input(backspace, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(field.get(Position::new(0, 0).unwrap()) == Masu::Empty);
        let esc = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        super::input(esc, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(end);
    }
    #[test]
    fn view_test() {
        let field = Field::new();
        let cursor = Position::new(0, 0).unwrap();
        let turn = BorW::Black;
        let mut buf = Vec::<u8>::new();
        let mut assert_buf = Vec::<u8>::new();
        super::view(&mut buf, &field, cursor, turn).unwrap();
        //let mut f = File::create("testdata/initview").unwrap();
        //use std::io::Write;
        //f.write_all(buf.into_boxed_slice().as_ref()).unwrap();
        let mut f = File::open("testdata/initview").unwrap();
        f.read_to_end(&mut assert_buf).unwrap();
        assert!(buf == assert_buf);
    }
}
