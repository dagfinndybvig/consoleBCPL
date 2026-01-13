# BCPL INTCODE Interpreter - Rust Port

This is a pure Rust port of the BCPL INTCODE interpreter, based on the Node.js and Python implementations.

## Features

- Pure Rust implementation with zero dependencies
- Full BCPL intcode instruction set support
- File I/O operations
- String operations (packed/unpacked BCPL strings)
- Assembler and interpreter in one binary

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
```

This will:
1. Concatenate the BCPL compiler (syni + trni)
2. Compile your BCPL source to OCODE
3. Compile OCODE to INTCODE
4. Run the INTCODE

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
