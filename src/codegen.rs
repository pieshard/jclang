#[allow(unused)] // keep it until this is usable

use crate::actions::*;
use crate::ast::*;

use serde_json::{json, Value};
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
macro_rules! note {
	($obj:expr, $line:expr, $($arg:tt)*) => {
		$obj.note($line, format_args!($($arg)*))
	};
}

macro_rules! action {
    ($($json:tt)*) => {
        Some(json!($($json)*))
    };
}
macro_rules! add_action {
    ($nodes:expr, $($json:tt)*) => {
        let action = json!($($json)*);
        $nodes.push(action);
    };
}

fn find_action<'a>(actions: &'a Vec<Action>, object: &str, name: &str) -> Option<&'a Action> {
	actions.iter().find(|action| {
		action.object == object && action.name == name
	})
}
fn find_arg<'a>(action: &'a Action, id: &str) -> Option<&'a Argument> {
	action.args.iter().find(|arg| {
		arg.id == id
	})
}

fn expect_number(arg: Option<&ExpressionBox>) -> Option<f32> {
	if let Some(arg) = arg {
		match arg.content {
			Expression::NumberLiteral(value) => Some(value),
			_ => None
		}
	} else {
		return None;
	}
}
fn expect_text(arg: Option<&ExpressionBox>) -> Option<String> {
	if let Some(arg) = arg {
		match &arg.content {
			Expression::StringLiteral { parsing: _, text } => Some(text.to_string()),
			_ => None
		}
	} else {
		return None;
	}
}


enum CValue {
	Variable {
		scope: String,
		name: String
	},
	Text {
		parsing: String,
		text: String
	},
	Number(f32),
	Array(Vec<CValue>),
	Null,
	Enum(String),
	GameValue(String),
	// fabrics
	Location(f32, f32, f32, f32, f32),
	Vector(f32, f32, f32),
	Sound {
		sound: String,
		pitch: f32,
		volume: f32,
		source: String
	},
	Particle {
		particle: String,
		count: f32,
		xspread: f32,
		yspread: f32
	},
	Potion {
		potion: String,
		amplifier: f32,
		duration: f32
	}
}
impl CValue {
	fn finalize(&self) -> Value {
		match self {
			CValue::Variable { scope, name } => json!({
				"type": "variable",
				"scope": scope,
				"variable": name
			}),
			CValue::Text { parsing, text } => json!({
				"type": "text",
				"parsing": parsing,
				"text": text
			}),
			CValue::Number(value) => json!({
				"type": "number",
				"number": value
			}),
			CValue::Array(array) => {
				let mut values = vec![];
				for element in array {
					values.push(element.finalize());
				}
				return json!({
					"type": "array",
					"values": values
				});
			},
			CValue::Null => Value::Null,
			CValue::Enum(name) => json!({
				"type": "enum",
				"enum": name
			}),
			CValue::GameValue(value) => json!({
				"type": "game_value",
				"game_value": value
			}),

			// fabrics
			CValue::Location(x, y, z, yaw, pitch) => json!({
				"type": "location",
				"x": x, "y": y, "z": z, "yaw": yaw, "pitch": pitch
			}),
			CValue::Vector(x, y, z) => json!({
				"type": "vector",
				"x": x, "y": y, "z": z
			}),
			CValue::Sound { sound, pitch, volume, source } => json!({
				"type": "sound",
				"sound": sound,
				"pitch": pitch,
				"volume": volume,
				"source": source
			}),
			CValue::Particle { particle, count, xspread, yspread } => json!({
				"type": "particle",
				"particle_type": particle,
				"count": count,
				"xspread": xspread,
				"yspread": yspread
			}),
			CValue::Potion { potion, amplifier, duration } => json!({
				"type": "potion",
				"potion": potion,
				"amplifier": amplifier,
				"duration": duration
			})

			// _ => unimplemented!(),
		}
	}
}

