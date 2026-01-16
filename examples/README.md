# Something to try

In the following I execute from the JS directory, but it does not matter much which version you are using. The Rust version needs to be built first though. The other two should be good to go. 

Also the sh-scripts are for Unix-style systems, so I added a compile.bat script specifically for Windows/Python since that seems a prevalent combination.

For all platforms: Look at the manual steps in the top-level README.md to learn more. Everything is basically the same independently of implementation, it is just a question of executing from the the right directory, and choosing the right compilation script if you are not doing it manually. 

Notice the file handling: Options -i -o in addition to normal piping.

If no options or redirecting I/O will be console.

## Transforming new-style BCPL to old-style syntax

The `transform.b` program converts text to uppercase and rewrites brace syntax:

- `{` becomes `$(`
- `}` becomes `$)`

This is useful for converting newer BCPL examples that use braces into the classic `$(` / `$)` block syntax.

NOTE: Some new features does not work in the old BCPL. For instance, I have commented out SECTION headings.

### Workflow (from the JS console)

Navigate to the `bcpl-js-console`. Use the Node-based interpreter from there:

./compile.sh ../examples/transform.b -i../examples/coins.b -o../examples/coins2.b > error.txt

- Input: `examples/coins.b` (new-style syntax)
- Output: `examples/coins2.b` (old-style syntax)
- Logs: `bcpl-js-console/error.txt`

Then compile and run the transformed program:

./compile.sh ../examples/coins2.b -o../examples/output.txt > error.txt

The program output will be written to `examples/output.txt`.

Of course you can run the command without options and redirections, then the output will just appear in the console.

### Windows (Python console with compile.bat)

From `bcpl-python-console` on Windows, use `compile.bat` instead of `compile.sh`.
It accepts the same arguments as the Linux/macOS script.

Transform `coins.b` into `coins2.b` (old-style syntax):

compile.bat ..\examples\transform.b -i..\examples\coins.b -o..\examples\coins2.b > error.txt

Then compile and run the transformed program:

compile.bat ..\examples\coins2.b -o..\examples\output.txt > error.txt

Check `error.txt` for compiler output and `examples\output.txt` for the program output.

# Coins problem background

The **coins** example is the classic *coin change counting* problem: given a set of denominations, it counts how many distinct ways there are to make a target sum where order does **not** matter. 
This is a standard combinatorics/dynamic‑programming benchmark that’s small, deterministic, and easy to verify, so it makes a good sanity test for the compiler, runtime, and I/O pipeline.

### Expected output (coins2.b)

After compiling and running `coins2.b`, the output should look something like this:

COINS PROBLEM
SUM =   0  NUMBER OF WAYS =      1<br>
SUM =   1  NUMBER OF WAYS =      1<br>
SUM =   2  NUMBER OF WAYS =      2<br>
SUM =   5  NUMBER OF WAYS =      4<br>
SUM =  21  NUMBER OF WAYS =     44<br>
SUM = 100  NUMBER OF WAYS =   4563<br>
SUM = 200  NUMBER OF WAYS =   8146<br>

---
