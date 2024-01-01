use std::{collections::HashMap, str::FromStr, string::ParseError};

pub mod instruction;
pub mod value_box;

use instruction::Instruction;

struct Block<'a> {
    name: String,
    instructions: Vec<Instruction>,
    next: Option<&'a Block<'a>>,
}

pub struct ScriptObject<'a> {
    blocks: HashMap<String, Block<'a>>,
    entry_point: &'a Block<'a>,
}

impl FromStr for ScriptObject<'_> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}
