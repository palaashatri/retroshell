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
xrandr --query 2>/dev/null | awk '/current/ { print "Xvfb", $8 "x" $10 }' || true

echo "=== Starting x11vnc ==="
x11vnc -display :99 -nopw -forever -listen 0.0.0.0 -rfbport 5900 &
sleep 1

echo "=== Starting noVNC ==="
/usr/share/novnc/utils/novnc_proxy --vnc localhost:5900 --listen 6080 &
sleep 1

echo "=== Starting Wayland Compositor ==="
export XDG_RUNTIME_DIR=/tmp/runtime-root
export XDG_CONFIG_HOME=/root/.config
export RETROSHELL_COMPOSITOR_WIDTH="$RETROSHELL_VM_WIDTH"
export RETROSHELL_COMPOSITOR_HEIGHT="$RETROSHELL_VM_HEIGHT"
mkdir -p "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"

# Try retro-compositor first; fall back to labwc if unavailable or crashes
DISPLAY=:99 retro-compositor &
RETRO_COMPOSITOR_PID=$!
sleep 3

if kill -0 "$RETRO_COMPOSITOR_PID" 2>/dev/null; then
    echo "=== retro-compositor is running ==="
    # Read actual socket name that the compositor writes to a file
    RETRO_SOCKET=""
    for _ in $(seq 1 20); do
        if [ -f /tmp/runtime-root/wayland-display ]; then
            RETRO_SOCKET=$(cat /tmp/runtime-root/wayland-display)
            break
        fi
        sleep 0.25
    done
    if [ -n "$RETRO_SOCKET" ]; then
        export WAYLAND_DISPLAY="$RETRO_SOCKET"
        echo "=== WAYLAND_DISPLAY=$WAYLAND_DISPLAY ==="
    else
        export WAYLAND_DISPLAY=wayland-0
        echo "=== wayland-display file not found, using wayland-0 ==="
    fi
else
    echo "=== retro-compositor not running; falling back to labwc ==="
    mkdir -p "$XDG_CONFIG_HOME/labwc"

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

    WLR_BACKENDS=x11 WLR_RENDERER_ALLOW_SOFTWARE=1 labwc &
    sleep 2
    export WAYLAND_DISPLAY=wayland-0
fi

echo "=== Ready ==="
# Keep container running and execute commands passed to it, or sleep
if [ $# -gt 0 ]; then
    exec "$@"
else
    exec sleep infinity
fi
