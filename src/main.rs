use interpreter::{memory::Memory, Interpreter};
use script_object::ScriptObject;

mod cli_reader;
mod interpreter;
mod script_object;

fn main() {
    // Read the command line arguments
    let args = cli_reader::read_args();

    // Objects used to execute the script
    let script_object = args
        .script_file
        .parse::<ScriptObject>()
        .unwrap()
        .validate_or_panic();
    let memory = Memory::with_data(args.memory, args.max_memory_address);
    let mut interpreter = Interpreter::new(memory);

    // Execute the script
    let outputs = interpreter.execute(&script_object, &args.input_values);

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
