#!/usr/bin/env bash
set -e

: "${RETROSHELL_VM_WIDTH:=1280}"
: "${RETROSHELL_VM_HEIGHT:=800}"
: "${RETROSHELL_VM_DEPTH:=24}"

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
Xvfb :99 -screen 0 "${RETROSHELL_VM_WIDTH}x${RETROSHELL_VM_HEIGHT}x${RETROSHELL_VM_DEPTH}" &
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
export XDG_CONFIG_HOME=/root/.config
mkdir -p "$XDG_RUNTIME_DIR" "$XDG_CONFIG_HOME/labwc"
chmod 700 "$XDG_RUNTIME_DIR"

cat > "$XDG_CONFIG_HOME/labwc/rc.xml" <<'EOF'
<?xml version="1.0" encoding="utf-8"?>
<labwc_config>
  <core>
    <decoration>server</decoration>
    <gap>0</gap>
  </core>
  <theme>
    <maximizedDecoration>none</maximizedDecoration>
    <dropShadows>no</dropShadows>
  </theme>
  <resize>
    <popupShow>Never</popupShow>
  </resize>
</labwc_config>
EOF

# Run labwc inside Xvfb (will use X11 backend)
WLR_BACKENDS=x11 WLR_RENDERER_ALLOW_SOFTWARE=1 labwc &
sleep 2

# Set WAYLAND_DISPLAY for clients after compositor started
export WAYLAND_DISPLAY=wayland-0

echo "=== Configuring labwc output mode ==="
# Set the nested wlroots output to match Xvfb resolution.
if ! command -v wlr-randr &>/dev/null; then
    echo "wlr-randr not installed; skipping output mode configuration"
else
    if command -v xdotool &>/dev/null; then
        x11_window="$(
            DISPLAY=:99 xwininfo -root -children 2>/dev/null \
                | awk '/^[[:space:]]+0x[0-9a-f]+/ { print $1; exit }'
        )"
        if [ -n "$x11_window" ]; then
            DISPLAY=:99 xdotool windowsize \
                "$x11_window" \
                "$RETROSHELL_VM_WIDTH" \
                "$RETROSHELL_VM_HEIGHT" \
                2>/dev/null || true
            sleep 0.5
        fi
    fi

    for _ in $(seq 1 20); do
        output="$(wlr-randr 2>/dev/null | awk '/^[^[:space:]]/ { print $1; exit }')"
        if [ -n "$output" ]; then
            wlr-randr \
                --output "$output" \
                --mode "${RETROSHELL_VM_WIDTH}x${RETROSHELL_VM_HEIGHT}" \
                --pos 0,0 \
                --scale 1 \
                2>/dev/null || true
            break
        fi
        sleep 0.25
    done
fi

echo "=== Ready ==="
# Keep container running and execute commands passed to it, or sleep
if [ $# -gt 0 ]; then
    exec "$@"
else
    exec sleep infinity
fi
