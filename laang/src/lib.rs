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

	fn is_open(&self) -> bool {
		self.content() == "["
	}

	fn is_close(&self) -> bool {
		self.content() == "]"
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
	Empty,
	Assign(Token),
	Call(Token),
}

#[derive(Debug, Clone)]
enum Expression {
	Empty,
	Lookup(Token),
	Collection(Vec<Token>),
}

#[derive(Debug, Clone)]
enum Phase {
	Statement,
	Expressions,
}

#[derive(Debug)]
struct Evaluator<'opts, W: Write> {
	defs: HashMap<String, String>,
	phase: Phase,
	expression: Expression,
	expressions: Vec<Expression>,
	statement: Statement,
	options: &'opts mut CliOptions<W>,
}

impl<'opts, W: Write> Evaluator<'opts, W> {
	fn new(options: &'opts mut CliOptions<W>) -> Self {
		Self {
			defs: HashMap::with_capacity(10),
			phase: Phase::Statement,
			expression: Expression::Empty,
			expressions: Vec::new(),
			statement: Statement::Empty,
			options,
		}
	}

	fn eval(&mut self, tokens: Vec<Token>) {
		for token in tokens.iter() {
			// When we reach a break, evaluate it.
			if token.is(TokenType::Break) {
				println!("---------");
				println!("Stmt: {:?}", self.statement);
				println!("Exprs: {:?}", self.expressions);
				println!("Defs: {:?}", self.defs);

				if let Statement::Assign(t) = &self.statement {
					self.defs.insert(t.content().to_string(), self.join());
				} else if let Statement::Call(t) = &self.statement {
					if t.content() == "print" {
						writeln!(self.options.stdout, "{}", self.join()).unwrap();
					}
				}

				self.phase = Phase::Statement;
				self.statement = Statement::Empty;
				self.expression = Expression::Empty;
				self.expressions.clear();
			// First we need a statement.
			} else if let Phase::Statement = self.phase {
				if let Statement::Empty = self.statement {
					if token.is(TokenType::Text) {
						self.statement = Statement::Call(token.clone());
						self.phase = Phase::Expressions;
					} else if token.is_open() {
						self.statement = Statement::Assign(Token::Empty);
					}
				// Then we need to fill an assignment statement.
				} else if let Statement::Assign(assignment_token) = &self.statement {
					if let Token::Empty = assignment_token {
						if token.is(TokenType::Text) {
							self.statement = Statement::Assign(token.clone());
						}
					} else if token.is_close() {
						self.phase = Phase::Expressions;
					}
				}
			// Then we need an expression.
			} else if let Phase::Expressions = self.phase {
				if let Expression::Empty = self.expression {
					if token.is(TokenType::Text) {
						self.expressions.push(Expression::Lookup(token.clone()));
						self.expression = Expression::Empty;
					} else if token.is_open() {
						self.expression = Expression::Collection(Vec::new());
					}
				// We continue building an expression.
				} else if let Expression::Collection(expression_tokens) = &mut self.expression {
					if token.is_close() {
						self
							.expressions
							.push(Expression::Collection(expression_tokens.clone()));
						self.expression = Expression::Empty;
					} else {
						expression_tokens.push(token.clone());
					}
				}
			}
		}
	}

	fn join(&self) -> String {
		self
			.expressions
			.iter()
			.map(|expr| match expr {
				Expression::Collection(tokens) => tokens
					.iter()
					.map(|token| token.content())
					.collect::<Vec<&str>>()
					.join(""),
				Expression::Lookup(t) => {
					let s: String = self
						.defs
						.get(t.content())
						.unwrap_or_else(|| panic!("Invalid lookup: {}", t.content()))
						.to_string();
					s
				}
				Expression::Empty => "".to_string(),
			})
			.collect::<Vec<String>>()
			.join("")
	}
}