#[derive(Clone)]
struct DefinedVariable {
	scope: String,
	line: u32
}

pub struct Codegen<'a> {
	actions: Vec<Action>,
	statements: Vec<StatementBox>,
	global: HashMap<String, DefinedVariable>, // varscope
	funcs: HashMap<String, HashMap<String, DefinedVariable>>, // functions and their varscopes
	procs: Vec<String>,
	source: &'a String,
	file: &'a String,
	pos: u32
}

impl<'a> Codegen<'a> {
	pub fn new(actions: Vec<Action>, stmts: Vec<StatementBox>, source: &'a String, file: &'a String) -> Self {
		Self {
			actions: actions,
			statements: stmts,
			global: HashMap::new(),
			funcs: HashMap::new(),
			procs: vec![],
			source: source,
			file: file,
			pos: 0
		}
	}

	// errors
	fn error(&self, line: u32, args: std::fmt::Arguments) -> ! {
		eprintln!("\x1b[0m");
		eprintln!("\x1b[91merror\x1b[97m: {}", args);
		eprintln!("\x1b[96m  --> \x1b[0m{}:{}", self.file, line);
		self.printsrc(line);

		eprintln!("\ncompilation aborted due to an error");
		std::process::exit(1);
	}
	fn warn(&self, line: u32, args: std::fmt::Arguments) {
		eprintln!("\x1b[0m");
		eprintln!("\x1b[93mwarning\x1b[97m: {}", args);
		eprintln!("\x1b[96m  --> \x1b[0m{}:{}", self.file, line);
		self.printsrc(line);
	}
	fn note(&self, line: u32, args: std::fmt::Arguments) {
		eprintln!("\x1b[96mnote\x1b[97m: {}", args);
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
	}


	// generators
	fn generate_top_statement(&mut self, stmtbox: &StatementBox) -> Option<Value> {
		match &stmtbox.content {
			Statement::FunctionDefinition {name, params: _, body} => {
				let mut varscope: HashMap<String, DefinedVariable> = HashMap::new();
				let operations = self.generate_body(body, &mut varscope);
				self.pos += 1;
				self.funcs.insert(name.to_string(), varscope.clone());
				return action!({
					"type": "function",
					"position": self.pos - 1,
					"name": name,
					"operations": operations
				});
			},

			Statement::ProcessDefinition {name, params: _, body} => {
				let mut varscope: HashMap<String, DefinedVariable> = HashMap::new();
				let operations = self.generate_body(body, &mut varscope);
				self.pos += 1;
				self.procs.push(name.to_string());
				return action!({
					"type": "process",
					"position": self.pos - 1,
					"name": name,
					"operations": operations
				});
			},

			Statement::EventDefinition {event, body} => {
				let mut varscope: HashMap<String, DefinedVariable> = HashMap::new();
				let operations = self.generate_body(body, &mut varscope);
				self.pos += 1;
				return action!({
					"type": "event",
					"position": self.pos - 1,
					"event": event,
					"operations": operations
				});
			},

			Statement::VariableDefinition {scope, name, value} => {
				let scope = {
					if scope == "default" { "line" } else { scope }
				};
				if let Some(_) = value {
					error!(self, stmtbox.line, "variables cannot be defined outside of a handler (headless action)");
				}
				let name = name.clone();
				if let Some(var) = self.global.get(&name) {
					error!(self, stmtbox.line, "variable `{}` has already been declared at line {}", name, var.line);
				}
				self.global.insert(name, DefinedVariable {
					scope: scope.to_string(),
					line: stmtbox.line
				});
				return None;
			},

			Statement::VariableSet {name: _, value: _} => {
				error!(self, stmtbox.line, "variables cannot be set outside of a handler (headless action)");
			}
			Statement::ExpressionStatement(_) => {
				error!(self, stmtbox.line, "expression statements cannot be used outside of a handler (headless action)");
			}
		}
	}

