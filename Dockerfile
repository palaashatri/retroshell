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

# Full source copy + release build.
# (Avoids the old "dummy lib.rs dep-cache" pattern, which left empty rlibs that
# cargo could reuse and break downstream crates with missing-module errors.)
COPY . .
RUN cargo build --release --workspace

FROM ubuntu:24.04 AS runtime

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    libwayland-client0 \
    libwayland-egl1 \
    libwayland-server0 \
    libvulkan1 \
    libegl1 \
    libgles2 \
    libxkbcommon0 \
    libxkbcommon-x11-0 \
    libdbus-1-3 \
    libfontconfig1 \
    libfreetype6 \
    fontconfig \
    fonts-dejavu-core \
    mesa-vulkan-drivers \
    mesa-utils \
    libgl1-mesa-dri \
    libgbm1 \
    libdrm2 \
    libinput10 \
    libudev1 \
    ca-certificates \
    xvfb \
    x11vnc \
    x11-utils \
    x11-xserver-utils \
    xdotool \
    dbus \
    dbus-x11 \
    pulseaudio \
    pulseaudio-utils \
    labwc \
    novnc \
    websockify \
    imagemagick \
    ffmpeg \
    xwayland \
    at-spi2-core \
    at-spi2-common \
    libatspi2.0-0 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/retro-shell /usr/local/bin/
COPY --from=builder /app/target/release/retro-compositor /usr/local/bin/
COPY --from=builder /app/target/release/finder /usr/local/bin/
COPY --from=builder /app/target/release/settings /usr/local/bin/
COPY --from=builder /app/target/release/textedit /usr/local/bin/
COPY --from=builder /app/target/release/terminal /usr/local/bin/
COPY --from=builder /app/target/release/appstore /usr/local/bin/

COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
COPY scripts/start-retroshell /usr/local/bin/start-retroshell
COPY packaging/ /usr/share/retroshell/packaging/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh /usr/local/bin/start-retroshell

EXPOSE 6080

ENV WGPU_BACKEND=vulkan
ENV WGPU_POWER_PREF=low-power
ENV LIBGL_ALWAYS_SOFTWARE=1
ENV MESA_LOADER_DRIVER_OVERRIDE=softpipe
# Default lock password is set in docker-entrypoint.sh when seeding settings.conf
# (not via ENV) so secrets are not baked into the image config layer.

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["retro-shell"]
