#![allow(dead_code)]
#![allow(unused_macros)]

use core::time::Duration;
use std::io::*;

pub mod cro;

macro_rules! ex {
    ( $( $x:expr ),* ) => {
        {
            use crossterm::{
                execute,
            };
            execute!(
                stdout(),
                $(
                    $x,
                )*
            )
            .unwrap();
        }
    };
}

macro_rules! rex {
    ( $( $x:expr ),* ) => {
        {
            use crossterm::{
                execute,
                cursor::{SavePosition, RestorePosition},
            };
            execute!(
                stdout(),
                SavePosition,
                $(
                    $x,
                )*
                RestorePosition
            )
            .unwrap();
        }
    };
}

use std::collections::HashMap;

type Coord = (i32, i32);
const BLANK: char = ' ';

#[derive(Clone, Debug)]
struct Screen {
    hash_map: HashMap<Coord, char>,
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            hash_map: HashMap::with_capacity(10 * 10),
        }
    }
}

impl Screen {
    fn write(&mut self, coord: Coord, ch: char) -> char {
        match ch {
            BLANK => self.hash_map.remove(&coord),
            _ => self.hash_map.insert(coord, ch),
        }
        .unwrap_or(BLANK)
    }

    fn read(&mut self, coord: Coord) -> char {
        match self.hash_map.get(&coord) {
            Some(&ch) => ch,
            None => BLANK,
        }
    }

    fn mem(&self) -> usize {
        (std::mem::size_of::<Coord>() + std::mem::size_of::<char>()) * self.hash_map.len()
    }
}

pub fn term_crossterm() -> crossterm::Result<()> {
    use crossterm::{
        cursor::{position, MoveTo, MoveToNextLine},
        event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
        execute,
        style::{Color, Print, SetForegroundColor},
        terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    };

    let mut screen = Screen::default();

    enable_raw_mode()?;
    {
        ex!(Clear(ClearType::All), MoveTo(0, 0));

        loop {
            let (w, h) = {
                let p = size().unwrap_or((0, 0));
                (p.0 as i32, p.1 as i32)
            };
            let (c, r): (i32, i32) = {
                let p = position().unwrap_or((0, 0));
                (p.0 as i32, p.1 as i32)
            };

            rex!(
                MoveTo(0, 0),
                Clear(ClearType::CurrentLine),
                SetForegroundColor(Color::Red),
                Print(format!(
                    "Size: {}x{} | Pos: {}x{} | Cur: {} | Screen Mem: {}",
                    w,
                    h,
                    c,
                    r,
                    screen.read((c, r)),
                    screen.mem()
                ))
            );

            while !poll(Duration::from_millis(500))? {}

            match read()? {
                Event::Key(event) => {
                    rex!(
                        MoveTo(0, 1),
                        Clear(ClearType::CurrentLine),
                        SetForegroundColor(Color::Blue),
                        Print(format!("{:?}", event))
                    );

                    enum Res {
                        Move((i32, i32)),
                        Write(char),
                        None,
                    }

                    let result: Res = match event {
                        // Jump move
                        KeyEvent {
                            code: KeyCode::Left,
                            modifiers: KeyModifiers::CONTROL,
                        } => Res::Move((-w, 0)),
                        KeyEvent {
                            code: KeyCode::Right,
                            modifiers: KeyModifiers::CONTROL,
                        } => Res::Move((w, 0)),

                        KeyEvent {
                            code: KeyCode::Up,
                            modifiers: KeyModifiers::CONTROL,
                        } => Res::Move((0, -h)),
                        KeyEvent {
                            code: KeyCode::Down,
                            modifiers: KeyModifiers::CONTROL,
                        } => Res::Move((0, h)),

                        // Jump
                        KeyEvent {
                            code: KeyCode::Left,
                            modifiers: _,
                        } => Res::Move((-1, 0)),
                        KeyEvent {
                            code: KeyCode::Right,
                            modifiers: _,
                        } => Res::Move((1, 0)),

                        KeyEvent {
                            code: KeyCode::Up,
                            modifiers: _,
                        } => Res::Move((0, -1)),
                        KeyEvent {
                            code: KeyCode::Down,
                            modifiers: _,
                        } => Res::Move((0, 1)),

                        // Quit
                        KeyEvent {
                            code: KeyCode::Char('c'),
                            modifiers: KeyModifiers::CONTROL,
                        } => break,

                        // Write
                        KeyEvent {
                            code: KeyCode::Char(ch),
                            modifiers: _,
                        } => Res::Write(ch),

                        // Unhandled
                        _ => Res::None,
                    };
                    match result {
                        Res::Move((dc, dr)) => {
                            let nc: u16 = (c + dc).max(0).min(w) as u16;
                            let nr: u16 = (r + dr).max(0).min(h) as u16;
                            ex!(MoveTo(nc, nr))
                        }

                        Res::Write(ch) => {
                            screen.write((c, r), ch);
                            ex!(Print(ch), MoveTo(c as u16, r as u16));
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        ex!(MoveToNextLine(2));
    }

    disable_raw_mode()?;

    Ok(())
}
