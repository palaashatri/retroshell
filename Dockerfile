FROM ubuntu:24.04 AS builder

# Install system dependencies
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    build-essential \
    pkg-config \
    libwayland-dev \
    libwayland-egl-backend-dev \
    libvulkan-dev \
    libegl1-mesa-dev \
    libgles2-mesa-dev \
    libxkbcommon-dev \
    libdbus-1-dev \
    libfontconfig-dev \
    libfreetype6-dev \
    fontconfig \
    fonts-dejavu-core \
    mesa-utils \
    cmake \
    libsystemd-dev \
    curl \
    git \
    libudev-dev \
    libinput-dev \
    libxcb1-dev \
    libxcb-icccm4-dev \
    libxcb-keysyms1-dev \
    libxcb-randr0-dev \
    libxcb-util0-dev \
    libxcb-xfixes0-dev \
    libgbm-dev \
    libdrm-dev \
    libseat-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/retro-render/Cargo.toml crates/retro-render/
COPY crates/retro-kit/Cargo.toml crates/retro-kit/
COPY crates/retro-shell/Cargo.toml crates/retro-shell/
COPY crates/retro-bus/Cargo.toml crates/retro-bus/
COPY crates/retro-sdk/Cargo.toml crates/retro-sdk/
COPY apps/finder/Cargo.toml apps/finder/
COPY apps/settings/Cargo.toml apps/settings/
COPY apps/textedit/Cargo.toml apps/textedit/
COPY apps/terminal/Cargo.toml apps/terminal/
COPY apps/appstore/Cargo.toml apps/appstore/

# Create dummy build files to cache dependencies
RUN mkdir -p crates/retro-render/src crates/retro-kit/src crates/retro-shell/src \
    crates/retro-bus/src crates/retro-sdk/src \
    apps/finder/src apps/settings/src apps/textedit/src apps/terminal/src apps/appstore/src \
    crates/retro-render/shaders \
    && for d in crates/retro-render crates/retro-kit crates/retro-shell crates/retro-bus crates/retro-sdk \
               apps/finder apps/settings apps/textedit apps/terminal apps/appstore; do \
        echo "fn main() {}" > "$d/src/main.rs"; \
    done \
    && touch crates/retro-render/src/lib.rs crates/retro-kit/src/lib.rs \
             crates/retro-bus/src/lib.rs crates/retro-sdk/src/lib.rs \
    && cargo build --release 2>/dev/null || true

# Copy actual source code
COPY . .

# Build everything
RUN cargo build --release --workspace

FROM ubuntu:24.04 AS runtime

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    libwayland-client0 \
    libwayland-egl1 \
    libvulkan1 \
    libegl1 \
    libxkbcommon0 \
    libdbus-1-3 \
    libfontconfig1 \
    libfreetype6 \
    fontconfig \
    fonts-dejavu-core \
    mesa-vulkan-drivers \
    ca-certificates \
    xvfb \
    x11vnc \
    dbus \
    pulseaudio \
    pulseaudio-utils \
    x11-utils \
    labwc \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/retro-shell /usr/local/bin/
COPY --from=builder /app/target/release/retro-compositor /usr/local/bin/
COPY --from=builder /app/target/release/finder /usr/local/bin/
COPY --from=builder /app/target/release/settings /usr/local/bin/
COPY --from=builder /app/target/release/textedit /usr/local/bin/
COPY --from=builder /app/target/release/terminal /usr/local/bin/
COPY --from=builder /app/target/release/appstore /usr/local/bin/

COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["retro-shell"]
