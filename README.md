# Thermocouple Logger

4-channel K-type thermocouple logger for the TA612C via serial (9600 8N1).

## Building

### macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
cargo build --release
./target/release/thermocouple_logger
```

### Windows

Install Rust from https://rustup.rs — use the default installer (MSVC toolchain).
You'll also need the Visual Studio C++ build tools if not already present; the Rust
installer will prompt you.

```cmd
cargo build --release
.\target\release\thermocouple_logger.exe
```

### Cross-compile macOS → Windows (optional)

```bash
brew install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
# produces: target/x86_64-pc-windows-gnu/release/thermocouple_logger.exe
```

## Usage

1. Plug in the TA612C via USB — it appears as a COM port (Windows) or `/dev/tty.usbserial-XXXX` (macOS)
2. Select the port from the dropdown (hit ↻ to refresh)
3. Set sample interval
4. **▶ Start** — begins capturing to in-memory buffer
5. **⏸ Pause / ▶ Resume** as needed
6. **⚑ Mark** — stamps a labelled event on the plot (edit the label field first)
7. **■ Stop** — ends capture; data stays in buffer
8. **↓ Export CSV** — writes `thermode_YYYYMMDD_HHMMSS.csv` to the working directory

## CSV format

```
elapsed_s,T1,T2,T3,T4
0.000,27.5,27.3,27.4,27.5
1.001,28.1,27.9,28.0,28.2
...

# Markers
# elapsed_s,label
# 12.340,contact
# 18.920,release
```

## Protocol notes

TA612C: USB serial, 9600 8N1, no parity.
Request:  `AA 55 01 03 03`
Response: `55 AA 01 0B [T1lo T1hi T2lo T2hi T3lo T3hi T4lo T4hi] [sum]` (13 bytes)
Values are signed int16 little-endian, divide by 10 for °C.
Open circuit / no probe → 0x7FFF → displayed as —
