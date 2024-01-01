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

> TO COMPLETE

Once you have your `.exe`, you can run it with the following command (assuming you're in the same directory. And on Windows.):

```bash
./hrm-interpreter.exe <script> <input>
```

Where `<script>` is the path to the script file, and `<input>` is the input to the script, as list of elements separated by spaces.

Example:

```bash
./hrm-interpreter.exe script.txt 1 2 a 4 f
```

## Scripts

Scripts are text files provided by the game. In any level, click the "Copy" button to copy your script as text file.

## FAQ

### Why?

Why not? It seemed like a fun project to do in Rust.

### How can I use the interpreter to help me with the game?

In the game you can click the "Copy" button to copy your script as a text file. You can then run it with the interpreter to see what it does.
In the interpreter, you can, contrary to the game:
- Control the input
- Run the script really fast (the game has a speed limit)
But you can't, contrary to the game:
- Run the script step by step
- Visualize the memory

You can also modify the script, test it, and paste it back in the game.

### So I can program my scripts outside of the game?

I guess! You'll still need to check the game levels to know your memory constraints, but you can write your scripts in your favorite editor. There are also some vscode plugins that can help you with that
(like [this](https://marketplace.visualstudio.com/items?itemName=grub4k.hrm-language) or [that](https://marketplace.visualstudio.com/items?itemName=jasonwthompson.human-resource-machine-language-support), though I haven't tried them).