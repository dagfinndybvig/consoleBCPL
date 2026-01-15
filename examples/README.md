# Something to try

If you have built the Rust version for your system you can follow the first example. Or follow the JS version below, it is essentially the same.
Or Python for that matter. You must just run the compile script from the relevant directory.

The is Linux-style, but Windows should be much the same. Look at compile.bat in the various interpreter directories.

Notice the file handling! Options -i -o in addition to normal piping.

If no options or redirecting I/O will be console.

## Transforming new-style BCPL to old-style syntax

The `transform.b` program converts text to uppercase and rewrites brace syntax:

- `{` becomes `$(`
- `}` becomes `$)`

This is useful for converting newer BCPL examples that use braces into the classic `$(` / `$)` block syntax.

### Workflow (from the Rust console)

Run the transformer using the Rust INTCODE interpreter in `bcpl-rust-console`:

./compile.sh ../examples/transform.b -i../examples/coins.b -o../examples/coins2.b > error.txt

- Input: `examples/coins.b` (new-style syntax)
- Output: `examples/coins2.b` (old-style syntax)
- Logs: `bcpl-rust-console/error.txt`

You can then compile and run the transformed program:

./compile.sh ../examples/coins2.b -o../examples/output.txt > error.txt

The program output will be written to `examples/output.txt`.

### Expected output (coins2.b)

After compiling and running `coins2.b`, the output should look like this:

COINS PROBLEM
SUM =   0  NUMBER OF WAYS =      1
SUM =   1  NUMBER OF WAYS =      1
SUM =   2  NUMBER OF WAYS =      2
SUM =   5  NUMBER OF WAYS =      4
SUM =  21  NUMBER OF WAYS =     44
SUM = 100  NUMBER OF WAYS =   4563
SUM = 200  NUMBER OF WAYS =   8146

### Workflow (from the JS console)

The same steps work from `bcpl-js-console`. Use the Node-based interpreter there:

./compile.sh ../examples/transform.b -i../examples/coins.b -o../examples/coins2.b > error.txt

- Input: `examples/coins.b` (new-style syntax)
- Output: `examples/coins2.b` (old-style syntax)
- Logs: `bcpl-js-console/error.txt`

Then compile and run the transformed program:

./compile.sh ../examples/coins2.b -o../examples/output.txt > error.txt

The program output will be written to `examples/output.txt`.