	fn generate_statement(&mut self,
		stmtbox: &StatementBox,
		varscope: &mut HashMap<String, DefinedVariable>,
		mut nodes: &mut Vec<Value>
	) {
		match &stmtbox.content {
			Statement::FunctionDefinition {name: _, params: _, body: _} => {
				error!(self, stmtbox.line, "functions cannot be defined inside of a handler");
			},
			Statement::ProcessDefinition {name: _, params: _, body: _} => {
				error!(self, stmtbox.line, "functions cannot be defined inside of a handler");
			},
			Statement::EventDefinition {event: _, body: _} => {
				error!(self, stmtbox.line, "functions cannot be defined inside of a handler");
			},

			Statement::VariableDefinition {scope, name, value} => {
				let scope = {
					if scope == "default" { "line" } else { scope }
				};
				
				let name = name.clone();
				if let Some(var) = self.global.get(&name) {
					error!(self, stmtbox.line, "variable `{}` has already been declared at line {}", name, var.line);
				} else if let Some(var) = varscope.get(&name) {
					error!(self, stmtbox.line, "variable `{}` has already been declared at line {}", name, var.line);
				}

				varscope.insert(name.clone(), DefinedVariable {
					scope: scope.to_string(),
					line: stmtbox.line
				});

				if let Some(expr) = value {
					add_action!(nodes, {
						"action": "set_variable_value",
						"values": [
							{
								"name": "variable",
								"value": {"type": "variable", "scope": scope, "variable": name}
							},
							{
								"name": "value",
								"value": self.generate_value(varscope, expr, &mut nodes).finalize()
							}
						]
					});
				}
			},

			Statement::VariableSet {name, value} => {
				let name = name.clone();
				let scope;
				if let Some(var) = self.global.get(&name) {
					scope = var.scope.clone();
				} else if let Some(var) = varscope.get(&name) {
					scope = var.scope.clone();
				} else {
					error!(self, stmtbox.line, "undefined variable {}", name);
				}

				add_action!(nodes, {
					"action": "set_variable_value",
					"values": [
						{
							"name": "variable",
							"value": {"type": "variable", "scope": scope, "variable": name}
						},
						{
							"name": "value",
							"value": self.generate_value(varscope, value, &mut nodes).finalize()
						}
					]
				});
			},

			Statement::ExpressionStatement(expression) => {
				self.generate_exprstmt(varscope, expression, &mut nodes);
			}
		}
	}

