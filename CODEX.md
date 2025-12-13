- This project targets a Raspberry Pi Zero 2 W with no desktop environment, using DRM/GBM/EGL for initialization.
- Always test the final ARM64 binary on the Pi over SSH to `monode.local`.

## Cross-Compilation (Docker ARM64 Emulation)

Native cross-compilation currently fails because the sysroot lacks `libgbm` and `libdrm`. Build via Docker with QEMU emulation instead.

### One-time setup: build the Docker image

```bash
docker build --platform linux/arm64 -t crypto-build -f - . <<'EOF'
FROM debian:bookworm
RUN apt-get update && apt-get install -y \
    curl build-essential pkg-config \
    libgbm-dev libdrm-dev libegl-dev libgles2-mesa-dev \
  && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /app
EOF
```

### Build (fast, uses cached image)

```bash
docker run --rm --platform linux/arm64 \
  -v "$(pwd)":/app -w /app \
  crypto-build cargo build --release
```

Binary output: `target/release/crypto-dashboard` (ARM64 ELF)

## Deploy to the Pi

```bash
scp target/release/crypto-dashboard config.json vanrez@monode.local:~/
ssh vanrez@monode.local "./crypto-dashboard"
```
