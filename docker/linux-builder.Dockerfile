# Imagem pra buildar os pacotes Linux (.deb/.AppImage) com uma glibc antiga
# o bastante pra rodar em qualquer distro razoavelmente recente — ver
# "Melhoria — Build Linux compatível com Ubuntu 22.04 (glibc)" no FASES.md
# pra entender o problema que isso resolve.
#
# Uso (a partir da raiz do repo):
#   docker build -t select-translate-builder:22.04 -f docker/linux-builder.Dockerfile docker
#   docker run --rm \
#     -v "$PWD:/app:ro" \
#     -v "$PWD/dist-linux:/build-target" \
#     -w /app/src-tauri \
#     -e CARGO_TARGET_DIR=/build-target \
#     select-translate-builder:22.04 \
#     cargo tauri build --bundles deb,appimage
#
# Os pacotes saem em ./dist-linux/release/bundle/{deb,appimage}/.

FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev \
    libssl-dev libayatana-appindicator3-dev librsvg2-dev \
    && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo install tauri-cli --version "^2" --locked
