use super::lang::{Lexeme, Position, Text, Token};
use fsm::Symbols;
use std::ops::{Bound, Range};

pub struct Lexer<'l> {
	text: &'l str,
	tokens: Vec<Token>,
	symbols: Symbols<String>,
	state: State,
	token_start: Position,
	pos: Position,
}

enum State {
	Common,
	BlockComment,
	BlockCommentAst,
	LineComment,
	Operator(char),
	Ident(usize),
	String(usize),
	StringEscSeq(usize),
	Number(usize, u32),
	Decimal(usize),
}

impl<'l> Lexer<'l> {
	pub fn parse(text: &'l str) -> Text {
		let mut lexer = Self {
			text,
			tokens: Vec::new(),
			symbols: Symbols::new(),
			state: State::Common,
			token_start: Position { col: 0, line: 0 },
			pos: Position { col: 1, line: 1 },
		};
		for (index, c) in text.char_indices() {
			while !lexer.next(c, index) {}
			if c == '\n' {
				lexer.pos.col = 0;
				lexer.pos.line += 1;
			}
			lexer.pos.col += 1;
		}
		lexer.finalize();
		Text::new(lexer.tokens, lexer.symbols.into_table())
	}

	fn next(&mut self, c: char, index: usize) -> bool {
		match self.state {
			State::Common => self.next_common(c, index),
			State::BlockComment => self.next_blk_comment(false, c),
			State::BlockCommentAst => self.next_blk_comment(true, c),
			State::LineComment => self.next_line_comment(c),
			State::Operator(first) => self.next_op(first, c),
			State::Ident(start) => self.next_id(c, start..index),
			State::String(start) => self.next_str(c, false, start..index),
			State::StringEscSeq(start) => self.next_str(c, true, start..index),
			State::Number(start, radix) => self.next_num(c, radix, start..index),
			State::Decimal(start) => self.next_decimal(c, start..index),
		}
	}

	fn next_common(&mut self, c: char, index: usize) -> bool {
		self.token_start = self.pos;
		self.state = match c {
			_ if c.is_whitespace() => return true,
			'"' => State::String(index + 1),
			'/' | ':' | '<' | '>' => State::Operator(c),
			'0' => State::Number(index, 0),
			'1'..='9' => State::Number(index, 10),
			_ if c.is_alphabetic() => State::Ident(index),
			_ => {
				return self.push_inclusive(match c {
					'!' => Lexeme::SignExclamation,
					'#' => Lexeme::SignNumber,
					'(' => Lexeme::SignLParen,
					')' => Lexeme::SignRParen,
					'*' => Lexeme::SignAsterisk,
					'+' => Lexeme::SignPlus,
					',' => Lexeme::SignComma,
					'-' => Lexeme::SignMinus,
					'.' => Lexeme::SignFullStop,
					';' => Lexeme::SignSemicolon,
					'=' => Lexeme::SignEquals,
					'?' => Lexeme::SignQuestion,
					_ => Lexeme::Error,
				});
			}
		};
		true
	}

	fn next_op(&mut self, first: char, second: char) -> bool {
		match (first, second) {
			('/', '*') => {
				self.state = State::BlockComment;
				true
			}
			('/', '/') => {
				self.state = State::LineComment;
				true
			}
			_ => {
				self.state = State::Common;
				match (first, second) {
					('/', _) => self.push_exclusive(Lexeme::OpSolidus),
					(':', '=') => self.push_inclusive(Lexeme::OpAssign),
					('<', '=') => self.push_inclusive(Lexeme::OpLessEqual),
					('<', _) => self.push_exclusive(Lexeme::OpLess),
					('>', '=') => self.push_inclusive(Lexeme::OpGreaterEqual),
					('>', _) => self.push_exclusive(Lexeme::OpGreater),
					_ => self.push_exclusive(Lexeme::Error),
				}
			}
		}
	}

