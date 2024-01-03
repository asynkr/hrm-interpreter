use std::{collections::HashMap, env, error::Error, fs, str::FromStr};

use crate::script_object::value_box::ValueBox;

#[derive(Debug)]
pub struct CommandLineArgs {
    pub script_file: String,
    pub input_values: Vec<ValueBox>,
    pub memory: HashMap<usize, ValueBox>,
    pub max_memory_address: usize,
}

enum CommandLineOption {
    InputValues,
    Memory,
    MaxMemoryAddress,
}

impl CommandLineArgs {
    fn default(script_file: String) -> Self {
        Self {
            script_file,
            input_values: Vec::new(),
            memory: HashMap::new(),
            max_memory_address: usize::MAX,
        }
    }
}

// Enum methods
impl CommandLineOption {
    fn all_options() -> [CommandLineOption; 3] {
        [Self::InputValues, Self::Memory, Self::MaxMemoryAddress]
    }
}

impl FromStr for CommandLineOption {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-i" | "--inputs" => Ok(Self::InputValues),
            "-m" | "--memory" => Ok(Self::Memory),
            "-M" | "--max-mem" => Ok(Self::MaxMemoryAddress),
            _ => Err(format!("Invalid option: {}", s).into()),
        }
    }
}

// Element methods
impl CommandLineOption {
    fn short_name(&self) -> &'static str {
        match self {
            Self::InputValues => "-i",
            Self::Memory => "-m",
            Self::MaxMemoryAddress => "-M",
        }
    }

    fn long_name(&self) -> &'static str {
        match self {
            Self::InputValues => "--inputs",
            Self::Memory => "--memory",
            Self::MaxMemoryAddress => "--max-mem",
        }
    }

    fn values_description(&self) -> &'static str {
        match self {
            Self::InputValues => "<value> <value>...",
            Self::Memory => "<address> <value>... | <memory_file>",
            Self::MaxMemoryAddress => "<max_address>",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::InputValues => "sets the values to be used as input",
            Self::Memory => "sets the values to be used as memory",
            Self::MaxMemoryAddress => {
                "sets the maximum memory address. That's the last tile number in the game."
            }
        }
    }

    fn example(&self) -> &'static str {
        match self {
            Self::InputValues => "-i 10 20 30 A E F",
            Self::Memory => "-m 0 10 1 A 2 30 | -m memory.txt",
            Self::MaxMemoryAddress => "-M 24",
        }
    }

    fn default_value(&self) -> &'static str {
        match self {
            Self::InputValues => "no input values",
            Self::Memory => "no starting memory values",
            Self::MaxMemoryAddress => "no (theoretical) maximum",
        }
    }

    fn handle_args(&self, option_args: &Vec<String>, command_line_args: &mut CommandLineArgs) {
        match self {
            Self::InputValues => {
                for arg in option_args {
                    command_line_args.input_values.push(
                        arg.parse::<ValueBox>()
                            .unwrap_or_else(|_| panic!("Invalid input value: {}", arg)),
                    );
                }
            }
            Self::Memory => {
                let args = if option_args.len() == 1 {
                    let memory_file = option_args.first().unwrap().clone();
                    let memory_content = fs::read_to_string(memory_file.clone())
                        .unwrap_or_else(|_| panic!("Could not read file {}", memory_file));
                    let memory_content = memory_content.lines().collect::<Vec<&str>>().join(" ");
                    memory_content
                        .split(' ')
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                } else {
                    option_args
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                };

                if args.len() % 2 != 0 {
                    panic!("Invalid memory arguments: expected an even number of arguments (couples of address and value)");
                }

                for i in 0..args.len() / 2 {
                    let address = args[i * 2]
                        .parse::<usize>()
                        .unwrap_or_else(|_| panic!("Invalid memory address: {}", args[i * 2]));
                    let value = args[i * 2 + 1]
                        .parse::<ValueBox>()
                        .unwrap_or_else(|_| panic!("Invalid memory value: {}", args[i * 2 + 1]));
                    command_line_args.memory.insert(address, value);
                }
            }
            Self::MaxMemoryAddress => {
                let max_memory_address = option_args[0]
                    .parse::<usize>()
                    .unwrap_or_else(|_| panic!("Invalid max memory address: {}", option_args[0]));
                command_line_args.max_memory_address = max_memory_address;
            }
        }
    }
}

