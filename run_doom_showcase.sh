#!/usr/bin/env bash
set -euo pipefail

: "${DISPLAY:=:99}"
: "${WAYLAND_DISPLAY:=wayland-0}"
: "${XDG_RUNTIME_DIR:=/tmp/runtime-root}"
: "${SDL_AUDIODRIVER:=pulse}"
: "${RETROSHELL_VM_WIDTH:=1280}"
: "${RETROSHELL_VM_HEIGHT:=800}"
: "${RETROSHELL_DOOM_WINDOWED_SECONDS:=10}"
: "${RETROSHELL_DOOM_BORDERLESS_SECONDS:=10}"
: "${RETROSHELL_DOOM_FULLSCREEN_SECONDS:=10}"
: "${RETROSHELL_DOOM_GAP_SECONDS:=2}"
: "${RETROSHELL_DOOM_OUTPUT:=/app/doom_evidence.mp4}"
: "${RETROSHELL_DOOM_LOG:=${RETROSHELL_DOOM_OUTPUT%.*}.ffmpeg.log}"
: "${RETROSHELL_AUDIO_SOURCE:=}"

export DISPLAY WAYLAND_DISPLAY XDG_RUNTIME_DIR SDL_AUDIODRIVER
if [[ -z "${PULSE_SERVER:-}" ]]; then
  for pulse_socket in /tmp/pulse-*/native; do
    if [[ -S "$pulse_socket" ]]; then
      PULSE_SERVER="$pulse_socket"
      break
    fi
  done
fi
if [[ -n "${PULSE_SERVER:-}" ]]; then
  export PULSE_SERVER
fi

DOOM_BIN="${RETROSHELL_DOOM_BIN:-}"
if [[ -z "$DOOM_BIN" ]]; then
  if command -v chocolate-doom >/dev/null 2>&1; then
    DOOM_BIN="$(command -v chocolate-doom)"
  elif [[ -x /usr/games/chocolate-doom ]]; then
    DOOM_BIN=/usr/games/chocolate-doom
  else
    echo "missing chocolate-doom binary" >&2
    exit 1
  fi
fi

IWAD="${RETROSHELL_DOOM_IWAD:-}"
if [[ -z "$IWAD" ]]; then
  for candidate in \
    /usr/share/games/doom/freedoom2.wad \
    /usr/share/games/doom/freedoom1.wad; do
    if [[ -f "$candidate" ]]; then
      IWAD="$candidate"
      break
    fi
  done
fi
if [[ -z "$IWAD" || ! -f "$IWAD" ]]; then
  echo "missing Freedoom IWAD" >&2
  exit 1
fi

command -v ffmpeg >/dev/null 2>&1 || {
  echo "missing ffmpeg" >&2
  exit 1
}
command -v ffprobe >/dev/null 2>&1 || {
  echo "missing ffprobe" >&2
  exit 1
}

duration=$((RETROSHELL_DOOM_WINDOWED_SECONDS + RETROSHELL_DOOM_BORDERLESS_SECONDS + RETROSHELL_DOOM_FULLSCREEN_SECONDS + 4 * RETROSHELL_DOOM_GAP_SECONDS))
video_size="${RETROSHELL_VM_WIDTH}x${RETROSHELL_VM_HEIGHT}"
if [[ -z "$RETROSHELL_AUDIO_SOURCE" ]] && command -v pactl >/dev/null 2>&1; then
  RETROSHELL_AUDIO_SOURCE="$({ pactl info 2>/dev/null || true; } | awk -F': ' '/Default Source/ { print $2; exit }')"
fi
if [[ -z "$RETROSHELL_AUDIO_SOURCE" ]]; then
  RETROSHELL_AUDIO_SOURCE="default"
fi

rm -f "$RETROSHELL_DOOM_OUTPUT" "$RETROSHELL_DOOM_LOG"
mkdir -p "$(dirname "$RETROSHELL_DOOM_OUTPUT")"

echo "=== Doom showcase evidence ==="
echo "binary: $DOOM_BIN"
echo "iwad: $IWAD"
echo "display: $DISPLAY ($video_size)"
echo "audio source: $RETROSHELL_AUDIO_SOURCE"
echo "output: $RETROSHELL_DOOM_OUTPUT"
echo "ffmpeg log: $RETROSHELL_DOOM_LOG"

ffmpeg -y \
  -f x11grab -video_size "$video_size" -framerate 30 -i "$DISPLAY" \
  -f pulse -i "$RETROSHELL_AUDIO_SOURCE" \
  -t "$duration" \
  -c:v libx264 -preset superfast -pix_fmt yuv420p \
  -c:a aac -b:a 128k \
  "$RETROSHELL_DOOM_OUTPUT" >"$RETROSHELL_DOOM_LOG" 2>&1 &
FFMPEG_PID=$!

DOOM_PID=""
cleanup() {
  if [[ -n "${DOOM_PID:-}" ]]; then
    kill "$DOOM_PID" >/dev/null 2>&1 || true
  fi
  kill "$FFMPEG_PID" >/dev/null 2>&1 || true
}
trap cleanup EXIT

run_mode() {
  local label="$1"
  local seconds="$2"
  shift 2

  echo "=== $label ==="
  if ! kill -0 "$FFMPEG_PID" >/dev/null 2>&1; then
    echo "ffmpeg exited before $label" >&2
    tail -80 "$RETROSHELL_DOOM_LOG" >&2 || true
    exit 1
  fi
  "$DOOM_BIN" "$@" -iwad "$IWAD" &
  DOOM_PID=$!
  if command -v xdotool >/dev/null 2>&1; then
    sleep 1
    xdotool search --sync --onlyvisible --pid "$DOOM_PID" windowactivate %@ windowraise %@ >/dev/null 2>&1 || true
  fi
  sleep "$seconds"
  kill "$DOOM_PID" >/dev/null 2>&1 || true
  wait "$DOOM_PID" >/dev/null 2>&1 || true
  DOOM_PID=""
  sleep "$RETROSHELL_DOOM_GAP_SECONDS"
}

sleep "$RETROSHELL_DOOM_GAP_SECONDS"
run_mode "1. Windowed mode" "$RETROSHELL_DOOM_WINDOWED_SECONDS" -window -width 640 -height 480
run_mode "2. Borderless fullscreen-sized window" "$RETROSHELL_DOOM_BORDERLESS_SECONDS" -window -width "$RETROSHELL_VM_WIDTH" -height "$RETROSHELL_VM_HEIGHT"
run_mode "3. Exclusive fullscreen request" "$RETROSHELL_DOOM_FULLSCREEN_SECONDS" -fullscreen

if ! wait "$FFMPEG_PID"; then
  echo "ffmpeg recording failed" >&2
  tail -120 "$RETROSHELL_DOOM_LOG" >&2 || true
  exit 1
fi
trap - EXIT

if [[ ! -s "$RETROSHELL_DOOM_OUTPUT" ]]; then
  echo "recording was not created: $RETROSHELL_DOOM_OUTPUT" >&2
  tail -120 "$RETROSHELL_DOOM_LOG" >&2 || true
  exit 1
fi

ffprobe -v error \
  -select_streams v:0 \
  -show_entries stream=codec_type,width,height \
  -of default=noprint_wrappers=1 "$RETROSHELL_DOOM_OUTPUT"
ffprobe -v error \
  -select_streams a:0 \
  -show_entries stream=codec_type,sample_rate,channels \
  -of default=noprint_wrappers=1 "$RETROSHELL_DOOM_OUTPUT"

echo "=== Done: $RETROSHELL_DOOM_OUTPUT ==="
