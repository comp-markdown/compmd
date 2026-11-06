# A machine like CI's runner (docs/ci.md, decision 12): Ubuntu 24.04 on x86-64, with the runner image's Chrome and
# fonts, and the repository's tools at the versions CI sets up (README, commands). `cargo make linux` builds and runs it,
# for linux/amd64.
FROM ubuntu:24.04

# The versions CI's setup names (.github/actions/setup), and the Node release the runner had.
ARG RUST=1.99.0
ARG NODE=24.22.1
ARG PNPM=12.14.0
ARG CARGO_MAKE=0.37.24
ARG CARGO_DENY=0.20.2
ARG CARGO_LLVM_COV=0.8.4
ARG WASM_BINDGEN=0.2.129

ENV DEBIAN_FRONTEND=noninteractive

# The build tools; then Chrome and the packages of the runner image that bring fonts (fonts-noto-color-emoji, xvfb), with
# their recommended packages, as the runner installs them: fontconfig's DejaVu is what a page's sans-serif falls back to.
RUN apt-get update \
 && apt-get install -y --no-install-recommends build-essential ca-certificates curl git gnupg pkg-config unzip xz-utils \
 && curl -fsSL https://dl.google.com/linux/linux_signing_key.pub | gpg --dearmor -o /usr/share/keyrings/google-chrome.gpg \
 && echo "deb [arch=amd64 signed-by=/usr/share/keyrings/google-chrome.gpg] https://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google-chrome.list \
 && apt-get update \
 && apt-get install -y google-chrome-stable fonts-noto-color-emoji xvfb \
 && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL "https://nodejs.org/dist/v${NODE}/node-v${NODE}-linux-x64.tar.xz" | tar -xJ -C /usr/local --strip-components=1 \
 && npm install --global "pnpm@${PNPM}"

ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo PATH=/usr/local/cargo/bin:$PATH
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain "${RUST}" \
      --component rustfmt,clippy,llvm-tools-preview --target wasm32-unknown-unknown

# The cargo tools as their releases' prebuilt binaries, as CI installs them.
RUN cd /usr/local/cargo/bin \
 && curl -fsSL -o cargo-make.zip "https://github.com/sagiegurari/cargo-make/releases/download/${CARGO_MAKE}/cargo-make-v${CARGO_MAKE}-x86_64-unknown-linux-musl.zip" \
 && unzip -jq cargo-make.zip '*/cargo-make' && rm cargo-make.zip \
 && curl -fsSL "https://github.com/EmbarkStudios/cargo-deny/releases/download/${CARGO_DENY}/cargo-deny-${CARGO_DENY}-x86_64-unknown-linux-musl.tar.gz" | tar -xz --strip-components=1 --wildcards '*/cargo-deny' \
 && curl -fsSL "https://github.com/taiki-e/cargo-llvm-cov/releases/download/v${CARGO_LLVM_COV}/cargo-llvm-cov-x86_64-unknown-linux-musl.tar.gz" | tar -xz \
 && curl -fsSL "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${WASM_BINDGEN}/wasm-bindgen-${WASM_BINDGEN}-x86_64-unknown-linux-musl.tar.gz" | tar -xz --strip-components=1 --wildcards '*/wasm-bindgen'

# Playwright's browsers in the volume `cargo make linux` mounts at /cache, beside pnpm's store (run.sh), as it mounts
# one at /work/target, so a later run reuses them.
ENV PLAYWRIGHT_BROWSERS_PATH=/cache/playwright

COPY --chmod=755 run.sh /usr/local/bin/compmd-linux
ENTRYPOINT ["compmd-linux"]
