#[allow(unused)] // keep it until this is usable

use crate::ast::*;
use crate::lexer::Token;
use crate::lexer::TokenType;

use std::collections::HashMap;

macro_rules! error {
	($obj:expr, $line:expr, $($arg:tt)*) => {
		$obj.error($line, format_args!($($arg)*))
	};
}
macro_rules! warn {
	($obj:expr, $line:expr, $($arg:tt)*) => {
		$obj.warn($line, format_args!($($arg)*))
	};
}


pub struct Parser<'a> {
	tokens: Vec<Token>,
	pos: usize,
	source: &'a String,
	file: &'a String
}

impl<'a> Parser<'a> {
	pub fn new(tokens: Vec<Token>, source: &'a String, file: &'a String) -> Self {
		Self { tokens, pos: 0, source, file }
	}


	// token methods
	fn peek(&self) -> &Token {
		return self.tokens.get(self.pos).expect("expected token, but got EOF");
	}
	fn peekv(&self) -> &str {
		return self.peek().value.as_str();
	}
	fn previous(&self) -> &Token {
		return &self.tokens[self.pos - 1];
	}
	fn consume(&mut self) -> &Token {
		self.pos += 1;
		return self.previous();
	}


	// errors
	fn error(&self, line: u32, args: std::fmt::Arguments) -> ! {
		eprintln!("\x1b[91merror\x1b[97m: {}", args);
		eprintln!("\x1b[96m  --> \x1b[0m{}:{}", self.file, line);
		self.printsrc(line);

		eprintln!("compilation aborted due to an error");
		std::process::exit(1);
	}
	fn warn(&self, line: u32, args: std::fmt::Arguments) {
		eprintln!("\x1b[93mwarning\x1b[97m: {}", args);
		eprintln!("\x1b[96m  --> \x1b[0m{}:{}", self.file, line);
		self.printsrc(line);
	}
	fn printsrc(&self, line: u32) {
		// eprintln!("\x1b[36m    |");

		let start = {
			if line <= 2 { 1 } else { line - 1 }
		};
		let finish = line + 2;

		for n in start..finish {
			if let Some(source_line) = self.source.lines().nth(n as usize - 1) {
				let source_line = source_line.replace('\t', "   ");
				if line == n {
					eprintln!("\x1b[96m  {} | \x1b[0m{}", n, source_line);
				} else {
					eprintln!("\x1b[36m  {} | \x1b[0m{}", n, source_line);
				}
			}
		}
		// eprintln!("\x1b[36m    |\n\x1b[0m");
		eprintln!("\x1b[0m");
	}


	// helpers
	fn expect_kind(&mut self, kind: TokenType, info: &str) -> &Token {
		let valid = {
			let token = self.consume();
			if token.kind == kind {
				true
			} else {
				false
			}
		};

		if valid {
			return self.previous();
		}
		let token = self.previous();
		error!(self, token.line,
			"expected {:?} but got {:?} {}",
			kind, token.kind, info
		);
	}
	fn expect_value(&mut self, value: &str, info: &str) -> &Token {
		let valid = {
			let token = self.consume();
			if token.value == value && token.kind != TokenType::String {
				true
			} else {
				false
			}
		};

		if valid {
			return self.previous();
		}
		let token = self.previous();
		error!(self, token.line,
			"expected `{}` but got `{}` {}",
			value, token.value, info
		);
	}


	// per kind
	fn expect_identifier(&mut self, info: &str) -> &Token {
		return self.expect_kind(TokenType::Identifier, info);
	}


	// parsers
	pub fn parse_program(&mut self) -> Vec<StatementBox> {
		let mut stmts = vec![];
		while self.pos < self.tokens.len() {
			stmts.push(self.parse_stmt());
		}
		return stmts;
	}

	fn parse_stmt(&mut self) -> StatementBox {
		if self.peekv() == "_WARNTEST" {
			let line = self.consume().line;
			warn!(self, line, "warning test");
		}

		let t = self.peekv();

		if t == "function" {
			self.consume(); // function
			return self.parse_function_def(true);
		}
		else if t == "process" {
			self.consume(); // process
			return self.parse_function_def(false);
		}
		else if t == "event" {
			self.consume(); // event
			return self.parse_event_def();
		}
		else if t == "line" || t == "local" || t == "game" || t == "save" {
			return self.parse_variable_def();
		}

		let exprbox = self.parse_expr();
		let line = exprbox.line;
		let expr = &exprbox.content;
		match expr {
			Expression::Identifier(_) => {
				if self.peekv() == "=" {
					return self.parse_variable_set(expr.clone(), line);
				}
			},
			_ => {}
		}

		self.expect_value(";", "at the end of a statement");
		let content = Statement::ExpressionStatement(exprbox);
		return StatementBox { content, line };
	}

