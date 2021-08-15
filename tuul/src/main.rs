#![allow(dead_code)]

use core::time::Duration;
use std::io::*;

use liib;

fn main() {
    term_crossterm()
}

fn term_crossterm() {
    use crossterm::{
        cursor::{MoveDown, MoveTo, MoveToNextLine},
        event::{poll, read, Event},
        execute,
        style::{Color, Print, ResetColor, SetForegroundColor},
        terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    };

    enable_raw_mode().unwrap();
    {
        let mut s = stdout();

        execute!(s, Clear(ClearType::All), MoveTo(0, 0)).unwrap();

        let (w, h) = size().unwrap();
        execute!(
            s,
            SetForegroundColor(Color::Red),
            Print(format!("Terminal Size: {}x{}\n", w, h)),
            ResetColor
        )
        .unwrap();

        execute!(s, MoveToNextLine(1)).unwrap();

        while !poll(Duration::from_millis(500)).unwrap() {}
        match read().unwrap() {
            Event::Key(event) => {
                execute!(s, MoveToNextLine(1)).unwrap();
                execute!(s, Print(format!("{:?}", event))).unwrap();
            }
            _ => {}
        }

        execute!(s, MoveToNextLine(1)).unwrap();
        execute!(
            s,
            SetForegroundColor(Color::Red),
            Print("Yahoo"),
            ResetColor
        )
        .unwrap();
        execute!(s, MoveToNextLine(1)).unwrap();
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
