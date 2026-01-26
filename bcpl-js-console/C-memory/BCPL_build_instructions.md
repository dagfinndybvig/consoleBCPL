Build instructions for icint.c

From the `bcpl-js-console/C-memory` directory, compile the interpreter with GCC on Linux using one of these exact commands:

- Release build (optimized):

```sh
gcc -O2 -std=c99 -Wall -Wextra -o C_icint icint.c
```

- Debug build (with symbols, no optimization):

```sh
gcc -g -O0 -std=c99 -Wall -Wextra -o C_icint icint.c
```

Notes:
- `icint.c` includes `icint.h` in the same directory; the commands above assume you're in `bcpl-js-console/C-memory`.
- The repository includes `C_compile.sh` which expects the interpreter binary to be named `C_icint` in the same directory; after building, run `./C_compile.sh <program.b>` to compile and execute BCPL programs using the interpreter.