	fn next_id(&mut self, c: char, index: Range<usize>) -> bool {
		if c.is_alphanumeric() {
			true
		} else {
			self.state = State::Common;
			self.push_exclusive(match &self.text[index] {
				"begin" => Lexeme::KwBegin,
				"call" => Lexeme::KwCall,
				"const" => Lexeme::KwConst,
				"do" => Lexeme::KwDo,
				"end" => Lexeme::KwEnd,
				"if" => Lexeme::KwIf,
				"odd" => Lexeme::KwOdd,
				"procedure" => Lexeme::KwProcedure,
				"then" => Lexeme::KwThen,
				"var" => Lexeme::KwVar,
				"while" => Lexeme::KwWhile,
				id => return self.push_symbol(id.to_owned()),
			})
		}
	}

	fn next_str(&mut self, c: char, is_escaped: bool, index: Range<usize>) -> bool {
		self.state = if is_escaped {
			State::String(index.start)
		} else {
			match c {
				'"' => {
					self.push_inclusive(Lexeme::String(self.text[index].to_owned()));
					State::Common
				}
				'\\' => State::StringEscSeq(index.start),
				_ => return true,
			}
		};
		true
	}

	fn next_num(&mut self, c: char, mut radix: u32, index: Range<usize>) -> bool {
		if radix == 0 {
			radix = match c {
				'b' => 2,
				'o' => 8,
				'x' => 16,
				_ => 10,
			};
			if radix == 10 {
				self.state = State::Number(index.start, radix);
			} else {
				self.state = State::Number(index.end + 1, radix);
				return true;
			}
		}
		if c.is_digit(radix) {
			true
		} else if c == '.' && radix == 10 {
			self.state = State::Decimal(index.start);
			true
		} else {
			self.state = State::Common;
			self.push_exclusive(match u32::from_str_radix(&self.text[index], radix) {
				Ok(num) => Lexeme::Number32(num),
				Err(_) => Lexeme::Error,
			})
		}
	}

	fn next_decimal(&mut self, c: char, index: Range<usize>) -> bool {
		if c.is_digit(10) {
			true
		} else {
			self.state = State::Common;
			self.push_exclusive(match self.text[index].parse() {
				Ok(num) => Lexeme::Decimal64(num),
				Err(_) => Lexeme::Error,
			})
		}
	}

	fn next_blk_comment(&mut self, after_asterisk: bool, c: char) -> bool {
		if after_asterisk {
			self.state = match c {
				'*' => return true,
				'/' => State::Common,
				_ => State::BlockComment,
			};
		} else if c == '*' {
			self.state = State::BlockCommentAst;
		}
		true
	}

	fn next_line_comment(&mut self, c: char) -> bool {
		if c == '\n' {
			self.state = State::Common;
		}
		true
	}

	fn push_inclusive(&mut self, lexeme: Lexeme) -> bool {
		self.tokens.push(Token::new(
			lexeme,
			self.token_start,
			Bound::Included(self.pos),
		));
		true
	}

	fn push_exclusive(&mut self, lexeme: Lexeme) -> bool {
		self.tokens.push(Token::new(
			lexeme,
			self.token_start,
			Bound::Excluded(self.pos),
		));
		false
	}

	fn push_symbol(&mut self, symbol: String) -> bool {
		self.tokens.push(Token::new(
			Lexeme::Ident(self.symbols.get_or_create_id(symbol)),
			self.token_start,
			Bound::Excluded(self.pos),
		));
		false
	}

	fn finalize(&mut self) {
		match self.state {
			State::Common | State::BlockComment | State::BlockCommentAst | State::LineComment => {
				false
			}
			State::Operator(first) => self.next_op(first, '\0'),
			State::Ident(start) => self.next_id('\0', start..self.text.len()),
			State::String(_) | State::StringEscSeq(_) => self.push_exclusive(Lexeme::Error),
			State::Number(start, radix) => self.next_num('\0', radix, start..self.text.len()),
			State::Decimal(start) => self.next_decimal('\0', start..self.text.len()),
		};
		self.token_start = self.pos;
		self.push_exclusive(Lexeme::Eof);
	}
}
