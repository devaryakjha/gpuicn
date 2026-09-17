FROM ubuntu:24.04
RUN dpkg --add-architecture arm64 \
 && sed -i '/^Components:/a Architectures: amd64' /etc/apt/sources.list.d/ubuntu.sources \
 && printf 'Types: deb\nURIs: http://ports.ubuntu.com/ubuntu-ports\nSuites: noble noble-updates noble-security\nComponents: main universe\nArchitectures: arm64\nSigned-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg\n' > /etc/apt/sources.list.d/arm64.sources \
 && apt-get update -qq \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ca-certificates build-essential gcc-aarch64-linux-gnu g++-aarch64-linux-gnu pkg-config clang cmake git python3 libssl-dev:arm64 libfontconfig1-dev:arm64 libfreetype-dev:arm64 libxkbcommon-dev:arm64 libxkbcommon-x11-dev:arm64 libwayland-dev:arm64 libxcb1-dev:arm64 libxcb-xkb-dev:arm64 libxcb-render0-dev:arm64 libxcb-shape0-dev:arm64 libxcb-xfixes0-dev:arm64 \
 && rm -rf /var/lib/apt/lists/*
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
 CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
 CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
 PKG_CONFIG_ALLOW_CROSS=1 \
 PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig \
 CARGO_HOME=/cargo RUSTUP_HOME=/rustup PATH=/cargo/bin:$PATH
WORKDIR /source
