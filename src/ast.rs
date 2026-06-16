#[allow(unused)] // keep it until this is usable

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ArgumentList {
	pub posargs: Vec<ExpressionBox>,
	pub kwargs: HashMap<String, ExpressionBox>,
	pub handler: Option<Vec<StatementBox>>
}

#[derive(Debug, Clone)]
pub enum Expression {
	Identifier(String),
	ExplicitVariable {
		scope: String,
		name: String
	},
	NumberLiteral(f32),
	StringLiteral {
		parsing: String,
		text: String
	},
	ArrayLiteral(Vec<ExpressionBox>),
	FunctionCall {
		name: String,
		args: Vec<ExpressionBox>
	},
	MethodCall {
		object: String,
		method: String,
		args: ArgumentList
	}
}

#[derive(Clone)]
pub struct ExpressionBox {
	pub content: Expression,
	pub line: u32
}
impl ExpressionBox {
	pub fn unwrap(self) -> Expression {
		return self.content;
	}
}
impl fmt::Debug for ExpressionBox {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.content.fmt(f)
	}
}

#[derive(Debug, Clone)]
pub enum Statement {
	FunctionDefinition {
		name: String,
		params: Vec<String>,
		body: Vec<StatementBox>
	},
	ProcessDefinition {
		name: String,
		params: Vec<String>,
		body: Vec<StatementBox>
	},
	EventDefinition {
		event: String,
		body: Vec<StatementBox>
	},
	VariableDefinition {
		scope: String,
		name: String,
		value: Option<ExpressionBox>
	},
	VariableSet {
		name: String,
		value: ExpressionBox
	},
	ExpressionStatement(ExpressionBox),
}

#[derive(Clone)]
pub struct StatementBox {
	pub content: Statement,
	pub line: u32
}
impl fmt::Debug for StatementBox {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.content.fmt(f)
	}
}