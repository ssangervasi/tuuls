#![allow(dead_code)]
#![allow(unused_macros)]

use core::time::Duration;
use std::io::*;

use liib;

fn main() {
    term_crossterm()
}

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

fn term_crossterm() {
    use crossterm::{
        cursor::{position, MoveTo, MoveToNextLine},
        event::{poll, read, Event, KeyCode, KeyEvent},
        execute,
        style::{Color, Print, SetForegroundColor},
        terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    };

    enable_raw_mode().unwrap();
    {
        ex!(Clear(ClearType::All), MoveTo(0, 0));

        loop {
            let (w, h) = size().unwrap_or((0, 0));
            let (c, r): (i32, i32) = {
                let p = position().unwrap_or((0, 0));
                (p.0 as i32, p.1 as i32)
            };

            rex!(
                MoveTo(0, 0),
                SetForegroundColor(Color::Red),
                Print(format!("Size: {}x{} | Pos: {}x{}", w, h, c, r))
            );

            while !poll(Duration::from_millis(500)).unwrap() {}

            match read().unwrap() {
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

                        KeyEvent {
                            code: KeyCode::Esc,
                            modifiers: _,
                        } => break,
                        KeyEvent {
                            code: KeyCode::Char(ch),
                            modifiers: _,
                        } => Res::Write(ch),

                        _ => Res::None,
                    };
                    match result {
                        Res::Move((dc, dr)) => {
                            let nc: u16 = ((c + dc).max(0) as u16).min(w);
                            let nr: u16 = ((r + dr).max(0) as u16).min(h);
                            ex!(MoveTo(nc, nr))
                        }

                        Res::Write(ch) => {
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
    disable_raw_mode().unwrap();
}

fn term_termion() {
    use termion::input::*;
    use termion::*;

    let t = terminal_size();
    let (w, h) = t.unwrap();

    let mut so = stdout();
    write!(so, "{}", clear::All).unwrap();
    write!(so, "{}", cursor::Goto(1, 1)).unwrap();
    write!(so, "Terminal Size: {}x{}", w, h).unwrap();
    write!(so, "{}", cursor::Goto(1, 3)).unwrap();
    so.flush().unwrap();

    let si = stdin();

    use std::thread::sleep;

    loop {
        sleep(Duration::from_millis(100));

        write!(so, "{}{}", cursor::Goto(1, 3), clear::AfterCursor).unwrap();

        let mut buf = String::new();
        si.read_line(&mut buf).unwrap();
        write!(so, ">> {}", buf).unwrap();
        so.flush().unwrap();

        for e in buf.as_bytes().events() {
            write!(so, "E: {:?} \n", e).unwrap();
        }
    }
}

fn ansi() {
    use ansi_term::*;
    println!("{}", liib::MSG);
    println!("<{}>", Colour::Yellow.bold().paint("Bold yellow"))
}

fn macro_test() {
    // let v: Vec<u8> = liib::revec![1, 2, 3];
    let v: Vec<u8> = liib::revec![n(1), n(2), n(3)];
    print!("{:?}", v)
}

fn n(ni: u8) -> u8 {
    println!("n({})", ni);
    ni
}
