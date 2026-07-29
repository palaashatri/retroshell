#!/usr/bin/env bash
# Stage 2 hold script — compositor + foot; host injects keys via VBox.
set -uxo pipefail

QA=~/qa-stage2
mkdir -p "$QA"
exec > >(tee "$QA/run.log") 2>&1

pkill -f './target/release/retro-compositor' 2>/dev/null || true
pkill -x foot 2>/dev/null || true
pkill -f retro-lock 2>/dev/null || true
pkill -x finder 2>/dev/null || true
sleep 1

export RETROSHELL_LOCK_PASSWORD=retroshell
export RUST_LOG=info
export RUST_BACKTRACE=1
export XDG_RUNTIME_DIR=/run/user/$(id -u)
mkdir -p "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
unset DISPLAY WAYLAND_DISPLAY RETROSHELL_FORCE_LABWC RETROSHELL_COMPOSITOR
export LANG=C.UTF-8
export LC_ALL=C.UTF-8

cd ~/retroshell
setsid env XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" ./target/release/retro-compositor \
  > "$QA/compositor.log" 2>&1 < /dev/null &
COMP=$!

SOCK=""
for _ in $(seq 1 60); do
  kill -0 "$COMP" 2>/dev/null || break
  S=$(ls "$XDG_RUNTIME_DIR" 2>/dev/null | grep -E '^wayland-[0-9]+$' | head -1 || true)
  if [ -n "$S" ] && grep -q "WAYLAND_DISPLAY=$S" "$QA/compositor.log" 2>/dev/null; then
    SOCK=$S
    break
  fi
  sleep 0.5
done

if [ -z "$SOCK" ] || ! kill -0 "$COMP" 2>/dev/null; then
  echo "COMPOSITOR_UP=NO" > "$QA/STATUS"
  tail -40 "$QA/compositor.log"
  exit 1
fi
export WAYLAND_DISPLAY="$SOCK"
echo "COMPOSITOR_UP=YES socket=$SOCK" > "$QA/STATUS"

client_env() {
  env XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" WAYLAND_DISPLAY="$WAYLAND_DISPLAY" LANG=C.UTF-8 LC_ALL=C.UTF-8 "$@"
}

client_env setsid foot > "$QA/foot.log" 2>&1 < /dev/null &
sleep 5
pgrep -x foot >/dev/null && echo "FOOT_ALIVE=YES" >> "$QA/STATUS" || {
  echo "FOOT_ALIVE=NO" >> "$QA/STATUS"
  tail -20 "$QA/foot.log"
  exit 1
}

marker() {
  echo "$1" > "$QA/MARKER"
  echo "MARKER=$1" >> "$QA/run.log"
  sleep "${2:-120}"
}

marker WAIT_INPUT 150
marker WAIT_SUPER_O 150
marker WAIT_SUPER_L 150
marker WAIT_LOCK_BYPASS 150
marker WAIT_UNLOCK 150
marker WAIT_SUPER_O2 150

echo "STAGE2_VERIFY_DONE" >> "$QA/STATUS"
sleep 60
