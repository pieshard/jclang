use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Action {
	pub id: String,
	pub name: String,
	pub object: String,
	pub args: Vec<Argument>,
	pub r#type: String
}

#[derive(Debug, Deserialize)]
pub struct Argument {
	pub id: String,
	pub r#type: String,
	pub values: Option<Vec<String>>
}

pub fn get_actions(json: &str) -> Vec<Action> {
	let actions: Vec<Action> = serde_json::from_str(json).unwrap();
	return actions;
}