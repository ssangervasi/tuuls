use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::{BufRead, Write};
use std::mem::discriminant;
use std::path::Path;

use lazy_static::lazy_static;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct CliOptions<W: Write> {
	pub path: String,
	pub stdout: W,
}

pub fn eval<W: Write>(opts: &mut CliOptions<W>) {
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

	Evaluator::new(opts).eval(tokenizer.tokens);
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

	fn is(&self, t: TokenType) -> bool {
		discriminant(&self.t()) == discriminant(&t)
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

#[derive(Debug, Clone)]
enum Statement {
	// Assign(Token),
	Call(Token),
	Empty,
}

#[derive(Debug, Clone)]
enum Expression {
	Collection(Vec<Token>),
	Empty,
}

#[derive(Debug)]
struct Evaluator<'opts, W: Write> {
	defs: HashMap<String, String>,
	exprs: Vec<Expression>,
	expr: Expression,
	strs: Vec<String>,
	statement: Statement,
	options: &'opts mut CliOptions<W>,
}

impl<'opts, W: Write> Evaluator<'opts, W> {
	fn new(options: &'opts mut CliOptions<W>) -> Self {
		Self {
			defs: HashMap::with_capacity(10),
			expr: Expression::Empty,
			exprs: Vec::new(),
			strs: Vec::new(),
			statement: Statement::Empty,
			options,
		}
	}

	fn eval(&mut self, tokens: Vec<Token>) {
		for token in tokens.iter() {
			// println!("{:?} {:?} {:?} {:?} ", in_str, token.content(), op, cur_str);

			if let Expression::Collection(v) = &mut self.expr {
				if token.content() == "]" {
					self.exprs.push(Expression::Collection(v.clone()));
					self.expr = Expression::Empty;
				} else {
					// self.expr.0.push(token);
					v.push(token.clone());
				}
			} else if let Statement::Empty = self.statement {
			} else if token.content() == "[" {
				self.expr = Expression::Collection(Vec::new());
			} else if token.is(TokenType::Break) {
				if let Statement::Call(t) = &self.statement {
					if t.content() == "print" {
						writeln!(self.options.stdout, "{}", join(&self.exprs)).unwrap();
						self.statement = Statement::Empty;
						self.exprs.clear();
					}
				}
			} else if token.is(TokenType::Text) {
				self.statement = Statement::Call(token.clone());
			}
		}
	}
}

fn join(exprs: &[Expression]) -> String {
	exprs
		.iter()
		.map(|expr| match expr {
			Expression::Collection(tokens) => tokens
				.iter()
				.map(|token| token.content())
				.collect::<Vec<&str>>()
				.join(""),
			Expression::Empty => "".to_string(),
		})
		.collect::<Vec<String>>()
		.join("")
}
