# A machine like CI's runner (docs/ci.md, decision 12): Ubuntu 24.04 on x86-64, with the runner image's Chrome and
# fonts, and the repository's tools at the versions CI sets up. `cargo make linux` builds it from the repository's root,
# of which it copies only `tools/versions.yml` and `package.json`, and runs it, for linux/amd64.
FROM ubuntu:24.04

# The Rust of rust-toolchain.toml, and the release of `package.json`'s Node 24 the runner had. The cargo tools are
# tools/versions.yml's, and pnpm is `package.json`'s `devEngines`', as CI's setup reads them.
ARG RUST=1.99.0
ARG NODE=24.22.1

ENV DEBIAN_FRONTEND=noninteractive

# The build tools, Chrome, and the runner image's fonts, the packages `dpkg-query -W 'fonts-*'` lists on it (2026-11-06):
# its sans-serif, serif, monospace and system-ui are DejaVu, which a page's fonts fall back to.
RUN apt-get update \
 && apt-get install -y --no-install-recommends build-essential ca-certificates curl git gnupg pkg-config unzip xz-utils \
      fonts-dejavu fonts-dejavu-core fonts-dejavu-extra fonts-dejavu-mono fonts-lato fonts-liberation fonts-noto-color-emoji fonts-ubuntu-console \
 && curl -fsSL https://dl.google.com/linux/linux_signing_key.pub | gpg --dearmor -o /usr/share/keyrings/google-chrome.gpg \
 && echo "deb [arch=amd64 signed-by=/usr/share/keyrings/google-chrome.gpg] https://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google-chrome.list \
 && apt-get update \
 && apt-get install -y --no-install-recommends google-chrome-stable \
 && rm -rf /var/lib/apt/lists/*

# After the system's packages, so a version change doesn't install them again.
COPY tools/versions.yml package.json /versions/
RUN curl -fsSL "https://nodejs.org/dist/v${NODE}/node-v${NODE}-linux-x64.tar.xz" | tar -xJ -C /usr/local --strip-components=1 \
 && npm install --global "pnpm@$(node -p 'require("/versions/package.json").devEngines.packageManager.version')"

ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo PATH=/usr/local/cargo/bin:$PATH
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain "${RUST}" \
      --component rustfmt,clippy,llvm-tools-preview --target wasm32-unknown-unknown

# The cargo tools as their releases' prebuilt binaries, as CI installs them, at tools/versions.yml's versions.
RUN version() { sed -nE "s/^$1: \"([^\"]+)\"$/\1/p" /versions/versions.yml; } \
 && CARGO_MAKE="$(version cargo-make)" CARGO_DENY="$(version cargo-deny)" \
 && CARGO_LLVM_COV="$(version cargo-llvm-cov)" WASM_BINDGEN="$(version wasm-bindgen)" \
 && cd /usr/local/cargo/bin \
 && curl -fsSL -o cargo-make.zip "https://github.com/sagiegurari/cargo-make/releases/download/${CARGO_MAKE}/cargo-make-v${CARGO_MAKE}-x86_64-unknown-linux-musl.zip" \
 && unzip -jq cargo-make.zip '*/cargo-make' && rm cargo-make.zip \
 && curl -fsSL "https://github.com/EmbarkStudios/cargo-deny/releases/download/${CARGO_DENY}/cargo-deny-${CARGO_DENY}-x86_64-unknown-linux-musl.tar.gz" | tar -xz --strip-components=1 --wildcards '*/cargo-deny' \
 && curl -fsSL "https://github.com/taiki-e/cargo-llvm-cov/releases/download/v${CARGO_LLVM_COV}/cargo-llvm-cov-x86_64-unknown-linux-musl.tar.gz" | tar -xz \
 && curl -fsSL "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${WASM_BINDGEN}/wasm-bindgen-${WASM_BINDGEN}-x86_64-unknown-linux-musl.tar.gz" | tar -xz --strip-components=1 --wildcards '*/wasm-bindgen'

# Playwright's browsers in the volume `cargo make linux` mounts at /cache, beside pnpm's store (run.sh), as it mounts
# one at /work/target, so a later run reuses them.
ENV PLAYWRIGHT_BROWSERS_PATH=/cache/playwright

COPY --chmod=755 tools/linux-ci/run.sh /usr/local/bin/compmd-linux
ENTRYPOINT ["compmd-linux"]
