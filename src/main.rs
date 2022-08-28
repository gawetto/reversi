use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};

#[derive(Debug, Clone)]
struct FieldOutError;
impl std::fmt::Display for FieldOutError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid range position")
    }
}

#[derive(Copy, Clone, PartialEq)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    pub fn new(x: i32, y: i32) -> std::result::Result<Self, FieldOutError> {
        if x < 0 || y < 0 || 7 < x || 7 < y {
            return Err(FieldOutError);
        }
        Ok(Self {
            x: x as usize,
            y: y as usize,
        })
    }
    pub fn up(self) -> std::result::Result<Self, FieldOutError> {
        Self::new(self.x as i32 - 1, self.y as i32)
    }
    pub fn down(self) -> std::result::Result<Self, FieldOutError> {
        Self::new(self.x as i32 + 1, self.y as i32)
    }
    pub fn left(self) -> std::result::Result<Self, FieldOutError> {
        Self::new(self.x as i32, self.y as i32 - 1)
    }
    pub fn right(self) -> std::result::Result<Self, FieldOutError> {
        Self::new(self.x as i32, self.y as i32 + 1)
    }
}

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

fn get_reversable(field: &[[Masu; 8]; 8], point: Position, color: BorW) -> Vec<Position> {
    let mut result = Vec::new();
    let direction = [
        |x: Position| x.up()?.left(),
        |x: Position| x.up(),
        |x: Position| x.up()?.right(),
        |x: Position| x.down()?.left(),
        |x: Position| x.down(),
        |x: Position| x.down()?.right(),
        |x: Position| x.right(),
        |x: Position| x.left(),
    ];
    for d in direction {
        let mut kouho = Vec::new();
        let mut position = point;
        let add = loop {
            match d(position) {
                Err(_) => break false,
                Ok(p) => {
                    if field[p.x][p.y] == Masu::Empty {
                        break false;
                    }
                    if field[p.x][p.y] == Masu::Putted(color) {
                        break true;
                    }
                    position = p;
                    kouho.push(p);
                }
            }
        };
        if add {
            result.append(&mut kouho);
        };
    }
    return result;
}

fn check_putable(field: &[[Masu; 8]; 8], point: Position, turn: BorW) -> bool {
    if field[point.x][point.y] != Masu::Empty {
        return false;
    }
    if get_reversable(field, point, turn).len() == 0 {
        return false;
    }
    return true;
}

fn auto_reverse(field: &mut [[Masu; 8]; 8], point: Position, turn: BorW) {
    get_reversable(field, point, turn)
        .into_iter()
        .for_each(|p| field[p.x][p.y] = field[point.x][point.y]);
}

fn input(
    event: Event,
    field: &mut [[Masu; 8]; 8],
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
                field[cursor.x][cursor.y] = Masu::Putted(*turn);
                auto_reverse(field, *cursor, *turn);
                *turn = get_another_color(*turn);
            }
        }
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            ..
        }) => {
            field[cursor.x][cursor.y] = Masu::Empty;
        }
        _ => {}
    }
    return Ok(());
}
fn view<T: std::io::Write>(
    output: &mut T,
    field: &[[Masu; 8]; 8],
    cursor: Position,
    turn: BorW,
) -> Result<()> {
    execute!(output, MoveTo(0, 0),)?;
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
            match field[p.x][p.y] {
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
    let mut cursor = Position::new(0, 0).unwrap();
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
        let mut cursor = Position::new(4, 2).unwrap();
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
        assert!(cursor.x == 4);
        assert!(cursor.y == 3);
        let downkey = Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        super::input(downkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(cursor.x == 5);
        assert!(cursor.y == 3);
        let leftkey = Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        super::input(leftkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(cursor.x == 5);
        assert!(cursor.y == 2);
        let upkey = Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        super::input(upkey, &mut field, &mut cursor, &mut end, &mut turn).unwrap();
        assert!(cursor.x == 4);
        assert!(cursor.y == 2);
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
        let cursor = Position::new(0, 0).unwrap();
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
        assert!(!check_putable(&field, Position::new(0, 0).unwrap(), turn));
        assert!(check_putable(&field, Position::new(4, 2).unwrap(), turn));
    }
    #[test]
    fn auto_reverse_test() {
        let mut field = [[Masu::Empty; 8]; 8];
        field[3][3] = Masu::Putted(BorW::Black);
        field[3][4] = Masu::Putted(BorW::White);
        field[3][5] = Masu::Putted(BorW::Black);
        auto_reverse(&mut field, Position::new(3, 5).unwrap(), BorW::Black);
        assert!(field[3][4] == Masu::Putted(BorW::Black));
    }
}
