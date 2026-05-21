# PyRs

A Python interpreter written in Rust.
- Currently only runs on linux

PyRs parses and executes Python source files by compiling them to an internal bytecode and running them on a custom stack-based virtual machine. It also provides an interactive REPL for quick experimentation.

## Features

- **File interpretation** – run `.py` files directly
- **Bytecode compilation** – compile `.py` files to `.pyc` bytecode cached under `__pycache__/`
- **Interactive REPL** – a live `>>>` prompt for evaluating expressions
- **Control flow** – `if` / `elif` / `else`, `while`, `for` loops
- **Functions** – `def`, `return`, `pass`
- **Classes** – basic class definitions and attribute access
- **Modules** – `import` statements that load other `.py` files
- **Built-in functions** – `print`, `input`, `range`, `exit`
- **Data types** – integers (arbitrary precision via [rug](https://crates.io/crates/rug)), floats, strings, booleans, lists, tuples, sets, `None`
- **Debug mode** – prints the parsed expressions, bytecode, and stack traces

## Requirements

| Tool | Notes |
|------|-------|
| Rust / Cargo | Install via [rustup](https://rustup.rs/) |
| GCC | Required by the `rug` crate (`build-essential` on Debian/Ubuntu) |
| GMP / MPFR | `libgmp-dev libmpfr-dev` on Debian/Ubuntu |
| m4 | `m4` package on Debian/Ubuntu |

A helper script for Linux is provided:

```sh
bash scripts/Install-Linux.sh
```

## Building

```sh
cargo build --release
```

The compiled binary is placed at `target/release/Pyrs`.

## Usage

```
PyRs [flags] [file]
```

| Flag | Long form | Description |
|------|-----------|-------------|
| _(none)_ | | Start the interactive REPL |
| `-h` | `--help` | Print help |
| `-a` | `--all` | Allow any file extension (default: `.py` only) |
| `-c` | `--compile` | Compile the file to bytecode instead of running it |
| `-d` | `--debug` | Enable debug output (parsed expressions, bytecode, stack) |

### Examples

**Run a Python file**

```sh
./target/release/Pyrs fib.py
```

**Compile a file to bytecode**

```sh
./target/release/Pyrs -c fib.py
# Output: __pycache__/fib.pyrs-001.pyc
```

**Run with debug output**

```sh
./target/release/Pyrs -d fib.py
```

**Run any file type**

```sh
./target/release/Pyrs -a script.txt
```

**Start the interactive REPL**

```sh
./target/release/Pyrs
>>> x = 1 + 2
>>> print(x)
3
>>> exit()
```

**Via `cargo run`**

```sh
cargo run -- fib.py
cargo run -- -d fib.py
cargo run -- -c fib.py
```

## Project Layout

```
src/
  main.rs               Entry point and CLI argument parsing
  pyrs_interpreter.rs   High-level interpreter, REPL, and file runner
  pyrs_vm.rs            Stack-based bytecode virtual machine
  pyrs_bytecode.rs      Bytecode instruction definitions
  pyrs_codeobject.rs    Compilation context and code objects
  pyrs_parsing.rs       Tokeniser, parser, and AST (Expression)
  pyrs_obj.rs           Python object types (Obj enum)
  pyrs_std.rs           Standard library function pointers
  pyrs_modules.rs       Module loading helpers
  pyrs_error.rs         Exception and error types
  pyrs_pyc.rs           .pyc file format support
  pyrs_serializer.rs    Code-object serialisation
  pyrs_utils.rs         Miscellaneous utilities
  pyrs_tests/           Unit and integration tests
tests/                  Sample Python test files
scripts/
  Install-Linux.sh      Dependency installer for Linux
```

## License

MIT – see [LICENSE](LICENSE).
