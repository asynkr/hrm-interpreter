use std::{env, fs};

use interpreter::Interpreter;
use script_object::{ScriptObject, value_box::ValueBox};

mod interpreter;
mod script_object;

fn main() {
    let file = env::args().nth(1).expect("please supply a file name");
    let script_content = fs::read_to_string(file).expect("could not read file");

    let script_object = script_content.parse::<ScriptObject>().unwrap();
    let interpreter = Interpreter::default();
    let outputs = interpreter.execute(&script_object, &vec![ValueBox::from(-10), ValueBox::from(20), ValueBox::from(30)]);

    print!("{}", outputs.iter().map(|value| value.to_string()).collect::<Vec<String>>().join(" "));
}
