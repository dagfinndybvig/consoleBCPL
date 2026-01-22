# Windows Cross-Compile Plan (Rust / BCPL INTCODE with Coroutines)

## Goal
Build the coroutine-enabled Rust `icint` binary for Windows on Ubuntu, then use it to compile and run BCPL programs in this folder.

---

## 1) Install cross-compilation tools (GNU target)
```bash
sudo apt update
sudo apt install -y mingw-w64
rustup target add x86_64-pc-windows-gnu
```

---

## 2) Build the Windows binary
```bash
cd /workspaces/consoleBCPL/bcpl-rust-console/bcpl-with-coroutines
cargo build --release --target x86_64-pc-windows-gnu
```

---

## 3) Verify the output binary
```bash
ls -lh target/x86_64-pc-windows-gnu/release/icint.exe
```

---

## 4) Copy to Windows PC
Use one of these options:

### Option A — download via VS Code
Use the file explorer to download:
```
target/x86_64-pc-windows-gnu/release/icint.exe
```

### Option B — copy via scp (if Windows has OpenSSH)
```bash
scp target/x86_64-pc-windows-gnu/release/icint.exe user@WINDOWS_HOST:C:\path\to\destination\
```

---

## 5) Build and run on Windows
Open a Command Prompt in the bcpl-with-coroutines folder on Windows:

```bat
compile.bat test_coroutines_min.b -iinput.txt -ooutput.txt
```

Check output.txt and error.txt after the run.

---

## Notes
- The GNU target is the simplest for a command-line app.
- If the build fails due to missing linker tools, re-check mingw-w64 installation.
