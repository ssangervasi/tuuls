use std::fs;
use std::io;
use std::io::BufRead;
use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

pub struct CliOptions {
    pub path: String,
}

pub fn eval(opts: &CliOptions) {
    let fpath = Path::new(&opts.path);
    let f = fs::File::open(fpath).unwrap();
    let reader = io::BufReader::new(f);
    let mut parser = Parser::new();
    for (i, line_r) in reader.lines().enumerate() {
        if let Ok(line) = line_r {
            println!("{}:\t{}", i, line);
            parser.take_line(&line);
        }
    }
    println!("{:?}", parser)
}

lazy_static! {
    static ref BRACKET: Regex = Regex::new(r"[\[\]]").unwrap();
    static ref SPACE: Regex = Regex::new(r"\s+").unwrap();
}

#[derive(Debug, Copy, Clone)]
enum TokenType {
    Bracket,
    Space,
    Any,
}

#[derive(Debug, Clone)]
enum Context {
    Single(TokenType, String),
    Multi(TokenType, String),
    Bare(String),
    Empty,
}

impl Context {
    fn combine(&self, other: &Self) -> (Self, Self) {
        use Context::*;
        match (self, other) {
            (Multi(s_t, s_s), Multi(o_t, o_s)) => match s_t {
                o_t => {
                    let mut s = String::new();
                    s.push_str(s_s);
                    s.push_str(o_s);
                    (Empty, Multi(*s_t, s))
                }
                _ => (self, other),
            },
            _ => (self, other),
        }
    }
}

#[derive(Debug)]
struct Parser {
    tokens: Vec<String>,
}

impl Parser {
    fn new() -> Self {
        Self {
            tokens: Vec::with_capacity(1000),
        }
    }

    fn push(&mut self, context: &Context) {
        use Context::*;
        match context {
            Single(_, s) => {
                self.tokens.push(s.to_string());
            }
            Multi(_, s) => {
                self.tokens.push(s.to_string());
            }
            Bare(s) => {
                self.tokens.push(s.to_string());
            }
            Empty => {}
        }
    }

    fn take_line(&mut self, line: &str) {
        let gs = UnicodeSegmentation::graphemes(line, true).collect::<Vec<&str>>();
        let mut context = Context::Empty;
        for g in gs {
            let current = if BRACKET.is_match(g) {
                Context::Single(TokenType::Bracket, g.to_string())
            } else if SPACE.is_match(g) {
                Context::Multi(TokenType::Bracket, g.to_string())
            } else {
                Context::Bare(g.to_string())
            };

            let (complete, incomplete) = context.combine(&current);
            self.push(&complete);
            context = incomplete;

            // if BRACKET.is_match(g) {
            //     if !cur.is_empty {
            //         self.tokens.push(cur);
            //         cur = String::new();
            //     }
            //     self.tokens.push(g.to_string());
            // } else {
            //     cur.push_str(g)
            // }
        }
    }
}
