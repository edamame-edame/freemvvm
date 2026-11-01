FROM rust:1.90.0-slim-bookworm

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends \
        build-essential \
        ca-certificates \
        fontconfig=2.14.1-4 \
        fontconfig-config=2.14.1-4 \
        fonts-noto-cjk=1:20220127+repack1-1 \
        libdrm-dev \
        libegl1 \
        libfreetype-dev=2.12.1+dfsg-5+deb12u4 \
        libfontconfig1=2.14.1-4 \
        libgbm-dev \
        libharfbuzz-dev=6.0.0+dfsg-3 \
        libgles2 \
        libgl1-mesa-dri \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy \
    && rustup toolchain install 1.85.0 --profile minimal

ENV EGL_PLATFORM=surfaceless \
    FC_LANG=ja-jp \
    LIBGL_ALWAYS_SOFTWARE=true

WORKDIR /workspace
