#![allow(dead_code)]
use liib;

fn main() {
    term()
}

fn term() {
    use std::io::*;
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
    use std::time::Duration;

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

// macro_rules! w {
//     () => {
//         write()
//     };
// }

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
