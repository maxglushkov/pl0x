use std::ops::Bound;

#[derive(Debug)]
pub struct Text {
	symbols: Vec<String>,
	tokens: Vec<Token>,
}

#[derive(Debug)]
pub struct Token {
	lexeme: Lexeme,
	start: Position,
	end: Bound<Position>,
}

#[derive(Debug)]
pub enum Lexeme {
	SignExclamation,
	SignNumber,
	SignLParen,
	SignRParen,
	SignAsterisk,
	SignPlus,
	SignComma,
	SignMinus,
	SignFullStop,
	SignSemicolon,
	SignEquals,
	SignQuestion,
	OpSolidus,
	OpAssign,
	OpLess,
	OpLessEqual,
	OpGreater,
	OpGreaterEqual,
	KwBegin,
	KwCall,
	KwConst,
	KwDo,
	KwEnd,
	KwIf,
	KwOdd,
	KwProcedure,
	KwThen,
	KwVar,
	KwWhile,
	Ident(usize),
	String(String),
	Number32(u32),
	Decimal64(f64),
	Error,
	Eof,
}

#[derive(Clone, Copy, Debug)]
pub struct Position {
	pub col: usize,
	pub line: usize,
}

impl Text {
	pub fn new(tokens: Vec<Token>, symbols: Vec<String>) -> Self {
		Self { symbols, tokens }
	}
}

impl Token {
	pub fn new(lexeme: Lexeme, start: Position, end: Bound<Position>) -> Self {
		Self { lexeme, start, end }
	}
}
