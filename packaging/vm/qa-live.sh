#!/usr/bin/env bash
# Leave a live RetroShell session running on the VM's DRM/KMS so the host can
# capture the real framebuffer with `VBoxManage controlvm ... screenshotpng`.
set -u
QA=/home/retro/qa; mkdir -p "$QA"
exec > >(tee "$QA/live.log") 2>&1

pkill -f retro-compositor 2>/dev/null
pkill -f 'release/(retro-shell|finder|terminal|textedit|settings|appstore)' 2>/dev/null
sleep 1

export XDG_RUNTIME_DIR=/run/user/$(id -u)
mkdir -p "$XDG_RUNTIME_DIR"; chmod 700 "$XDG_RUNTIME_DIR"
mkdir -p "$HOME/.config/retroshell"
cat > "$HOME/.config/retroshell/settings.conf" <<EOF
theme=${RS_THEME:-classic}
appearance=${RS_APPEARANCE:-light}
hdr_requested=${RS_HDR:-false}
vrr_adaptive=${RS_VRR:-false}
refresh_rate=60hz
color_space=srgb
lock_password=retroshell
EOF
unset DISPLAY WAYLAND_DISPLAY
export RUST_LOG=info RUST_BACKTRACE=1
export RETROSHELL_COMPOSITOR_WIDTH=1280 RETROSHELL_COMPOSITOR_HEIGHT=800

setsid retro-compositor > "$QA/compositor.log" 2>&1 < /dev/null &
sleep 4
SOCK=$(ls "$XDG_RUNTIME_DIR" | grep -E '^wayland-[0-9]+$' | head -1)
[ -z "$SOCK" ] && { echo "no socket"; tail -20 "$QA/compositor.log"; exit 1; }
export WAYLAND_DISPLAY="$SOCK"
echo "WAYLAND_DISPLAY=$SOCK"

setsid retro-shell > "$QA/shell.log" 2>&1 < /dev/null &
sleep 8
for app in "$@"; do
  setsid "$app" > "$QA/$app.log" 2>&1 < /dev/null &
  sleep 6
done

echo "--- live processes ---"
pgrep -a -f 'retro-compositor|retro-shell|finder|terminal|textedit|settings|appstore' | sed 's/ .*release\// /'
echo "--- frame pump ---"
N1=$(grep -c "submission index" "$QA/shell.log" 2>/dev/null || echo 0)
sleep 6
N2=$(grep -c "submission index" "$QA/shell.log" 2>/dev/null || echo 0)
echo "shell wgpu submissions: $N1 -> $N2"
[ "$N2" -gt "$N1" ] && echo "FRAME_PUMP=RUNNING" || echo "FRAME_PUMP=STALLED"
echo "--- compositor window state ---"
grep -E "toplevel mapped|workspace active" "$QA/compositor.log" | tail -6
echo LIVE_SESSION_UP
