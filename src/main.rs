mod lang;
mod lexer;
use lexer::Lexer;
use std::io::Read;

fn main() {
	let mut program = String::new();
	if let Err(err) = std::io::stdin().read_to_string(&mut program) {
		eprintln!("Error: {}", err);
		std::process::exit(1);
	}
	println!("{:#?}", Lexer::parse(&program));
}
