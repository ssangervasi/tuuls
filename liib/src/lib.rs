#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_macros)]

use core::time::Duration;
use crossterm::{
    cursor::{position as get_position, MoveTo, MoveToNextLine, RestorePosition, SavePosition},
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, SetForegroundColor},
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        size as get_size,
        //
        Clear,
        ClearType,
        //
        ScrollDown,
        ScrollUp,
    },
};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::*;
use std::thread::sleep;

pub mod cro;
pub mod position;

use position::{Position, Visible};

macro_rules! ex {
    ( $( $x:expr ),* ) => {
        execute!(
            stdout(),
            $(
                $x,
            )*
        )
        .unwrap();
    }
}

macro_rules! rex {
    ( $( $x:expr ),* ) => {
        execute!(
            stdout(),
            SavePosition,
            $(
                $x,
            )*
            RestorePosition
        )
        .unwrap();
    };
}

pub const BLANK: char = ' ';

#[derive(Clone, Debug)]
pub struct Screen {
    cols: i32,
    rows: i32,
    hash_map: HashMap<Position, char>,
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            cols: 100,
            rows: 100,
            hash_map: HashMap::with_capacity(10 * 10),
        }
    }
}

impl Screen {
    pub fn with_size(col_row: Position) -> Self {
        Self {
            cols: col_row.col,
            rows: col_row.row,
            hash_map: HashMap::with_capacity(10 * 10),
        }
    }

    pub fn write(&mut self, position: &Position, ch: char) -> char {
        match ch {
            BLANK => self.hash_map.remove(position),
            _ => self.hash_map.insert(*position, ch),
        }
        .unwrap_or(BLANK)
    }

    pub fn read(&self, position: &Position) -> char {
        match self.hash_map.get(position) {
            Some(&ch) => ch,
            None => BLANK,
        }
    }

    fn mem(&self) -> usize {
        (std::mem::size_of::<Position>() + std::mem::size_of::<char>()) * self.hash_map.len()
    }

    fn height(&self) -> i32 {
        self.rows
    }

    fn width(&self) -> i32 {
        self.cols
    }

    fn clip(&self, position: &Position) -> Visible {
        (*position).into()
    }
}

pub fn dump_screen(screen: Screen) -> crossterm::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    for r in 0..=(screen.height()) {
        for c in 0..=(screen.width()) {
            let p = Position::new(c, r);
            let clipped = screen.clip(&p);
            queue!(stdout, MoveTo(clipped.0, clipped.1), Print(screen.read(&p)))?;
        }
    }
    stdout.flush()?;
    disable_raw_mode()?;

    Ok(())
}

#[test]
fn test_dump() {
    let mut screen = Screen::default();
    screen.write(&(0, 0).into(), 'a');
    screen.write(&(0, 1).into(), 'b');
    screen.write(&(0, 2).into(), 'd');
    screen.write(&(0, 3).into(), 'e');
    screen.write(&(0, 4).into(), 'f');

    screen.write(&(5, 0).into(), 'a');
    screen.write(&(5, 1).into(), 'b');
    screen.write(&(5, 2).into(), 'd');
    screen.write(&(5, 3).into(), 'e');
    screen.write(&(5, 4).into(), 'f');

    screen.write(&(1, 2).into(), 'h');
    screen.write(&(2, 2).into(), 'h');
    screen.write(&(3, 2).into(), 'h');
    screen.write(&(4, 2).into(), 'h');

    make_room();
    dump_screen(screen).unwrap();
    println!();
}

pub fn term_crossterm() -> crossterm::Result<()> {
    make_room();
    edit_loop()?;

    Ok(())
}

fn make_room() {
    ex!(Clear(ClearType::All), MoveTo(0, 0));
}

/**
 * What a fun lesson in how up/down just shift the buffer contents and
 * dlete lines from the ends. Not nearly as useful.
 */
