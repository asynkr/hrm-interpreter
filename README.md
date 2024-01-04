# Human Resource Machine Interpreter

This is a simple interpreter for the script language used in the game [Human Resource Machine](http://tomorrowcorporation.com/humanresourcemachine) by [Tomorrow Corporation](http://tomorrowcorporation.com/).

## Usage

### Source

To build the interpreter, you need to have [Rust](https://www.rust-lang.org/) and [Cargo](https://crates.io/) installed. Then that's as simple as running:

```bash
cargo build --release
```

You can then use the built `.exe`.

You can also take the `.exe` directly from the [releases](https://github.com/asynkr/hrm-interpreter/releases).

### Binary

Once you have your `.exe`, you can run it with the following command (assuming you're in the same directory. And on Windows.):

```bash
./hrm-interpreter.exe <script> [options]
```

Where `<script>` is the path to the script file. The options are (also described in the `--help` command):
```
Human Resource Machine interpreter
Get this help: hrm-interpreter.exe -h | --help
Usage:         hrm-interpreter.exe <script_file> [options]
Options:
  -i, --inputs <value> <value>...                         sets the values to be used as input
                                                            Example: -i 10 20 30 A E F
                                                            Default: no input values
  -m, --memory <address> <value>... | <memory_file>       sets the values to be used as memory
                                                            Example: -m 0 10 1 A 2 30 | -m memory.txt
                                                            Default: no starting memory values
  -M, --max-mem <max_address>                             sets the maximum memory address. That's the last tile number in the game.
                                                            Example: -M 24
                                                            Default: no (theoretical) maximum
```

_NB_: I chose to use the maximum **address** and not the length of the memory - the latter being more common in the programming world. That's because I think it's easier to look at the last tile number in-game than to always remember to add one when switching to this interpreter.

### Sample scripts

In the `samples` folder, you'll find sample scripts corresponding to some levels from the game. You can run them with the interpreter to see what they do.
Here are the commands to run them (here with `cargo run`, but you can also directly use the `.exe`) with the correct options:

* `cargo run --release -- ./samples/01-MailRoom.hrm -i 6 5 6`
* `cargo run --release -- ./samples/06-RainySummer.hrm -i 0 4 7 3 -5 1 -M 2`
* `cargo run --release -- ./samples/20-MultiplicationWorkshop.hrm -i 4 3 4 1 9 0 0 1 7 8 -m 9 0 -M 9`
* `cargo run --release -- ./samples/30-StringStorageFloor.hrm -i 4 15 7 0 20 17 11 21 2 13 4 17 20 -m ./samples/mem/30-mem.txt -M 24`
* `cargo run --release -- ./samples/41-SortingRoom.hrm -i 71 26 65 0 A L I V E 0 35 74 69 90 67 72 65 74 84 14 0 86 0 -m 24 0 -M 24`

In these commands I provided the inputs and memory constraints as described in the game. Scripts are mine :)

Other scripts for other levels can be found on the [wiki](https://strategywiki.org/wiki/Human_Resource_Machine). For inputs and memory constraints, you'll have to look in-game.

## Scripts

Scripts are text files provided by the game. In any level, click the "Copy" button to copy your script as text file. However, slightly different formats can be supported, so here are the different assumptions made by the parser:
- A line starting with "--" is ignored
- A line with the command "COMMENT" is ignored
- The parsing stops at the first "DEFINE" command. In the game, the following lines are used to define labels (which are drawing) for comments and memory tiles. The interpreter doesn't need them, so they are ignored, and as far as I know, they are always at the end of the script.
- Multiple spaces are the same as one space
- Indents are ignored
- ":" character is used for and only for jump destinations
- ALL COMMANDS are allowed. In-game, you are limited in early levels, with commands unlocking as you progress. The interpreter doesn't care about that, so you can use any command in any level. It's up to you to use only commands you have access to for that level.

## FAQ

### Why?

Why not? It seemed like a fun project to do in Rust.

### How can I use the interpreter to help me with the game?

In the game you can click the "Copy" button to copy your script as a text file. You can then run it with the interpreter to see what it does.

In the interpreter, you **can**, contrary to the game:
- Control the input
- Run the script really fast (the game has a speed limit)

But you **cannot**, contrary to the game:
- Run the script step by step
- Visualize the memory

You can also modify the script, test it, and paste it back in the game.

### So I can program my scripts outside of the game?

I guess! You'll still need to check the game levels to know your inputs and memory constraints, but you can write your scripts in your favorite editor. There are also some vscode plugins that can help you with that
(like [this](https://marketplace.visualstudio.com/items?itemName=grub4k.hrm-language) or [that](https://marketplace.visualstudio.com/items?itemName=jasonwthompson.human-resource-machine-language-support), though I haven't tried them).