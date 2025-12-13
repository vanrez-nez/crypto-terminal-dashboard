# crypto-dashboard

## Setup

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the prompts, then restart your terminal or run:

```bash
source "$HOME/.cargo/env"
```

### Build & Run

```bash
cargo run
```

### Cross-Compile for Raspberry Pi (Linux ARM64)

1. Add the target:

```bash
rustup target add aarch64-unknown-linux-gnu
```

2. Install the cross-compiler toolchain:

**macOS (Homebrew):**
```bash
brew install messense/macos-cross-toolchains/aarch64-unknown-linux-gnu
```

**Ubuntu/Debian:**
```bash
sudo apt install gcc-aarch64-linux-gnu
```

3. Configure Cargo linker in `~/.cargo/config.toml`:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-unknown-linux-gnu-gcc"
```

4. Build the release binary:

```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

5. The binary will be at `target/aarch64-unknown-linux-gnu/release/crypto-dashboard`

6. Copy to Raspberry Pi along with `config.json`:

```bash
scp target/aarch64-unknown-linux-gnu/release/crypto-dashboard pi@<raspberry-pi-ip>:~/
scp config.json pi@<raspberry-pi-ip>:~/
```

- After finishing a task you must compile and run the binary in the Raspberry Pi.