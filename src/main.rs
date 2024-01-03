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
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });
    script_object.validate().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    let memory = Memory::with_data(args.memory, args.max_memory_address);
    let mut interpreter = Interpreter::new(memory);

    // Execute the script
    match interpreter.execute(&script_object, &args.input_values) {
        Ok(outputs) => {
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
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