fn print_help() {
    println!("Human Resource Machine interpreter");
    println!("Get this help: hrm-interpreter.exe -h | --help");
    println!("Usage:         hrm-interpreter.exe <script_file> [options]");
    println!("Options:");
    for option in CommandLineOption::all_options() {
        let short_name_long_name_and_values = format!(
            "{}, {} {}",
            option.short_name(),
            option.long_name(),
            option.values_description()
        );
        println!(
            "  {: <55} {}",
            short_name_long_name_and_values,
            option.description()
        );
        println!("  {: <55}   Example: {}", "", option.example());
        println!("  {: <55}   Default: {}", "", option.default_value());
    }
}

pub fn read_args() -> CommandLineArgs {
    let mut args = env::args().skip(1);

    let first_arg = args.next().unwrap_or_else(|| {
        print_help();
        std::process::exit(1);
    });

    if first_arg == "-h" || first_arg == "--help" {
        print_help();
        std::process::exit(0);
    }

    let script_file = fs::read_to_string(first_arg.clone())
        .unwrap_or_else(|_| panic!("Could not read file {}", first_arg));

    let mut option = match args.next() {
        Some(option) => Some(
            option
                .parse::<CommandLineOption>()
                .unwrap_or_else(|_| panic!("Invalid option: {}. See '-h' for help", option)),
        ),
        None => {
            // No options, use default values
            return CommandLineArgs::default(script_file);
        }
    };

    let mut command_line_args = CommandLineArgs::default(script_file);

    while option.is_some() {
        let mut option_args: Vec<String> = Vec::new();

        loop {
            let next_arg = args.next();

            if next_arg.is_none() {
                // No more arguments
                option
                    .unwrap()
                    .handle_args(&option_args, &mut command_line_args);
                option = None;
                break;
            }

            let next_arg = next_arg.unwrap();

            if let Ok(next_option) = next_arg.parse::<CommandLineOption>() {
                // Next argument is an option, so we're done with this option
                option
                    .unwrap()
                    .handle_args(&option_args, &mut command_line_args);
                option = Some(next_option);
                break;
            } else {
                // Next argument is not an option, so it's an argument for the current option
                option_args.push(next_arg);
            }
        }
    }

    command_line_args
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn test_input_values_from_args() {
        let args = vec!["10", "20", "30", "A", "E", "F"];
        let args = args.iter().map(|s| s.to_string()).collect();

        let option = CommandLineOption::InputValues;
        let mut command_line_args = CommandLineArgs::default("".to_string());

        option.handle_args(&args, &mut command_line_args);

        assert_eq!(
            command_line_args.input_values,
            vec![
                ValueBox::Number(10),
                ValueBox::Number(20),
                ValueBox::Number(30),
                ValueBox::Character('A'),
                ValueBox::Character('E'),
                ValueBox::Character('F'),
            ]
        );
    }

    #[test]
    fn test_memory_from_args() {
        let args = vec!["0", "10", "1", "A", "2", "30", "10", "-5"];
        let args = args.iter().map(|s| s.to_string()).collect();

        let option = CommandLineOption::Memory;
        let mut command_line_args = CommandLineArgs::default("".to_string());

        option.handle_args(&args, &mut command_line_args);

        assert_eq!(
            command_line_args.memory,
            vec![
                (0, ValueBox::Number(10)),
                (1, ValueBox::Character('A')),
                (2, ValueBox::Number(30)),
                (10, ValueBox::Number(-5)),
            ]
            .into_iter()
            .collect::<HashMap<usize, ValueBox>>()
        );
    }

    #[test]
    fn test_max_memory_address_from_args() {
        let args = vec!["24"];
        let args = args.iter().map(|s| s.to_string()).collect();

        let option = CommandLineOption::MaxMemoryAddress;
        let mut command_line_args = CommandLineArgs::default("".to_string());

        option.handle_args(&args, &mut command_line_args);

        assert_eq!(command_line_args.max_memory_address, 24);
    }
}
