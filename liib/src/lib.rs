#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_macros)]

use core::time::Duration;
use crossterm::{
    cursor::{position, MoveTo, MoveToNextLine, RestorePosition, SavePosition},
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        size,
        Clear,
        ClearType,
        //
        ScrollDown as ScrollBack,
        //
        ScrollDown,
        //
        ScrollUp as ScrollForward,
        //
        ScrollUp,
    },
};
use std::collections::HashMap;
use std::io::*;
use std::thread::sleep;

pub mod cro;

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
    // ex!(ScrollDown(size().unwrap_or((0, 0)).1));
    let (_w, h) = size().unwrap_or((0, 0));

    ex!(Print("\n--A--\n"));

    // enable_raw_mode()?;
    // {
    //     // ex!(Clear(ClearType::All), MoveTo(0, 0));
    //     // edit_loop()?;
    //     // ex!(MoveToNextLine(2));
    //     ex!(MoveTo(0, 0), Print("|^"));
    //     ex!(MoveTo(w - 2, 0), Print("^|"));
    //     ex!(MoveTo(0, h - 1), Print("|_"));
    //     ex!(MoveTo(w - 2, h - 1), Print("_|"));
    //     ex!(MoveTo(w / 2 - 5, h / 2), Print("-Middle-"));

    //     while !poll(Duration::from_millis(500))? {}
    // }
    // disable_raw_mode()?;

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

    // ex!(MoveTo(0, 0));
    // ex!(Print("--B--"));
    // ex!(ScrollForward(5));
    // ex!(Print("--CC--"));
    // ex!(ScrollForward(10));
    // ex!(Print("--DDD--"));
    // ex!(ScrollForward(20));
    // ex!(Print("--EEEE--"));

    Ok(())
}

fn edit_loop() -> crossterm::Result<()> {
    let mut screen = Screen::default();

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

                let result: Res = process_event(&mut screen, event, (w, h), (c, r));
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
                    Res::Quit => break,
                    Res::None => {}
                }
            }
            _ => {}
        }
    }

    Ok(())
}

enum Res {
    Move((i32, i32)),
    Write(char),
    Quit,
    None,
}

fn process_event(screen: &mut Screen, event: KeyEvent, (w, h): Coord, (c, r): Coord) -> Res {
    match event {
        // Jump ends
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

        // Jump boundaries
        KeyEvent {
            code: KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down,
            modifiers: KeyModifiers::ALT,
        } => {
            let mut res = Res::Move((0, 0));
            let match_blank = screen.read((c, r)) == BLANK;

            match event.code {
                KeyCode::Left => {
                    for i in (0..=c).rev() {
                        if (screen.read((i, r)) == BLANK) != match_blank {
                            res = Res::Move((i - c, 0));
                        }
                    }
                }
                KeyCode::Right => {
                    for i in c..=w {
                        if (screen.read((i, r)) == BLANK) != match_blank {
                            res = Res::Move((i - c, 0));
                        }
                    }
                }
                KeyCode::Up => {
                    for i in (0..=r).rev() {
                        if (screen.read((c, i)) == BLANK) != match_blank {
                            res = Res::Move((0, i - r));
                        }
                    }
                }
                KeyCode::Down => {
                    for i in r..=h {
                        if (screen.read((c, i)) == BLANK) != match_blank {
                            res = Res::Move((0, i - r));
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

fn smerf() {
    let mut d = derf();
    d[0] = 40;
    berf(&mut d);
    d[0] = 40;
}

fn derf() -> Vec<i32> {
    let mut t = vec![1, 2];
    berf(&mut t);
    t[1] = t[0];
    t
}

fn berf(b: &mut Vec<i32>) {
    let c = b;
    // b[0] = 10;
    c[0] = 12;
}

// fn smerf() {
//     let mut d = derf();
//     d[0] = 40;
// }

// fn derf() -> [i32; 2] {
//     // let t = vec![1, 2];
//     let mut t = [1, 2];
//     berf(&mut t);
//     t[1] = t[0];
//     t
// }

// fn berf(b: &mut [i32; 2]) {
//     b[0] = 10
// }