	//  [line/local/game/save] var <name> [ = <expression> ]
	fn parse_variable_def(&mut self) -> StatementBox {
		static PARSING_DEF: &str = "when parsing variable declaration";

		let scope;
		let name: String;
		let t = self.peekv();
		if t == "line" || t == "local" || t == "game" || t == "save" {
			scope = self.expect_identifier(PARSING_DEF).value.clone();
		} else {
			scope = "default".to_string();
		}
		
		let nametoken = self.consume();
		let line = nametoken.line;
		let kind = nametoken.kind.clone();
		match kind {
			TokenType::Identifier => name = nametoken.value.clone(),
			TokenType::FString => name = nametoken.value.clone(),
			_ => error!(self, line, "expected a variable name but got {:?} {}", kind, PARSING_DEF)
		}

		let value;
		if self.peekv() == "=" {
			self.consume(); // =
			value = Some(self.parse_expr());
		} else {
			value = None;
		}

		self.expect_value(";", "at the end of a statement");
		let content = Statement::VariableDefinition { scope, name, value };
		return StatementBox { content, line };
	}

	//  <name> = <expression>
	fn parse_variable_set(&mut self, identifier: Expression, line: u32) -> StatementBox {
		static PARSING_DEF: &str = "when parsing variable set";

		let name;
		match identifier {
			Expression::Identifier(v) => name = v,
			_ => panic!("unexpected variable set name type")
		}
		self.expect_value("=", PARSING_DEF);
		let value = self.parse_expr();

		self.expect_value(";", "at the end of a statement");
		let content = Statement::VariableSet { name, value };
		return StatementBox { content, line };
	}

	//  function/process <name> ( <param>, ... ) <body>
	fn parse_function_def(&mut self, func: bool) -> StatementBox {
		static PARSING_DEF: &str = "when parsing function declaration";
		static PARSING_PARAM: &str = "when parsing parameters in function declaration";

		let identifier = self.expect_identifier(PARSING_DEF);
		let name = identifier.value.clone();
		let line = identifier.line;

		self.expect_value("(", PARSING_DEF);
		let mut params = vec![];
		loop {
			if self.peekv() == ")" {
				self.expect_value(")", PARSING_PARAM);
				break;
			}
			params.push(self.expect_identifier(PARSING_PARAM).value.clone());
			if self.peekv() == ":" {
				let line = self.consume().line;
				error!(self, line,
					"parameter types have not been implemented yet");
			}
			if self.peekv() != "," {
				self.expect_value(")", PARSING_PARAM);
				break;
			}
		}

		let body = self.parse_body();
		if func {
			let content = Statement::FunctionDefinition { name, params, body };
			return StatementBox { content, line };
		} else {
			let content = Statement::ProcessDefinition { name, params, body };
			return StatementBox { content, line };
		}
	}

	//  event\<<eventname>\> <body>
	fn parse_event_def(&mut self) -> StatementBox {
		static PARSING_DEF: &str = "when parsing event definition";

		self.expect_value("<", PARSING_DEF);
		let event = self.expect_identifier(PARSING_DEF);
		let name = event.value.clone();
		let line = event.line;
		self.expect_value(">", PARSING_DEF);

		let body = self.parse_body();
		let content = Statement::EventDefinition { event: name, body };
		return StatementBox { content, line };
	}

	//  { <stmt>* }
	fn parse_body(&mut self) -> Vec<StatementBox> {
		self.expect_value("{", "when parsing body");
		let mut stmts = vec![];
		loop {
			if self.peekv() == "}" { break }
			stmts.push(self.parse_stmt());
		}
		self.expect_value("}", "when parsing body");
		return stmts;
	}

