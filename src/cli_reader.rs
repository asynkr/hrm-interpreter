use std::{collections::HashMap, env};

use crate::script_object::value_box::ValueBox;

pub struct CommandLineArgs {
    pub input_file: String,
    pub input_values: Vec<ValueBox>,
    pub memory: HashMap<usize, ValueBox>,
    pub max_memory_address: usize,
}

struct CommandLineOption {
    short_name: &'static str,
    long_name: &'static str,
    values_dexcription: &'static str,
    description: &'static str,
    example: &'static str,
    default_value: &'static str,
}

static OPTIONS: [CommandLineOption; 4] = [
    CommandLineOption {
        short_name: "-i",
        long_name: "--inputs",
        values_dexcription: "<value> <value> ...",
        description: "sets the values to be used as input",
        example: "-i 10 20 30 A E F",
        default_value: "no input values",
    },
    CommandLineOption {
        short_name: "-m",
        long_name: "--memory",
        values_dexcription: "<address> <value> <address> <value> ...",
        description: "sets the values to be used as memory",
        example: "-m 0 10 1 A 2 30",
        default_value: "no starting memory values",
    },
    CommandLineOption {
        short_name: "-m",
        long_name: "--memory",
        values_dexcription: "<memory_file>",
        description: "sets the memory from a file (with the same format as the command line version)",
        example: "-m memory.txt",
        default_value: "no starting memory values",
    },
    CommandLineOption {
        short_name: "-M",
        long_name: "--max-mem",
        values_dexcription: "<max_address>",
        description: "sets the maximum memory address. That's the last tile number in the game.",
        example: "-M 24",
        default_value: "no (theoretical) maximum",
    },
];

fn print_help() {
    println!("Human Resource Machine interpreter");
    println!("Get this help: hrm-interpreter.exe -h | --help");
    println!("Usage:         hrm-interpreter.exe <script_file> [options]");
    println!("Options:");
    for option in OPTIONS.iter() {
        let short_name_long_name_and_values = format!("{}, {} {}", option.short_name, option.long_name, option.values_dexcription);
        println!(
            "  {: <55} {}",
            short_name_long_name_and_values, option.description
        );
        println!("  {: <55}   Example: {}", "", option.example);
        println!("  {: <55}   Default: {}", "", option.default_value);
    }
}

pub fn read_args() -> CommandLineArgs {
    let mut args = env::args().skip(1);

    let first_arg = args.next().expect("please supply a script file name");

    if first_arg == "-h" || first_arg == "--help" {
        print_help();
        std::process::exit(0);
    }

    let input_file = first_arg;

    todo!()
}