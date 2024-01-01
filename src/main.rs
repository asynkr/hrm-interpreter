use std::{env, fs};

use interpreter::Interpreter;
use script_object::{value_box::ValueBox, ScriptObject};

mod interpreter;
mod script_object;

fn main() {
    let file = env::args().nth(1).expect("please supply a file name");
    let script_content = fs::read_to_string(file).expect("could not read file");

    let script_object = script_content.parse::<ScriptObject>().unwrap();
    let mut interpreter = Interpreter::default();
    let outputs = interpreter.execute(
        &script_object,
        &[ValueBox::from(-10), ValueBox::from(20), ValueBox::from(30)],
    );

    print!(
        "{}",
        outputs
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    );
}
