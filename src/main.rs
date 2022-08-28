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
    Putted(BorW),
}

#[derive(Copy, Clone, PartialEq)]
enum BorW {
    Black,
    White,
}

fn get_another_color(color: BorW) -> BorW {
    match color {
        BorW::Black => BorW::White,
        BorW::White => BorW::Black,
    }
}

fn get_reversable(
    field: &[[Masu; 8]; 8],
    point: (usize, usize),
    color: BorW,
) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    let direction = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];
    for i in 0..direction.len() {
        let mut count = 0;
        let count = loop {
            count += 1;
            let x = point.0 as isize + direction[i].0 * count;
            if x < 0 || 8 <= x {
                break 0;
            }
            let y = point.1 as isize + direction[i].1 * count;
            if y < 0 || 8 <= y {
                break 0;
            }
            if field[x as usize][y as usize] == Masu::Empty {
                break 0;
            }
            if field[x as usize][y as usize] == Masu::Putted(color) {
                break count;
            }
        };
        for j in 1..count {
            let x = point.0 as isize + direction[i].0 * j;
            let y = point.1 as isize + direction[i].1 * j;
            result.push((x as usize, y as usize));
        }
    }
    return result;
}

fn check_putable(field: &[[Masu; 8]; 8], point: (usize, usize), turn: BorW) -> bool {
    if field[point.0][point.1] != Masu::Empty {
        return false;
    }
    if get_reversable(field, point, turn).len() == 0 {
        return false;
    }
    return true;
}

fn auto_reverse(field: &mut [[Masu; 8]; 8], point: (usize, usize), turn: BorW) {
    get_reversable(field, point, turn)
        .into_iter()
        .for_each(|x| field[x.0][x.1] = field[point.0][point.1]);
}

fn input(
    event: Event,
    field: &mut [[Masu; 8]; 8],
    cursor: &mut (usize, usize),
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
            code: KeyCode::Enter,
            ..
        }) => {
            if check_putable(&field, *cursor, *turn) {
                field[cursor.0][cursor.1] = Masu::Putted(*turn);
                auto_reverse(field, *cursor, *turn);
                *turn = get_another_color(*turn);
            }
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
    cursor: (usize, usize),
    turn: BorW,
) -> Result<()> {
    execute!(output, MoveTo(0, 0),)?;
    for i in 0..8 {
        for j in 0..8 {
            if i == cursor.0 && j == cursor.1 {
                execute!(output, SetBackgroundColor(Color::Grey))?;
            } else {
                if (i + j) % 2 == 0 {
                    execute!(output, SetBackgroundColor(Color::DarkGreen))?;
                } else {
                    execute!(output, SetBackgroundColor(Color::Green))?;
                }
            }
            match field[i][j] {
                Masu::Empty => {
                    execute!(output, Print('　'))?;
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
    return Ok(());
}

fn init_field(field: &mut [[Masu; 8]; 8]) {
    field[3][3] = Masu::Putted(BorW::Black);
    field[4][4] = Masu::Putted(BorW::Black);
    field[3][4] = Masu::Putted(BorW::White);
    field[4][3] = Masu::Putted(BorW::White);
}

fn main() -> Result<()> {
    let mut field = [[Masu::Empty; 8]; 8];
    let mut cursor = (0, 0);
    let mut end = false;
    let mut turn = BorW::Black;
    init_field(&mut field);
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
        let mut field = [[Masu::Empty; 8]; 8];
        init_field(&mut field);
        let mut cursor = (4, 2);
        let mut end = false;
        let mut turn = BorW::Black;
        let enterkey = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        super::input(enterkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(field[4][2] == Masu::Putted(BorW::Black));
        assert!(turn == BorW::White);
        let pkey = Event::Key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE));
        super::input(pkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(turn == BorW::Black);
        super::input(pkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(turn == BorW::White);
        let rightkey = Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        super::input(rightkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(cursor.0 == 4);
        assert!(cursor.1 == 3);
        let downkey = Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        super::input(downkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(cursor.0 == 5);
        assert!(cursor.1 == 3);
        let leftkey = Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        super::input(leftkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(cursor.0 == 5);
        assert!(cursor.1 == 2);
        let upkey = Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        super::input(upkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(cursor.0 == 4);
        assert!(cursor.1 == 2);
        let backspace = Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        super::input(backspace, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(field[0][0] == Masu::Empty);
        let esc = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        super::input(esc, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(end);
    }
    #[test]
    fn view_test() {
        let mut field = [[Masu::Empty; 8]; 8];
        let cursor = (0, 0);
        let turn = BorW::Black;
        field[3][3] = Masu::Putted(BorW::Black);
        field[4][4] = Masu::Putted(BorW::Black);
        field[3][4] = Masu::Putted(BorW::White);
        field[4][3] = Masu::Putted(BorW::White);
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
    #[test]
    fn init_field_test() {
        let mut field = [[Masu::Empty; 8]; 8];
        init_field(&mut field);
        assert!(field[3][3] == Masu::Putted(BorW::Black));
        assert!(field[4][4] == Masu::Putted(BorW::Black));
        assert!(field[3][4] == Masu::Putted(BorW::White));
        assert!(field[4][3] == Masu::Putted(BorW::White));
    }
    #[test]
    fn check_putable_test() {
        let mut field = [[Masu::Empty; 8]; 8];
        init_field(&mut field);
        let turn = BorW::Black;
        assert!(!check_putable(&field, (0, 0), turn));
        assert!(check_putable(&field, (4, 2), turn));
    }
    #[test]
    fn auto_reverse_test() {
        let mut field = [[Masu::Empty; 8]; 8];
        field[3][3] = Masu::Putted(BorW::Black);
        field[3][4] = Masu::Putted(BorW::White);
        field[3][5] = Masu::Putted(BorW::Black);
        auto_reverse(&mut field, (3, 5), BorW::Black);
        assert!(field[3][4] == Masu::Putted(BorW::Black));
    }
}
