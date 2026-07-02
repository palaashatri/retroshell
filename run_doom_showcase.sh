#!/usr/bin/env bash
set -ex

export DISPLAY=:99
export WAYLAND_DISPLAY=wayland-0
export XDG_RUNTIME_DIR=/tmp/runtime-root
export SDL_AUDIODRIVER=pulse
export PULSE_SERVER=127.0.0.1

# Clean up any previous recording
rm -f /app/doom_evidence.mp4

echo "=== Starting ffmpeg screen & audio recording ==="
# Capture virtual display :99 and monitor of default null sink for PulseAudio
# We record for 40 seconds to cover all states
ffmpeg -y -f x11grab -video_size 1280x800 -i :99 -f pulse -i default -t 40 -c:v libx264 -preset superfast -pix_fmt yuv420p -c:a aac -b:a 128k /app/doom_evidence.mp4 &
FFMPEG_PID=$!

sleep 3

echo "=== 1. Launching Doom in Windowed Mode ==="
/usr/games/chocolate-doom -window -width 640 -height 480 -iwad /usr/share/games/doom/freedoom2.wad &
DOOM_PID=$!
sleep 10
kill -9 $DOOM_PID || true
sleep 2

echo "=== 2. Launching Doom in Borderless Fullscreen Mode ==="
# Use windowed mode matching screen size to simulate borderless fullscreen
/usr/games/chocolate-doom -window -width 1280 -height 800 -iwad /usr/share/games/doom/freedoom2.wad &
DOOM_PID=$!
sleep 10
kill -9 $DOOM_PID || true
sleep 2

echo "=== 3. Launching Doom in Exclusive Fullscreen Mode ==="
/usr/games/chocolate-doom -fullscreen -iwad /usr/share/games/doom/freedoom2.wad &
DOOM_PID=$!
sleep 10
kill -9 $DOOM_PID || true
sleep 2

echo "=== Waiting for ffmpeg to finish recording ==="
wait $FFMPEG_PID || true

echo "=== Done! Video saved at /app/doom_evidence.mp4 ==="
