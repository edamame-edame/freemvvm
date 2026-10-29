FROM rust:1.90.0-slim-bookworm

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends \
        build-essential \
        ca-certificates \
        libdrm-dev \
        libegl1 \
        libgbm-dev \
        libgles2 \
        libgl1-mesa-dri \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy \
    && rustup toolchain install 1.85.0 --profile minimal

ENV EGL_PLATFORM=surfaceless \
    LIBGL_ALWAYS_SOFTWARE=true

WORKDIR /workspace
