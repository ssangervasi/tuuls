use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::BufRead;
use std::mem::discriminant;
use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

pub struct CliOptions {
	pub path: String,
}

pub fn eval(opts: &CliOptions) {
	let fpath = Path::new(&opts.path);
	println!("Path: {:?}", fpath);

	let f = fs::File::open(fpath).unwrap();
	let reader = io::BufReader::new(f);
	let mut tokenizer = Tokenizer::new();

	println!("---------");
	println!("Contents:");

	for (i, line_r) in reader.lines().enumerate() {
		if let Ok(line) = line_r {
			println!("{}:\t{}", i, line);
			tokenizer.take_line(&line);
		}
	}
	println!("---------");
	println!("Tokens:");
	println!("{:?}", tokenizer);

	println!("---------");
	println!("Result:");

	Evaluator::new().eval(tokenizer.tokens);
}

lazy_static! {
	static ref BRACKET: Regex = Regex::new(r"[\[\]]").unwrap();
	static ref SPACE: Regex = Regex::new(r"[\t ]+").unwrap();
	static ref BREAK: Regex = Regex::new(r"[\n\r]+").unwrap();
}

#[derive(Debug, Copy, Clone)]
enum TokenType {
	Bracket,
	Space,
	Break,
	Text,
}

#[derive(Debug, Clone)]
enum Token {
	Single(TokenType, String),
	Multi(TokenType, String),
	Empty,
}

impl Token {
	fn combine(&self, other: &Self) -> (Self, Self) {
		use Token::*;
		match (self, other) {
			(Multi(s_t, s_s), Multi(o_t, o_s)) => {
				if discriminant(s_t) == discriminant(o_t) {
					let mut s = String::new();
					s.push_str(s_s);
					s.push_str(o_s);
					(Empty, Multi(*s_t, s))
				} else {
					(self.clone(), other.clone())
				}
			}
			_ => (self.clone(), other.clone()),
		}
	}

	fn t(&self) -> TokenType {
		use Token::*;
		match self {
			Multi(t, _) => *t,
			Single(t, _) => *t,
			Empty => TokenType::Space,
		}
	}

	fn content(&self) -> &str {
		use Token::*;
		match self {
			Multi(_, s) => s,
			Single(_, s) => s,
			Empty => "",
		}
	}
}

#[derive(Debug)]
struct Tokenizer {
	tokens: Vec<Token>,
}

impl Tokenizer {
	fn new() -> Self {
		Self {
			tokens: Vec::with_capacity(1000),
		}
	}

	fn take_line(&mut self, line: &str) {
		let gs = UnicodeSegmentation::graphemes(line, true).collect::<Vec<&str>>();
		let mut context = Token::Empty;
		for g in gs {
			let current = if BRACKET.is_match(g) {
				Token::Single(TokenType::Bracket, g.to_string())
			} else if SPACE.is_match(g) {
				Token::Multi(TokenType::Space, g.to_string())
			} else if BREAK.is_match(g) {
				Token::Multi(TokenType::Break, g.to_string())
			} else {
				Token::Multi(TokenType::Text, g.to_string())
			};

			let (complete, incomplete) = context.combine(&current);
			self.push(complete);
			context = incomplete;
		}
		self.push(context);
		self.push(Token::Multi(TokenType::Break, "\n".to_string()));
	}

	fn push(&mut self, token: Token) {
		use Token::*;
		match token {
			Empty => {}
			_ => {
				self.tokens.push(token);
			}
		}
	}
}

#[derive(Debug)]
struct Evaluator {
	defs: HashMap<String, String>,
	in_str: bool,
	strs: Vec<String>,
	op: Option<Token>,
}

impl Evaluator {
	fn new() -> Self {
		Self {
			defs: HashMap::with_capacity(10),
			in_str: false,
			strs: Vec::new(),
			op: None,
		}
	}

	fn eval(&mut self, tokens: Vec<Token>) {
		for (i, token) in tokens.iter().enumerate() {
			// println!("{:?} {:?} {:?} {:?} ", in_str, token.content(), op, cur_str);

			if self.in_str {
				if token.content() == "]" {
					self.in_str = false
				} else {
					self.strs.last_mut().unwrap().push_str(token.content());
				}
			} else if token.content() == "[" {
				self.in_str = true;
				self.strs.push(String::new());
			} else if discriminant(&token.t()) == discriminant(&TokenType::Break) {
				if let Some(t) = &self.op {
					if t.content() == "print" {
						println!("{}", self.strs[0]);
						self.op = None;
						self.strs.clear();
					}
				}
			} else if discriminant(&token.t()) == discriminant(&TokenType::Text) {
				self.op = Some(tokens[i].clone());
			}
		}
	}
}
