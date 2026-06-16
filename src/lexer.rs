#[allow(unused)] // keep it until this is usable

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
	Identifier,
	Number,
	String,
	FString, // `...`
	Symbol
}

#[derive(Debug)]
pub struct Token {
	pub kind: TokenType,
	pub value: String,
	pub line: u32
}

static SYMBOLS: &[&str] = &[
	";", "{", "}", "(", ")", "[", "]", "<", ">",
	",", ".", "::", "//", "="
];

pub fn tokenize(content: String) -> Vec<Token> {
	let mut tokens = vec![];

	let chars = content.as_bytes();
	let length = chars.len();
	let mut i = 0;
	let mut line = 1;

	'chars:
	while i < length {
		let c = chars[i];

		// whitespace
		if c.is_ascii_whitespace() {
			if c == b'\n' {
				line += 1;
			}
			i += 1;
		}

		// string
		else if c == b'"' {
			let start = i;
			i += 1;
			// fetch all bytes until " that haven't been escaped by \
			// (every case has been tested)
			while i < length && (chars[i] != b'"' || chars[i - 1] == b'\\') {
				i += 1;
			}
			i += 1;
			tokens.push(Token {
				kind: TokenType::String,
				value: content[start+1..i-1].to_string(),
				line: line
			});
		}

		// fstring
		else if c == b'`' {
			let start = i;
			i += 1;
			// fetch all bytes until ` that haven't been escaped by \
			// (every case has been tested)
			while i < length && (chars[i] != b'`' || chars[i - 1] == b'\\') {
				i += 1;
			}
			i += 1;
			tokens.push(Token {
				kind: TokenType::FString,
				value: content[start+1..i-1].to_string(),
				line: line
			});
		}

		// numbers
		else if c.is_ascii_digit() {
			let start = i;
			while i < length && chars[i].is_ascii_digit() {
				i += 1;
			}
			if chars[i] == b'.' {
				i += 1;
				// fetch decimal part
				while i < length && chars[i].is_ascii_digit() {
					i += 1;
				}
			}
			tokens.push(Token {
				kind: TokenType::Number,
				value: content[start..i].to_string(),
				line: line
			});
		}

		// identifiers
		else if c.is_ascii_alphanumeric() || c == b'_' {
			let start = i;
			while i < length && (chars[i].is_ascii_alphanumeric() || chars[i] == b'_') {
				i += 1;
			}
			tokens.push(Token {
				kind: TokenType::Identifier,
				value: content[start..i].to_string(),
				line: line
			});
		}

		// symbols
		else {
			for symbol in SYMBOLS {
				let start = i;
				let end = i + symbol.len();
				if end > length { continue }
				if content[start..end] != **symbol { continue }

				i = end;

				// comments
				let token = &content[start..i];
				if token == "//" {
					while i < length && chars[i] != b'\n' {
						i += 1;
					}
					continue 'chars;
				}

				tokens.push(Token {
					kind: TokenType::Symbol,
					value: token.to_string(),
					line: line
				});
				continue 'chars;
			}

			panic!("Unknown character {:?} @ line {}", c as char, line);
		}
	}

	return tokens;
}