# BCPL INTCODE Interpreter - Rust Port

This is a pure Rust port of the BCPL INTCODE interpreter, based on the Node.js and Python implementations. It has been tested with all available BCPL sample programs and produces identical output to the other implementations.

## Features

- Pure Rust implementation with zero dependencies
- Full BCPL INTCODE instruction set support
- File I/O operations
- String operations (packed/unpacked BCPL strings)
- Assembler and interpreter in one binary
- Tested with test.b, fact.b, queens.b, and cmpltest.b

## Prerequisites

You need Rust installed on your system. If you don't have it, install it using rustup:

```bash
# On Linux/macOS:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# On Windows:
# Download and run rustup-init.exe from https://rustup.rs
```

After installation, restart your terminal or run:
```bash
source $HOME/.cargo/env
```

Verify the installation:
```bash
rustc --version
cargo --version
```

## Building

Build in release mode for best performance:

```bash
cargo build --release
```

The binary will be created at `target/release/icint`.

## Usage

### Running INTCODE files directly

```bash
./target/release/icint INTCODE_FILE
```

### With input/output redirection

```bash
./target/release/icint INTCODE_FILE -iINPUT_FILE -oOUTPUT_FILE
```

### Compiling and running BCPL programs

Use the `compile.sh` script to compile and run BCPL source files:

```bash
./compile.sh test.b
./compile.sh test.b -iINPUT -oOUTPUT
```

This will:
1. Concatenate the BCPL compiler (syni + trni)
2. Compile your BCPL source to OCODE
3. Compile OCODE to INTCODE
4. Run the INTCODE

## Windows (GNU) build and usage

You can cross-compile the Windows binary from Linux using the GNU target. Install the toolchain and target:

```bash
sudo apt update
sudo apt install -y mingw-w64
rustup target add x86_64-pc-windows-gnu
```

Build the Windows binary:

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

The Windows executable will be created at:

```
target/x86_64-pc-windows-gnu/release/icint.exe
```

On Windows, you can use the `compile.bat` script (analogous to `compile.sh`) to compile and run BCPL programs:

```bat
compile.bat test.b
compile.bat test.b -iINPUT -oOUTPUT
```

## Example Programs

- `test.b` - Simple "Hello World" program
- `fact.b` - Factorial calculator
- `queens.b` - N-Queens problem solver
- `scan.b` - Character scanner demo

## Implementation Notes

This Rust port closely follows the JavaScript and Python implementations while leveraging Rust's:
- Memory safety guarantees
- Zero-cost abstractions
- Performance optimizations

The interpreter uses:
- `Vec<i16>` for 16-bit word memory
- Little-endian byte ordering for BCPL string packing
- Buffered I/O for better performance
- Wrapping arithmetic to match BCPL semantics

## Performance

The Rust version typically performs comparably to or better than the Node.js version and significantly better than the Python version (especially compared to CPython).