pub fn scroll_test() -> crossterm::Result<()> {
    let (_w, h) = get_size().unwrap_or((0, 0));

    ex!(Print("\n--A--\n"));

    ex!(MoveTo(0, 0));
    for i in 0..(2 * h) {
        ex!(Print(format!("{}\n", i)));
        sleep(Duration::from_millis(50));
    }

    ex!(MoveTo(0, 1), Print("Scroll up"));
    sleep(Duration::from_millis(2000));
    ex!(ScrollUp(h / 2));
    ex!(MoveTo(0, 2), Print("^^^^^"));
    sleep(Duration::from_millis(2000));

    ex!(MoveTo(0, 3), Print("Scroll down"));
    sleep(Duration::from_millis(2000));
    ex!(ScrollDown(h / 2));
    ex!(MoveTo(0, 4), Print("VVVV"));
    sleep(Duration::from_millis(2000));

    ex!(Print("Done"));

    Ok(())
}

fn edit_loop() -> crossterm::Result<()> {
    let mut screen = Screen::default();

    enable_raw_mode()?;
    loop {
        let size: Position = get_size().unwrap_or((0, 0)).into();
        let cursor: Position = get_position().unwrap_or((0, 0)).into();
        rex!(
            MoveTo(0, 0),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(Color::Red),
            Print(format!(
                "Size: {} | Pos: {} | Cur: {} | Screen Mem: {}",
                size,
                cursor,
                screen.read(&cursor),
                screen.mem()
            ))
        );

        while !poll(Duration::from_millis(500))? {}

        if let Event::Key(event) = read()? {
            rex!(
                MoveTo(0, 1),
                Clear(ClearType::CurrentLine),
                SetForegroundColor(Color::Blue),
                Print(format!("{:?}", event))
            );

            let result: Res = process_event(&mut screen, event, size, cursor);
            match result {
                Res::Move(dp) => {
                    let np = screen.clip(&(cursor + dp));
                    ex!(MoveTo(np.0, np.1))
                }

                Res::Write(ch) => {
                    screen.write(&cursor, ch);
                    let np = screen.clip(&cursor);
                    ex!(Print(ch), MoveTo(np.0, np.1));
                }
                Res::Quit => break,
                Res::None => {}
            }
        }
    }
    disable_raw_mode()?;

    Ok(())
}

enum Res {
    Move(Position),
    Write(char),
    Quit,
    None,
}

fn process_event(
    screen: &mut Screen,
    event: KeyEvent,
    Position { col: w, row: h }: Position,
    Position { col: c, row: r }: Position,
) -> Res {
    match event {
        // Jump ends
        KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::CONTROL,
        } => Res::Move((-w, 0).into()),
        KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::CONTROL,
        } => Res::Move((w, 0).into()),

        KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::CONTROL,
        } => Res::Move((0, -h).into()),
        KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::CONTROL,
        } => Res::Move((0, h).into()),

        // Jump boundaries
        KeyEvent {
            code: KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down,
            modifiers: KeyModifiers::ALT,
        } => {
            let mut res = Res::Move((0, 0).into());
            let match_blank = screen.read(&(c, r).into()) == BLANK;

            match event.code {
                KeyCode::Left => {
                    for i in (0..=c).rev() {
                        if (screen.read(&(i, r).into()) == BLANK) != match_blank {
                            res = Res::Move((i - c, 0).into());
                        }
                    }
                }
                KeyCode::Right => {
                    for i in c..=w {
                        if (screen.read(&(i, r).into()) == BLANK) != match_blank {
                            res = Res::Move((i - c, 0).into());
                        }
                    }
                }
                KeyCode::Up => {
                    for i in (0..=r).rev() {
                        if (screen.read(&(c, i).into()) == BLANK) != match_blank {
                            res = Res::Move((0, i - r).into());
                        }
                    }
                }
                KeyCode::Down => {
                    for i in r..=h {
                        if (screen.read(&(c, i).into()) == BLANK) != match_blank {
                            res = Res::Move((0, i - r).into());
                        }
                    }
                }
                _ => {}
            }
            res
        }

        // Jump
        KeyEvent {
            code: KeyCode::Left,
            modifiers: _,
        } => Res::Move((-1, 0).into()),
        KeyEvent {
            code: KeyCode::Right,
            modifiers: _,
        } => Res::Move((1, 0).into()),

        KeyEvent {
            code: KeyCode::Up,
            modifiers: _,
        } => Res::Move((0, -1).into()),
        KeyEvent {
            code: KeyCode::Down,
            modifiers: _,
        } => Res::Move((0, 1).into()),

        // Quit
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
        } => Res::Quit,

        // Write
        KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: _,
        } => Res::Write(ch),

        // Unhandled
        _ => Res::None,
    }
}
