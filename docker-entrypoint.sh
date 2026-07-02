#!/usr/bin/env bash
set -e

# Clean up leftover lock files and temp dirs from previous runs
rm -f /tmp/.X99-lock /tmp/.X11-unix/X99 /run/dbus/pid || true
rm -rf /tmp/pulse-* /tmp/runtime-root/* || true

echo "=== Starting PulseAudio ==="
# Allow running pulseaudio as root
mkdir -p /var/run/dbus
dbus-daemon --system --fork || true
pulseaudio -D --verbose --exit-idle-time=-1 --log-target=stderr --disallow-exit || true

echo "=== Loading PulseAudio null sink ==="
# Load null sink to capture audio without a physical soundcard
pactl load-module module-null-sink sink_name=VirtualSink sink_properties=device.description="Virtual_Sink" || true
pactl set-default-sink VirtualSink || true

echo "=== Starting Xvfb ==="
Xvfb :99 -screen 0 1280x800x24 &
sleep 1
export DISPLAY=:99

echo "=== Starting x11vnc ==="
x11vnc -display :99 -nopw -forever -listen 0.0.0.0 -rfbport 5900 &
sleep 1

echo "=== Starting noVNC ==="
/usr/share/novnc/utils/novnc_proxy --vnc localhost:5900 --listen 6080 &
sleep 1

echo "=== Starting Wayland Compositor (labwc) ==="
export XDG_RUNTIME_DIR=/tmp/runtime-root
mkdir -p $XDG_RUNTIME_DIR
chmod 700 $XDG_RUNTIME_DIR
# Run labwc inside Xvfb (will use X11 backend)
labwc &
sleep 2

# Set WAYLAND_DISPLAY for clients after compositor started
export WAYLAND_DISPLAY=wayland-0

echo "=== Configuring labwc output mode ==="
# Install wlr-randr if not present and set output to match Xvfb resolution
if ! command -v wlr-randr &>/dev/null; then
    apt-get update -qq && apt-get install -y -qq wlr-randr 2>/dev/null || true
fi
# Give labwc a moment to initialize the output
sleep 1
wlr-randr --output WL-1 --mode 1280x800 2>/dev/null || true

echo "=== Ready ==="
# Keep container running and execute commands passed to it, or sleep
if [ $# -gt 0 ]; then
    exec "$@"
else
    exec sleep infinity
fi
