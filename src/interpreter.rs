use crate::script_object::{value_box::ValueBox, ScriptObject};

pub struct Interpreter {
    memory: Vec<Option<ValueBox>>,
    head: Option<ValueBox>,
}

// Initializzation methods
impl Default for Interpreter {
    fn default() -> Self {
        Self {
            memory: vec![],
            head: None,
        }
    }
}

impl Interpreter {
    pub fn new(memory: Vec<Option<ValueBox>>) -> Self {
        Self {
            memory: memory,
            head: None,
        }
    }

    pub fn with_memory_size(memory_size: usize) -> Self {
        Self {
            memory: vec![None; memory_size],
            head: None,
        }
    }
}

// Execution methods
impl Interpreter {
    pub fn execute(&self, script: &ScriptObject, inputs: &Vec<ValueBox>) -> Vec<ValueBox> {
        todo!()
    }
}