	fn generate_exprstmt(&mut self,
		varscope: &mut HashMap<String, DefinedVariable>,
		exprbox: &ExpressionBox,
		mut nodes: &mut Vec<Value>
	) {
		let line = exprbox.line;
		let expr = exprbox.clone().unwrap();

		match expr {
			Expression::FunctionCall {name, args: _} => {
				if let Some(fvarscope) = self.funcs.get(&name) {
					for (key, value) in fvarscope.into_iter() {
						if let Some(redefined) = varscope.get(key) {
							warn!(self, line, "variable `{}` (defined at line {}) is possibly changed from calling {}",
								key, redefined.line, name);
							note!(self, value.line, "redefined here")
						} else {
							varscope.insert(key.to_string(), value.clone());
						}
					}

					add_action!(nodes, {
						"action": "call_function",
						"values": [
							{
								"name": "function_name",
								"value": {"type": "text", "text": name, "parsing": "plain"}
							}
						]
					});
				} else if self.procs.contains(&name) {
					add_action!(nodes, {
						"action": "start_process",
						"values": [
							{
								"name": "process_name",
								"value": {"type": "text", "text": name, "parsing": "plain"}
							}
						]
					});
				} else {
					error!(self, line, "undefined function call");
				}
			},

			Expression::MethodCall { object, method, invert, selection, args } => {
				let optaction = find_action(&self.actions, &object, &method);
				let action;
				match optaction {
					Some(someaction) => action = someaction,
					None => error!(self, line, "undefined action call"),
				}

				if action.args.len() < args.posargs.len() {
					error!(self, line, "argument count mismatch (expected {} but got {})",
						action.args.len(), args.posargs.len());
				}

				let mut values = vec![];
				for (i, expr) in args.posargs.iter().enumerate() {
					let arg = &action.args[i];
					let key = arg.id.clone();
					let value = self.generate_val_or_enum(varscope, &expr, &mut nodes, &arg, &key);

					values.push(json!({
						"name": key,
						"value": value.finalize()
					}));
				}
				for (key, expr) in args.kwargs {
					let argopt = find_arg(&action, &key);
					let arg;
					match argopt {
						Some(argsome) => arg = argsome,
						None => error!(self, line, "key `{}` not found in action arguments", key),
					}
					let value = self.generate_val_or_enum(varscope, &expr, &mut nodes, &arg, &key);

					values.push(json!({
						"name": key.clone(),
						"value": value.finalize()
					}));
				}

				let select;
				if let Some(selectionopt) = selection {
					let selectionjson = json!({
						"type": selectionopt
					});
					select = Some(serde_json::to_string(&selectionjson).expect("failed to serialize selection json"));
				} else {
					select = None;
				}

				if let Some(handler) = args.handler {
					add_action!(nodes, {
						"action": action.id,
						"values": values,
						"selection": select,
						"is_inverted": invert,
						"operations": self.generate_body(&handler, varscope)
					});
				} else {
					add_action!(nodes, {
						"action": action.id,
						"values": values,
						"selection": select,
						"is_inverted": invert
					});
				}
			},

			_ => {
				warn!(self, line, "expression statement does not yield an action");
				self.generate_value(varscope, exprbox, &mut nodes);
			}
		}
	}

