use std::{collections::HashMap, env, fs};

use interpreter::{memory::Memory, Interpreter};
use script_object::{value_box::ValueBox, ScriptObject};

mod interpreter;
mod script_object;

fn main() {
    // Read the script from the file
    let file = env::args().nth(1).expect("please supply a file name");
    let script_content = fs::read_to_string(file).expect("could not read file");

    // Read the command line arguments
    // TODO

    // Objects used to execute the script
    let script_object = script_content.parse::<ScriptObject>().unwrap();
    let memory = Memory::with_data(HashMap::new(), 10);
    let mut interpreter = Interpreter::new(memory);

    // Execute the script
    let outputs = interpreter.execute(
        &script_object,
        &[ValueBox::from(-10), ValueBox::from(20), ValueBox::from(30)],
    );

    // Print the outputs to stdout
    print!(
        "{}",
        outputs
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    );
}
