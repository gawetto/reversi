#[derive(Debug, Clone)]
pub struct FieldOutError;
impl std::fmt::Display for FieldOutError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid range position")
    }
}

pub struct Field {
    field: [[Masu; 8]; 8],
}

pub enum GameResult {
    Win(BorW),
    Draw,
    Playing,
}

impl Field {
    pub fn new() -> Self {
        let mut f = Self {
            field: [[Masu::Empty; 8]; 8],
        };
        f.field[3][3] = Masu::Putted(BorW::Black);
        f.field[4][4] = Masu::Putted(BorW::Black);
        f.field[3][4] = Masu::Putted(BorW::White);
        f.field[4][3] = Masu::Putted(BorW::White);
        f
    }
    pub fn get(&self, p: Position) -> Masu {
        self.field[p.x][p.y]
    }
    pub fn set(&mut self, p: Position, masu: Masu) {
        self.field[p.x][p.y] = masu
    }
    pub fn count(&self, color: BorW) -> usize {
        self.field
            .iter()
            .flatten()
            .filter(|x| match x {
                Masu::Putted(c) => c.eq(&color),
                _ => false,
            })
            .count()
    }
    pub fn puttable(&self, color: BorW) -> bool {
        for i in 0..8 {
            for j in 0..8 {
                let p = Position::new(i, j).unwrap();
                if check_putable(self, p, color) {
                    return true;
                }
            }
        }
        return false;
    }
    pub fn get_gameresult(&self) -> GameResult {
        if self.puttable(BorW::White) || self.puttable(BorW::Black) {
            return GameResult::Playing;
        }
        let bcount = self.count(BorW::Black);
        let wcount = self.count(BorW::White);
        if bcount == wcount {
            GameResult::Draw
        } else if bcount > wcount {
            GameResult::Win(BorW::Black)
        } else {
            GameResult::Win(BorW::White)
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Position {
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
    pub fn x(self) -> usize {
        self.x
    }
    pub fn y(self) -> usize {
        self.y
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Masu {
    Empty,
    Putted(BorW),
}

#[derive(Copy, Clone, PartialEq)]
pub enum BorW {
    Black,
    White,
}

pub fn get_another_color(color: BorW) -> BorW {
    match color {
        BorW::Black => BorW::White,
        BorW::White => BorW::Black,
    }
}

fn get_reversable(field: &Field, point: Position, color: BorW) -> Vec<Position> {
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
                    if field.get(p) == Masu::Empty {
                        break false;
                    }
                    if field.get(p) == Masu::Putted(color) {
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

pub fn check_putable(field: &Field, point: Position, turn: BorW) -> bool {
    if field.get(point) != Masu::Empty {
        return false;
    }
    if get_reversable(field, point, turn).len() == 0 {
        return false;
    }
    return true;
}

pub fn auto_reverse(field: &mut Field, point: Position, turn: BorW) {
    get_reversable(field, point, turn)
        .into_iter()
        .for_each(|p| field.set(p, field.get(point)));
}

pub fn create_initial_data() -> (Field, Position, BorW) {
    let field = Field::new();
    let cursor = Position::new(0, 0).unwrap();
    let turn = BorW::Black;
    return (field, cursor, turn);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
    #[test]
    fn init_field_test() {
        let field = Field::new();
        assert!(field.get(Position::new(3, 3).unwrap()) == Masu::Putted(BorW::Black));
        assert!(field.get(Position::new(4, 4).unwrap()) == Masu::Putted(BorW::Black));
        assert!(field.get(Position::new(3, 4).unwrap()) == Masu::Putted(BorW::White));
        assert!(field.get(Position::new(4, 3).unwrap()) == Masu::Putted(BorW::White));
    }
    #[test]
    fn check_putable_test() {
        let field = Field::new();
        let turn = BorW::Black;
        assert!(!check_putable(&field, Position::new(0, 0).unwrap(), turn));
        assert!(check_putable(&field, Position::new(4, 2).unwrap(), turn));
    }
    #[test]
    fn auto_reverse_test() {
        let mut field = Field::new();
        field.set(Position::new(3, 3).unwrap(), Masu::Putted(BorW::Black));
        field.set(Position::new(3, 4).unwrap(), Masu::Putted(BorW::White));
        field.set(Position::new(3, 5).unwrap(), Masu::Putted(BorW::Black));
        auto_reverse(&mut field, Position::new(3, 5).unwrap(), BorW::Black);
        assert!(field.get(Position::new(3, 4).unwrap()) == Masu::Putted(BorW::Black));
    }
    #[test]
    fn count_test() {
        let mut field = Field::new();
        field.set(Position::new(0, 0).unwrap(), Masu::Putted(BorW::Black));
        assert!(field.count(BorW::Black) == 3)
    }
}