	fn generate_val_or_enum(&self, 
		varscope: &mut HashMap<String, DefinedVariable>,
		expr: &ExpressionBox,
		mut nodes: &mut Vec<Value>,
		arg: &Argument,
		key: &str
	) -> CValue {
		let line = expr.line;
		if arg.r#type == "enum" {
			match &expr.content {
				Expression::StringLiteral { text, parsing: _ } => {
					let enumexpr = CValue::Enum(text.clone());

					// check if the enum is present
					// (values is always present when type is enum)
					if !arg.values.as_ref().unwrap().contains(&text) {
						error!(self, line, "enum `{}` is not valid in key `{}`", text, key);
					}
					return enumexpr;
				},
				_ => error!(self, line, "expected string literal in place of enum (in key `{}`)", key),
			}
		} else {
			self.generate_value(varscope, &expr, &mut nodes)
		}
	}

	fn generate_value(&self,
		varscope: &mut HashMap<String, DefinedVariable>,
		exprbox: &ExpressionBox,
		nodes: &mut Vec<Value>
	) -> CValue {
		let line = exprbox.line;
		let expr = exprbox.clone().content;

		match expr {
			Expression::Identifier(name) => {
				if let Some(var) = self.global.get(&name) {
					return CValue::Variable { scope: var.scope.clone(), name: name };
				} else if let Some(var) = varscope.get(&name) {
					return CValue::Variable { scope: var.scope.clone(), name: name };
				}
				error!(self, line, "undefined variable {}", name);
			},
			Expression::NumberLiteral(value) => CValue::Number(value),
			Expression::StringLiteral { parsing, text } => CValue::Text { parsing, text },
			Expression::ArrayLiteral(array) => {
				let mut values = vec![];
				for value in array {
					values.push(self.generate_value(varscope, &value, nodes));
				}
				return CValue::Array(values);
			},
			Expression::NullLiteral => CValue::Null,

			Expression::ExplicitVariable { scope, name } => CValue::Variable { scope, name },

			Expression::FunctionCall { name, args } => {
				if name == "location".to_string() { self.generate_location(args, line) }
				else if name == "vector".to_string() { self.generate_vector(args, line) }
				else {
					error!(self, line, "cannot use function calls as values");
				}
			},

			Expression::MethodCall { .. } => {
				error!(self, line, "cannot use action calls as values");
			},
			Expression::Value(value) => CValue::GameValue(value)

			// _ => unimplemented!()
		}
	}


	// fabric generators
	fn required_number(&self, arg: Option<&ExpressionBox>, line: u32, help: &str) -> f32 {
		if let Some(value) = expect_number(arg) { value }
		else { error!(self, line, "{}", help) }
	}
	fn optional_number(&self, arg: Option<&ExpressionBox>, default: f32) -> f32 {
		if let Some(value) = expect_number(arg) { value }
		else { default }
	}
	fn required_text(&self, arg: Option<&ExpressionBox>, line: u32, help: &str) -> String {
		if let Some(value) = expect_text(arg) { value }
		else { error!(self, line, "{}", help) }
	}
	fn optional_text(&self, arg: Option<&ExpressionBox>, default: String) -> String {
		if let Some(value) = expect_text(arg) { value }
		else { default }
	}

	fn generate_location(&self, args: Vec<ExpressionBox>, line: u32) -> CValue {
		let x = self.required_number(args.get(0), line, "expected x (number) at argument №1");
		let y = self.required_number(args.get(1), line, "expected y (number) at argument №2");
		let z = self.required_number(args.get(2), line, "expected z (number) at argument №3");
		let yaw = self.optional_number(args.get(3), 0.0);
		let pitch = self.optional_number(args.get(4), 0.0);
		return CValue::Location(x, y, z, yaw, pitch);
	}
	fn generate_vector(&self, args: Vec<ExpressionBox>, line: u32) -> CValue {
		let x = self.required_number(args.get(0), line, "expected x (number) at argument №1");
		let y = self.required_number(args.get(1), line, "expected y (number) at argument №2");
		let z = self.required_number(args.get(2), line, "expected z (number) at argument №3");
		return CValue::Vector(x, y, z);
	}
	fn generate_sound(&self, args: Vec<ExpressionBox>, line: u32) -> CValue {
		let sound = self.required_text(args.get(0), line, "expected sound (text) at argument №1");
		let pitch = self.optional_number(args.get(1), 1.0);
		let volume = self.optional_number(args.get(2), 1.0);
		let source = self.optional_text(args.get(3), "MASTER".to_string());
		return CValue::Sound { sound, pitch, volume, source };
	}
	fn generate_particle(&self, args: Vec<ExpressionBox>, line: u32) -> CValue {
		let particle = self.required_text(args.get(0), line, "expected particle (text) at argument №1");
		let count = self.optional_number(args.get(1), 1.0);
		let xspread = self.optional_number(args.get(2), 0.0);
		let yspread = self.optional_number(args.get(3), 0.0);
		return CValue::Particle { particle, count, xspread, yspread };
	}
	fn generate_potion(&self, args: Vec<ExpressionBox>, line: u32) -> CValue {
		let potion = self.required_text(args.get(0), line, "expected potion (text) at argument №1");
		let amplifier = self.optional_number(args.get(1), 0.0);
		let duration = self.optional_number(args.get(2), -1.0);
		return CValue::Potion { potion, amplifier, duration };
	}


	fn generate_body(&mut self,
		body: &Vec<StatementBox>,
		varscope: &mut HashMap<String, DefinedVariable>
	) -> Value {
		let mut nodes = vec![];
		for statement in body.iter() {
			self.generate_statement(statement, varscope, &mut nodes);
		}
		return Value::Array(nodes);
	}

	pub fn generate(&mut self) -> Value {
		let statements = std::mem::take(&mut self.statements);
		let mut nodes = vec![];
		for statement in statements.iter() {
			let node = self.generate_top_statement(statement);
			match node {
				Some(expr) => nodes.push(expr),
				None => {}
			}
		}

		return json!({
			"handlers": Value::Array(nodes)
		});
	}
}