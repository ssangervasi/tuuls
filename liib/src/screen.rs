use std::collections::HashMap;

use crate::position::Position;

pub const BLANK: char = ' ';

#[derive(Debug)]
pub struct Screen {
    pub cols: i32,
    pub rows: i32,
    written: HashMap<Position, char>,
    buffer: HashMap<Position, char>,
}

impl Default for Screen {
    fn default() -> Self {
        Self::with_size((100, 100).into())
    }
}

impl Screen {
    pub fn with_size(col_row: Position) -> Self {
        Self {
            cols: col_row.col,
            rows: col_row.row,
            written: HashMap::with_capacity(10 * 10),
            buffer: HashMap::with_capacity(10 * 10),
        }
    }

    /**
     * Inserts the character into the buffer. Until the screen is [#flush]-ed the written
     * value will not be returned by [#read]. Returns the character that was previously
     * buffered, or [BLANK].
     */
    pub fn write(&mut self, position: &Position, ch: char) -> char {
        self.buffer.insert(*position, ch).unwrap_or(BLANK)
    }

    /**
     * Returns the character that has been written and flushed at the designated position.
     * If the position has not been touched, [BLANK] is returned.
     */
    pub fn read(&self, position: &Position) -> char {
        match self.written.get(position) {
            Some(&ch) => ch,
            None => BLANK,
        }
    }

    /**
     * Flushes the buffered writes into the written state and returns copies of the elements
     * that were written.
     */
    pub fn flush(&mut self) -> Vec<(Position, char)> {
        let mut updates: Vec<(Position, char)> = Vec::with_capacity(self.buffer.capacity());
        for (&position, &ch) in self.buffer.iter() {
            if self.clamp(&position) != position {
                // Out-of-bounds positions can be buffered, but the are ignored at flush.
                continue;
            }

            let original = self.written.insert(position, ch);
            if original == None || original != Some(ch) {
                updates.push((position, ch));
            }
        }
        updates
    }

    // pub(crate) fn mem(&self) -> usize {
    //     (std::mem::size_of::<Position>() + std::mem::size_of::<char>()) * self.written.len()
    // }

    pub fn clamp(&self, position: &Position) -> Position {
        position.clamp(Position::new(0, 0), Position::new(self.cols, self.rows))
    }
}

#[macro_export]
macro_rules! scrite {
    ($s:expr, $( $x:expr ),+ ) => {
        let screen: &mut Screen = $s;
        $(
            {
                let (c, r, ch): (i32, i32, char) = $x;
                screen.write(&(c, r).into(), ch);
            }
        )+
    };
}

#[test]
fn test_buffer() {
    let mut screen = Screen::default();
    scrite!(&mut screen, (0, 1, 'h'), (1, 2, 'i'));

    assert_eq!(screen.read(&(0, 1).into()), ' ');
    assert_eq!(screen.read(&(1, 2).into()), ' ');

    let out = screen.flush();
    assert_eq!(out.len(), 2);

    assert_eq!(screen.read(&(0, 1).into()), 'h');
    assert_eq!(screen.read(&(1, 2).into()), 'i');
}
