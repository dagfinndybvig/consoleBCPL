# Windows Cross-Compile Plan (Rust / BCPL INTCODE)

## Goal
Build the Rust `icint` binary for Windows on this Ubuntu 24.04.3 LTS Codespace, then copy it to a local Windows PC.

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
cd /workspaces/consoleBCPL/bcpl-rust-console
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
Use the file explorer in VS Code to download:
```
target/x86_64-pc-windows-gnu/release/icint.exe
```

### Option B — copy via `scp` (if Windows has OpenSSH)
```bash
scp target/x86_64-pc-windows-gnu/release/icint.exe user@WINDOWS_HOST:C:\path\to\destination\
```

---

## Notes
- The GNU target is the simplest for a command-line app.
- If a build fails due to missing linker tools, re-check `mingw-w64` installation.