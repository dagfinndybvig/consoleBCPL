Windows cross-build for `icint.c`

Usage:

- From `bcpl-c-console` run the script:

  ```bash
  ./Windows/compile.sh
  ```

- Or use `make` from the `Windows` folder:

  ```bash
  make -C Windows
  ```

Notes:
- The default cross-compiler is `x86_64-w64-mingw32-gcc`. To build 32-bit use `CC=i686-w64-mingw32-gcc ./Windows/compile.sh`.
- If the cross-compiler is not installed on your system, install `mingw-w64` (package names vary by distro).
- The script expects `../icint.c` to exist (it does in this repo).
