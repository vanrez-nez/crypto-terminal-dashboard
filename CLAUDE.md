- This project is meant to run inside a raspberry zero 2 w with no desktop environment using DRM/GBM/EGL initialization
- Test on the Pi via ssh to monode.local

## Cross-Compilation (Docker ARM64 Emulation)

Native cross-compilation fails because the sysroot lacks libgbm/libdrm. Use Docker with QEMU emulation instead:

```bash
docker run --rm --platform linux/arm64 \
  -v "$(pwd)":/app -w /app \
  debian:bookworm bash -c '
    rm -rf /app/.cargo /app/dashboard-system/.cargo &&
    apt-get update &&
    apt-get install -y curl build-essential pkg-config \
      libgbm-dev libdrm-dev libegl-dev libgles2-mesa-dev &&
    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y &&
    . ~/.cargo/env &&
    cargo build --release
  '
```

Binary output: `target/release/crypto-dashboard` (ARM64 ELF)

## Deploy to Pi

```bash
scp target/release/crypto-dashboard config.json vanrez@monode.local:~/
ssh vanrez@monode.local "./crypto-dashboard"
```