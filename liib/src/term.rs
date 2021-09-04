#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_macros)]

use core::time::Duration;
use crossterm::{
    cursor::{
        position as crossterm_position, MoveTo, MoveToNextLine, RestorePosition, SavePosition,
    },
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, SetForegroundColor},
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        size as crossterm_size,
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

use crate::position::{Position, Visible};
use crate::screen::Screen;

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

const BLANK: char = ' ';

pub fn free_draw() -> crossterm::Result<()> {
    make_room();
    edit_loop()?;

    Ok(())
}

pub fn make_room() {
    ex!(Clear(ClearType::All), MoveTo(0, 0));
}

pub fn get_size() -> Position {
    crossterm_size().unwrap_or((0, 0)).into()
}

pub fn get_position() -> Position {
    crossterm_position().unwrap_or((0, 0)).into()
}

fn edit_loop() -> crossterm::Result<()> {
    let mut screen = Screen::with_size(get_size());
    let mut res: Res = Res::None;

    enable_raw_mode()?;
    loop {
        let size = get_size();
        let cursor = get_position();
        just_dump_screen(&mut screen)?;
        rex!(
            MoveTo(0, 0),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(Color::Red),
            Print(format!(
                "Size: {} | Pos: {} | Cur: '{}' | Res: {:?}",
                size,
                cursor,
                screen.read(&cursor),
                res
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
            res = result;

            match result {
                Res::Move(dp) => {
                    let np = screen.clip(&(cursor + dp));
                    ex!(MoveTo(np.0, np.1))
                }
                Res::Write(ch) => {
                    screen.write(&cursor, ch);
                }
                Res::Quit => break,
                Res::None => {}
            }
        }
    }
    disable_raw_mode()?;

    Ok(())
}

#[derive(Debug, Copy, Clone)]
pub enum Res {
    Move(Position),
    Write(char),
    Quit,
    None,
}

pub fn process_event(
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
            let start_is_blank = screen.read(&(c, r).into()) == BLANK;

            match event.code {
                KeyCode::Left => {
                    for i in 0..=c {
                        if (screen.read(&(c - i, r).into()) == BLANK) != start_is_blank {
                            res = Res::Move((-i, 0).into());
                            break;
                        }
                    }
                }
                KeyCode::Right => {
                    for i in 0..=(w - c) {
                        if (screen.read(&(c + i, r).into()) == BLANK) != start_is_blank {
                            res = Res::Move((i, 0).into());
                            break;
                        }
                    }
                }
                KeyCode::Up => {
                    for i in 0..=r {
                        if (screen.read(&(c, r - i).into()) == BLANK) != start_is_blank {
                            res = Res::Move((0, -i).into());
                            break;
                        }
                    }
                }
                KeyCode::Down => {
                    for i in 0..=(h - r) {
                        if (screen.read(&(c, r + i).into()) == BLANK) != start_is_blank {
                            res = Res::Move((0, i).into());
                            break;
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

pub fn dump_screen(screen: &mut Screen) -> crossterm::Result<()> {
    enable_raw_mode()?;
    just_dump_screen(screen)?;
    disable_raw_mode()?;

    Ok(())
}

pub fn just_dump_screen(screen: &mut Screen) -> crossterm::Result<()> {
    let mut stdout = stdout();
    queue!(stdout, SavePosition)?;
    for (postion, ch) in screen.flush() {
        let clipped = screen.clip(&postion);
        queue!(stdout, MoveTo(clipped.0, clipped.1), Print(ch))?;
    }
    queue!(stdout, RestorePosition)?;
    stdout.flush()?;

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
    dump_screen(&mut screen).unwrap();
    println!();
}

pub fn e() {
    enable_raw_mode().unwrap();
}
pub fn d() {
    disable_raw_mode().unwrap();
}

#[macro_export]
macro_rules! rawful {
    ($t:expr) => {
        liib::term::e();
        let v = $t;
        liib::term::d();
        v
    };
}

/**
 * What a fun lesson in how up/down just shift the buffer contents and
 * dlete lines from the ends. Not nearly as useful.
 */
pub fn scroll_test() -> crossterm::Result<()> {
    let (_w, h): Visible = get_size().into();

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
