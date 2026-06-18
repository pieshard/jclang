mod actions;
mod ast;
mod lexer;
mod parser;
mod codegen;

use std::env;
use std::fs;
use std::time::Instant;
use serde_json::{Value};

use crate::lexer::tokenize;
use crate::parser::Parser;
use crate::codegen::Codegen;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	#[cfg(windows)]
	enable_ansi_support::enable_ansi_support().unwrap();

	let start = Instant::now();

	// parse arguments
	let args: Vec<String> = env::args().collect();
	if args.len() < 2 {
		println!("Usage: jc <filename>");
		std::process::exit(1);
	}
	let chunk = args[1].clone();
	let mut i = 2;
	let mut upload = false;
	let mut output = false;
	while i < args.len() {
		if args[i] == "-u" {
			upload = true;
		}
		if args[i] == "-o" {
			output = true;
		}
		i += 1;
	}

	// read actions
	let json = include_str!("../data/actions.json");
	let actions: Vec<actions::Action> = actions::get_actions(json);

	let durationactions = start.elapsed();

	// read source
	let content: String;
	match fs::read_to_string(&chunk) {
		Ok(buff) => {
			content = buff
		},
		Err(e) => {
			println!("Failed to read file: {}", e);
			std::process::exit(1);
		}
	}

	// compile
	let tokens = tokenize(content.to_string());
	let mut parser = Parser::new(tokens, &content, &chunk);
	let ast = parser.parse_program();
	let mut codegen = Codegen::new(actions, ast, &content, &chunk);
	let json = codegen.generate();
	
	let duration = start.elapsed();
	
	// do something
	if upload {
		println!("Uploading to JustMC...");
		let client = reqwest::Client::new();
		let response = client
			.post("https://m.justmc.ru/api/upload")
			.json(&json)
			.send()
			.await?;

		let body: Value = response.json().await?;
		eprintln!("Use this command to import (replace everything with) your compiled code:");
		eprintln!("	\x1b[97m/module loadUrl force https://m.justmc.ru/api/{}\x1b[0m",
			body.as_object().unwrap().get("id").unwrap().as_str().unwrap());
		eprintln!("This link will expire in 3 minutes!");
	}
	if output {
		println!("{}", serde_json::to_string_pretty(&json).unwrap());
	}

	eprintln!("Parsing actions JSON took: {:?}", durationactions);
	eprintln!("Compilation took: {:?}", duration - durationactions);

	Ok(())
}