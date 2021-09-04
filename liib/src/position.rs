use core::cmp::Ordering;
use core::fmt;
use core::hash::Hash;
use std::convert;
use std::ops;

pub type Visible = (u16, u16);

#[derive(Default, Copy, Clone, Debug, Eq)]
pub struct Position {
    pub col: i32,
    pub row: i32,
}

impl Position {
    pub fn new(col: i32, row: i32) -> Self {
        Self { col, row }
    }

    pub fn clamp(&self, min: Position, max: Position) -> Self {
        Self {
            col: self.col.clamp(min.col, max.col),
            row: self.row.clamp(min.row, max.row),
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.col, self.row)
    }
}

impl Hash for Position {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        let t: (i32, i32) = (*self).into();
        t.hash(state);
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.row == other.row && self.col == other.col
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.row.cmp(&other.row) {
            Ordering::Equal => self.col.cmp(&other.col),
            ne_result => ne_result,
        }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl ops::Sub for Position {
    type Output = Position;

    fn sub(self, other: Position) -> Position {
        Position {
            col: self.col - other.col,
            row: self.row - other.row,
        }
    }
}

impl ops::Add for Position {
    type Output = Position;

    fn add(self, other: Position) -> Position {
        Position {
            col: self.col + other.col,
            row: self.row + other.row,
        }
    }
}

impl convert::From<Position> for Visible {
    fn from(p: Position) -> Self {
        let Position { col, row } = p;
        (floorcast(col), floorcast(row))
    }
}

fn floorcast(signed: i32) -> u16 {
    if signed < 0 {
        0
    } else {
        signed as u16
    }
}

impl convert::From<Visible> for Position {
    fn from(t: Visible) -> Self {
        let (col, row) = t;
        Position {
            // Unsigned bits will always fit, so as casting is fine.
            col: col as i32,
            row: row as i32,
        }
    }
}

impl convert::From<Position> for (i32, i32) {
    fn from(p: Position) -> Self {
        let Position { col, row } = p;
        (col, row)
    }
}

impl convert::From<(i32, i32)> for Position {
    fn from(t: (i32, i32)) -> Self {
        let (col, row) = t;
        Position { col, row }
    }
}

#[test]
fn test_position() {
    let small = Position::new(1, 2);
    let big = Position::new(3, 6);
    assert!((small + big - small + small) < (big + big));

    let s: u16 = 2 << 14;
    let t: i32 = s as i32;
    let u: u16 = t as u16;
    assert_eq!(s, u);

    let neg = Position::new(-2 << 12, -2 << 6);

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    fn hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    assert!(hash(&neg) == hash(&neg));
    assert!(hash(&neg) != hash(&big));
    assert!(hash(&small) != hash(&big));
    assert!(hash(&small) == hash(&small));
}
