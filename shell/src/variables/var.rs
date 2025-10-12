#[derive(Debug)]
pub struct Var {
	pub command: String,
	pub flags: Vec<String>,
	pub args: Vec<String>,
}

impl Var {
	pub fn new() -> Self {
		Var {
			command: String::new(),
			flags: Vec::new(),
			args: Vec::new(),
		}
	}
	
}