	fn parse_expr(&mut self) -> ExpressionBox {
		let token = self.peek();
		let line = token.line;

		match token.kind {
			TokenType::Identifier => {
				let name = self.consume().value.clone();

				let t = self.peekv();

				// method calls
				if t == "::" {
					self.consume(); // ::
					let method = self.expect_identifier("when parsing method call").value.clone();
					let args = self.parse_mixed_argument_list();
					let content = Expression::MethodCall {
						object: name,
						method: method,
						args: args
					};
					return ExpressionBox { content, line };
				}

				// function calls
				else if t == "(" {
					let args = self.parse_pos_argument_list();
					let content = Expression::FunctionCall {
						name: name,
						args: args
					};
					return ExpressionBox { content, line };
				}

				else {
					// explicit variable scope
					if self.peek().kind == TokenType::FString {
						// name becomes l/g/s
						let scope = {
							if name == "ln" { "line" }
							else if name == "l" { "local" }
							else if name == "g" { "game" }
							else if name == "s" { "save" }
							else { error!(self, line, "unknown explicit variable scope"); }
						};
						let var = self.consume().value.clone();
						let content = Expression::ExplicitVariable {
							scope: scope.to_string(),
							name: var
						};
						return ExpressionBox { content, line };
					}
					// string parsing
					if self.peek().kind == TokenType::String {
						let parsing = {
							if name == "p" { "plain" }
							else if name == "l" { "legacy" }
							else if name == "m" { "minimessage" }
							else if name == "j" { "json" }
							else { error!(self, line, "unknown text parsing"); }
						};
						let text = self.consume().value.clone();
						let content = Expression::StringLiteral {
							parsing: parsing.to_string(),
							text: text
						};
						return ExpressionBox { content, line };
					}

					// identifier
					let content = Expression::Identifier(name);
					return ExpressionBox { content, line };
				}
			}

			TokenType::Number => {
				let v = &self.consume().value;
				let content = Expression::NumberLiteral(v.parse::<f32>().unwrap());
				return ExpressionBox { content, line };
			}

			TokenType::String => {
				let text = self.consume().value.clone();
				let content = Expression::StringLiteral { parsing: "legacy".to_string(), text: text };
				return ExpressionBox { content, line };
			}
			TokenType::FString => {
				let name = self.consume().value.clone();
				let content = Expression::Identifier(name);
				return ExpressionBox { content, line };
			}

			TokenType::Symbol => {
				// array literals
				if self.peekv() == "[" {
					static PARSING_ARR: &str = "when parsing an array literal";

					self.expect_value("[", PARSING_ARR);
					let mut content = vec![];
					while self.peekv() != "]" {
						let value = self.parse_expr();
						content.push(value);
						if self.peekv() == "," {
							self.consume();
						} else {
							break;
						}
					}
					self.expect_value("]", PARSING_ARR);
					let content = Expression::ArrayLiteral(content);
					return ExpressionBox { content, line };
				}

				// array literals
				if self.peekv() == "(" {
					static PARSING_ARR: &str = "when parsing a handler";

					self.expect_value("(", PARSING_ARR);
					self.expect_value(")", PARSING_ARR);
					self.expect_value("=>", PARSING_ARR);

					let body = self.parse_body();
					let content = Expression::Handler(body);
					return ExpressionBox { content, line };
				}

				else {
					let line = self.peek().line;
					error!(self, line,
						"unexpected '{}' while parsing an expression", self.peekv())
				}
			}

			// _ => {
			// 	let line = self.peek().line;
			// 	error!(self, line,
			// 		"unexpected '{}' while parsing an expression", self.peekv())
			// }
		}
	}

	fn parse_mixed_argument_list(&mut self) -> ArgumentList {
		static PARSING_LIST: &str = "when parsing argument list";
		static PARSING_KW: &str = "when parsing kwarg in argument list";

		self.expect_value("(", PARSING_LIST);

		let mut posargs = vec![];
		let mut krawg_first: Option<ExpressionBox> = None;
		let mut handler = None;

		loop {
			if self.peekv() == ")" {
				self.expect_value(")", PARSING_LIST);
				break;
			} else if self.peekv() == "(" {
				break;
			}

			let argument = self.parse_expr();
			if self.peekv() == "=" {
				assert!(
					matches!(argument.content, Expression::Identifier {..}),
					"kwarg fields can only be defined with identifiers @ line {}",
					argument.line
				);
				krawg_first = Some(argument);
				break;
			}
			posargs.push(argument);
			
			if self.peekv() == "," {
				self.consume();
			} else {
				self.expect_value(")", PARSING_LIST);
				break;
			}
		}

		let mut kwargs: HashMap<String, ExpressionBox> = HashMap::new();
		let mut kwargs_used = false;
		match krawg_first {
			Some(ExpressionBox { content: Expression::Identifier(name), .. }) => {
				self.expect_value("=", PARSING_KW);
				let value = self.parse_expr();
				kwargs.insert(name, value);
				kwargs_used = true;
			},
			None => {},
			// would be normally impossible, handled by an assert earlier
			_ => panic!("unexpected first kwarg name type")
		}
		while kwargs_used {
			if self.peekv() == "," {
				self.consume();
			} else if self.peekv() == "(" {
				break;
			} else {
				self.expect_value(")", PARSING_LIST);
				break;
			}

			let name = self.expect_identifier(PARSING_KW).value.clone();
			self.expect_value("=", PARSING_KW);
			let value = self.parse_expr();
			kwargs.insert(name, value);
		}
		
		if self.peekv() == "(" {
			handler = Some(Box::new(self.parse_expr()));
			self.expect_value(")", PARSING_LIST);
		}
		
		return ArgumentList {
			posargs: posargs,
			kwargs: kwargs,
			handler: handler
		};
	}

	fn parse_pos_argument_list(&mut self) -> Vec<ExpressionBox> {
		self.expect_value("(", "when parsing argument list");

		let mut args = vec![];
		loop {
			if self.peekv() == ")" {
				self.expect_value(")", "when parsing argument list");
				break;
			}
			args.push(self.parse_expr());
			if self.peekv() == "," {
				self.consume();
			} else {
				self.expect_value(")", "when parsing argument list");
				break;
			}
		}
		
		return args;
	}
}