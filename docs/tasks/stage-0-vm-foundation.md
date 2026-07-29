# Stage 0 — VM Foundation (arm64 UTM + KMS + SSH bridge)

> **For executors:** read [docs/tasks/README.md](README.md) first. Do tasks in
> order. **Stage status: UNVERIFIED** — no task here has been run on a real VM.
> Promote each to VERIFIED only after its acceptance passes; paste transcripts
> into [docs/qa/stage-0.md](../qa/stage-0.md).

**Goal:** a reproducible Arch Linux **aarch64** UTM VM with real `virtio-gpu`
KMS (`/dev/dri/card0`), reachable over SSH from the macOS host, able to build the
workspace. This is the machine the compositor has never had.

**Why this ordering:** the whole project's failure mode was "never run on real
KMS." Nothing downstream is meaningful until this VM exists and builds the tree.

## Global constraints (apply to every task)

- Host is **macOS arm64 (Apple Silicon)**; VM is **Arch aarch64**; hypervisor is
  **UTM**. Never assume x86, VirtualBox, or vmwgfx.
- The VM disk is **`/dev/vda`** (virtio-blk), not `/dev/sda`.
- Boot is **UEFI** (UTM ships edk2/AAVMF). aarch64 EFI binary is `BOOTAA64.EFI`.
- GPU is **virtio-gpu**; the kernel module is **`virtio_gpu`**. There is no
  `vmwgfx`.
- Repo: `https://github.com/palaashatri/retroshell.git`, branch `main`.

## Two paths to the VM — pick one

There are two ways to get the Stage 0 VM. **Do not mix them.**

- **Path A — Prebuilt image (recommended, chosen 2026-07-30):** import the UTM
  gallery **Arch Linux ARM** prebuilt VM (`https://mac.getutm.app/gallery/archlinux-arm`).
  The OS is already installed. You **provision on top** with
  `packaging/vm/provision-arm64.sh`. This matches Stage 4's layer-onto-existing
  model. **Skip Tasks 0.1, 0.3, 0.4** (the from-scratch path) and do
  **0.1A / 0.3A / 0.4A** instead. Tasks 0.2, 0.5, 0.6, 0.7, 0.8 are shared.
- **Path B — From scratch (ISO):** Tasks 0.1, 0.3, 0.4 build the VM from an
  aarch64 Arch/archboot ISO with `arch-install-arm64.sh`. Kept for the Stage 4
  bootable ISO and clean-room reproduction.

> **⚠️ NEVER run `arch-install-arm64.sh` on a Path-A prebuilt image.** It runs
> `sgdisk --zap-all /dev/vda` and reformats the disk — it will destroy the
> prebuilt system. The installer is Path B only.

---

### Task 0.1A — Import and boot the UTM gallery Arch Linux ARM prebuilt VM   [UNVERIFIED · Path A]

Host GUI + human. An agent cannot click UTM.

Steps:
1. Download the **Arch Linux ARM** VM from `https://mac.getutm.app/gallery/archlinux-arm`
   and open it in UTM (double-click the `.utm`/downloaded bundle to import).
2. Before first boot, in **VM Settings → Display**, confirm the emulated GPU is
   **virtio-gpu (GL)** / **virtio-ramfb-gl**. If it is a plain VGA device, change
   it to virtio-gpu — KMS/`/dev/dri` depends on it. In **Network**, keep
   **Shared Network** (gateway `10.0.2.2`).
3. Boot. Log in. **CONFIRM AT RUNTIME:** the default credentials ship with the
   image — the gallery page documents them. Try the documented user; a common
   fallback for this image family is `root` / `root`. Record the working
   credentials in `docs/qa/stage-0.md`. Do not assume; verify what logs you in.

Acceptance (in the VM, the Stage-0 gate check up front):
```bash
uname -m                 # → aarch64
ls /dev/dri/card0        # → /dev/dri/card0   (virtio-gpu KMS present)
lspci | grep -i vga ; lsmod | grep virtio_gpu   # driver/module evidence
```
→ expect: `aarch64` and `/dev/dri/card0` present. **If `card0` is absent**, the
imported bundle's display device is not DRM/KMS — go back to step 2, set the
display to virtio-gpu, reboot, and re-check. Record the result either way in
`docs/qa/stage-0.md` (Task 0.1A row + the credentials + driver values).

DO NOT:
- Run `arch-install-arm64.sh` (it reformats `/dev/vda` — destroys this image).
- Proceed past a missing `/dev/dri/card0` — fix the display device first.

Commit: _none (host/GUI action)._

---

### Task 0.3A — Create `packaging/vm/provision-arm64.sh` (Path A)   [UNVERIFIED · Path A]

Repo-side file authoring (no VM needed). This is the provision-on-top counterpart
to `arch-install-arm64.sh`: it installs deps on an already-installed system and
does **not** touch partitions.

Precondition:
```bash
test -f packaging/vm/arch-install-arm64.sh && echo ok   # → ok (Path B installer exists)
```

Files: Create `packaging/vm/provision-arm64.sh`

Steps:
1. Create the file with exactly this content:

```bash
#!/usr/bin/env bash
# Provision an ALREADY-INSTALLED Arch Linux ARM VM (e.g. the UTM gallery image)
# with RetroShell's build + runtime deps, the host SSH key, tty1 autologin, and a
# built workspace. Run INSIDE the VM as a sudo-capable user:
#   curl -sL http://10.0.2.2:8000/provision-arm64.sh | bash
#
# This does NOT partition or format any disk. It is safe to run on a live system,
# and idempotent (safe to re-run). It is the Path-A (prebuilt image) counterpart
# to arch-install-arm64.sh, and matches Stage 4's layer-onto-existing model.
set -euxo pipefail

USERNAME="${SUDO_USER:-$(whoami)}"
REPO_URL="${REPO_URL:-https://github.com/palaashatri/retroshell.git}"
REPO_BRANCH="${REPO_BRANCH:-main}"
HOST_HTTP="${HOST_HTTP:-http://10.0.2.2:8000}"

echo "=== refresh keyring + sync ==="
sudo pacman -Sy --noconfirm archlinux-keyring || true

echo "=== install build + runtime deps (no base/kernel; system already installed) ==="
sudo pacman -S --needed --noconfirm \
  base-devel pkgconf git curl wget rust \
  wayland wayland-protocols libxkbcommon libinput seatd libdrm mesa \
  vulkan-icd-loader vulkan-swrast vulkan-tools \
  libdisplay-info pixman \
  dbus at-spi2-core \
  fontconfig freetype2 ttf-dejavu ttf-liberation \
  pipewire pipewire-pulse wireplumber libpipewire \
  polkit xorg-xwayland labwc foot \
  imagemagick grim wl-clipboard \
  networkmanager nm-connection-editor upower \
  openssh htop qemu-guest-agent

echo "=== ensure virtio_gpu in initramfs (KMS at boot) ==="
if ! grep -q 'virtio_gpu' /etc/mkinitcpio.conf; then
  sudo sed -i 's/^MODULES=(\(.*\))/MODULES=(\1 virtio_gpu)/' /etc/mkinitcpio.conf
  sudo mkinitcpio -P
fi

echo "=== groups for seat/DRM/input ==="
sudo usermod -aG video,input "$USERNAME"
sudo usermod -aG seat "$USERNAME" || true   # 'seat' group may not exist until seatd

echo "=== services ==="
sudo systemctl enable --now seatd || true
sudo systemctl enable --now sshd
sudo systemctl enable --now qemu-guest-agent || true

echo "=== install host SSH public key ==="
install -d -m 700 "$HOME/.ssh"
curl -sL "$HOST_HTTP/qa_key.pub" -o "$HOME/.ssh/authorized_keys"
chmod 600 "$HOME/.ssh/authorized_keys"

echo "=== autologin on tty1 ==="
sudo mkdir -p /etc/systemd/system/getty@tty1.service.d
sudo tee /etc/systemd/system/getty@tty1.service.d/autologin.conf >/dev/null <<EOF
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin $USERNAME --noclear %I \$TERM
EOF

echo "=== clone + build RetroShell ==="
if [ ! -d "$HOME/retroshell" ]; then
  git clone --branch "$REPO_BRANCH" "$REPO_URL" "$HOME/retroshell" \
    || git clone "$REPO_URL" "$HOME/retroshell"
fi
mkdir -p "$HOME/.config/retroshell"
cat > "$HOME/.config/retroshell/settings.conf" <<EOF
theme=classic
appearance=light
lock_password=retroshell
EOF
cd "$HOME/retroshell"
cargo build --release --workspace

echo "=== done; reboot so virtio_gpu + group membership take effect ==="
```

2. Make it executable and syntax-check it:
   ```bash
   chmod +x packaging/vm/provision-arm64.sh
   bash -n packaging/vm/provision-arm64.sh && echo "syntax-ok"
   ```

Acceptance:
```bash
bash -n packaging/vm/provision-arm64.sh && echo "syntax-ok"        # → syntax-ok
grep -c 'sgdisk\|mkfs\|--zap-all' packaging/vm/provision-arm64.sh   # → 0 (never formats)
grep -q 'pacman -S --needed' packaging/vm/provision-arm64.sh && echo "installs-on-top"  # → installs-on-top
```
→ expect: `syntax-ok`, a format-command count of exactly `0`, and `installs-on-top`.

DO NOT:
- Add any partition/format command (`sgdisk`, `mkfs`, `parted`, `--zap-all`) — this
  script must be safe to run on a live, already-installed system.
- Touch `arch-install-arm64.sh`.

Commit: `feat(vm): provision-on-top script for prebuilt Arch ARM VM (Path A)`

---

### Task 0.4A — Run the provision script in the prebuilt VM (deps, SSH key, autologin, build)   [UNVERIFIED · Path A]

Runs **inside the VM**, on the already-installed system. Requires Task 0.2 (host
serving `qa_key.pub` + scripts) and Task 0.1A (VM booted with `/dev/dri/card0`).

Precondition (host serving files from Task 0.2):
```bash
curl -s http://127.0.0.1:8000/provision-arm64.sh | head -1   # → #!/usr/bin/env bash
```

Steps:
1. In the VM (as a sudo-capable user), run:
   ```bash
   curl -sL http://10.0.2.2:8000/provision-arm64.sh | bash
   ```
   This installs build+runtime deps with `pacman -S` (no partitioning), ensures
   `virtio_gpu` in initramfs, adds the user to `video`/`input`/`seat`, enables
   `seatd`+`sshd`, installs the host SSH key, sets tty1 autologin, and clones +
   `cargo build --release --workspace`.
2. Reboot so group membership and initramfs changes take effect.

Acceptance (in the VM after reboot):
```bash
systemctl is-active sshd                 # → active
ls ~/retroshell/target/release/retro-compositor ~/retroshell/target/release/retro-shell
groups | grep -o 'seat\|video\|input'    # → video input (seat if present)
```
→ expect: `active`, both release binaries present, and `video`+`input` groups.
This replaces Path B's Tasks 0.4/0.6 for the prebuilt image. Continue at Task 0.5.

DO NOT:
- Run `arch-install-arm64.sh` here — it reformats the disk.
- Rsync the working tree yet — that is Task 0.6 (do it after SSH works, 0.5).

Commit: `feat(vm): provision-on-top script for prebuilt Arch ARM VM (Path A)`

---

### Task 0.1 — Create the UTM aarch64 VM and boot an Arch aarch64 live ISO   [UNVERIFIED · Path B]

This task is **host GUI + human**: an agent cannot click UTM. Do it by hand, then
report the console prompt you land on.

Precondition:
```bash
# On the host:
utmctl --version    # UTM CLI present. If "command not found", install UTM from
                    # https://mac.getutm.app and ensure /Applications/UTM.app.
```

Steps:
1. Obtain an **aarch64** Arch install ISO. Arch Linux proper is x86-only; for
   aarch64 use **archboot** (maintained aarch64 Arch ISOs): download the latest
   `aarch64` image from https://archboot.com (or its release mirror). **CONFIRM
   AT RUNTIME:** the file must be an `aarch64` ISO — verify with
   `file <iso>` → it should mention `ARM64`/`aarch64`. Do not use an x86_64 ISO.
2. In UTM: **Create a New Virtual Machine → Virtualize → Linux**. Do **not** use
   Emulate (that would be slow x86). Attach the aarch64 ISO as the boot image.
3. Set: Architecture `ARM64 (aarch64)`; Memory ≥ 4096 MB; CPU ≥ 4 cores; a new
   ≥ 32 GB disk (this becomes `/dev/vda`).
4. **Display:** set the graphics card to **virtio-gpu (GL)** if offered, else
   **virtio-ramfb-gl** / **virtio-gpu**. This is what provides KMS. Do NOT pick a
   "VGA"-only device.
5. **Network:** keep the default **Shared Network** (UTM's slirp) — the gateway
   is `10.0.2.2`, matching the install scripts.
6. Boot the VM. Wait for the Arch/archboot live shell prompt (usually a root
   shell or a login you complete as `root`).

Acceptance:
```bash
# Inside the VM's live console:
uname -m            # → aarch64
ls /dev/vda         # → /dev/vda   (the virtio disk exists)
pacman --version    # → prints a version (pacman is available in the live env)
```
→ expect: `aarch64`, `/dev/vda` present, `pacman` runs.

DO NOT:
- Use UTM "Emulate" mode or an x86_64 ISO (wrong architecture, unusably slow).
- Pick a non-virtio display device — KMS/`/dev/dri` depends on virtio-gpu.
- Continue to Task 0.4 before this acceptance passes.

Commit: _none (host/GUI action; nothing to commit)._

---

### Task 0.2 — Generate an SSH key and serve the install files from the host   [UNVERIFIED]

The VM will pull the installer and your public key over HTTP from the host at
`10.0.2.2` (UTM's slirp gateway), the same pattern the repo already uses.

Precondition:
```bash
cd "$(git rev-parse --show-toplevel)" && test -d packaging/vm && echo ok   # → ok
```

Steps:
1. Generate a dedicated keypair (no passphrase, for automation):
   ```bash
   ssh-keygen -t ed25519 -N "" -f packaging/vm/qa_key -C retroshell-vm
   ```
   (`packaging/vm/qa_key*` is already gitignored — see `.gitignore`.)
2. From the repo root, serve the `packaging/vm` directory so the VM can fetch the
   installer and the public key:
   ```bash
   ( cd packaging/vm && python3 -m http.server 8000 ) &
   ```
   Leave this running until the install finishes.

Acceptance:
```bash
curl -s http://127.0.0.1:8000/qa_key.pub | head -c 20   # → begins "ssh-ed25519 AAAA"
```
→ expect: the first bytes of your public key.

DO NOT:
- Commit `qa_key` or `qa_key.pub` (they are gitignored; keep it that way).
- Bind the HTTP server to a public interface — `python3 -m http.server` on
  localhost is reachable by the VM via `10.0.2.2` without exposing it externally.

Commit: _none (local key + transient server; nothing tracked changes)._

---

### Task 0.3 — Create the arm64 installer `packaging/vm/arch-install-arm64.sh`   [UNVERIFIED]

This is the x86-VirtualBox `arch-install.sh` rewritten for aarch64 UTM +
virtio-gpu, with the exact deltas the research identified. It is a **new file**;
leave the old `arch-install.sh` in place for reference.

Precondition:
```bash
test -f packaging/vm/arch-install.sh && echo ok   # → ok (reference exists)
```

Files: Create `packaging/vm/arch-install-arm64.sh`

Steps:
1. Create the file with exactly this content:

```bash
#!/usr/bin/env bash
# Unattended Arch (aarch64) install for the RetroShell UTM verification VM.
#
# Run from the aarch64 Arch/archboot live environment:
#   curl -sL http://10.0.2.2:8000/arch-install-arm64.sh | bash
#
# Produces a machine that boots to an autologin TTY with sshd + the host's key,
# on real virtio-gpu DRM/KMS — the environment RetroShell has never been run on.
set -euxo pipefail

DISK=/dev/vda                       # virtio-blk (NOT /dev/sda)
HOSTNAME=retroshell-vm
USERNAME=retro
PASSWORD=retro
REPO_URL="${REPO_URL:-https://github.com/palaashatri/retroshell.git}"
REPO_BRANCH="${REPO_BRANCH:-main}"
HOST_HTTP="${HOST_HTTP:-http://10.0.2.2:8000}"   # host file server (Task 0.2)

# CONFIRM AT RUNTIME: which aarch64 kernel package this live env provides.
#   pacman -Ss '^linux$' ; pacman -Ss '^linux-aarch64$'
# archboot-based aarch64 Arch typically uses `linux`; Arch Linux ARM uses
# `linux-aarch64`. Override with KERNEL_PKG=... if the default is not found.
KERNEL_PKG="${KERNEL_PKG:-linux}"

echo "=== clock + keyring ==="
timedatectl set-ntp true || true
pacman -Sy --noconfirm archlinux-keyring || true

echo "=== partition $DISK (GPT: 512M ESP + rest root) ==="
sgdisk --zap-all "$DISK"
sgdisk -n 1:0:+512M -t 1:ef00 -c 1:EFI "$DISK"
sgdisk -n 2:0:0     -t 2:8300 -c 2:ROOT "$DISK"
partprobe "$DISK"
sleep 2
mkfs.fat -F32 "${DISK}1"
mkfs.ext4 -F "${DISK}2"
mount "${DISK}2" /mnt
mkdir -p /mnt/boot
mount "${DISK}1" /mnt/boot

echo "=== pacstrap base + RetroShell build/runtime deps (aarch64) ==="
pacstrap -K /mnt \
  base "$KERNEL_PKG" linux-firmware \
  networkmanager sudo vim nano git curl wget \
  base-devel pkgconf \
  rust \
  wayland wayland-protocols libxkbcommon libinput seatd libdrm mesa \
  vulkan-icd-loader vulkan-swrast vulkan-tools \
  libdisplay-info pixman \
  dbus at-spi2-core \
  fontconfig freetype2 ttf-dejavu ttf-liberation \
  pipewire pipewire-pulse wireplumber libpipewire \
  polkit \
  xorg-xwayland \
  labwc foot \
  imagemagick grim wl-clipboard \
  networkmanager nm-connection-editor \
  upower \
  openssh htop \
  qemu-guest-agent \
  grub efibootmgr

genfstab -U /mnt >> /mnt/etc/fstab

echo "=== configure the installed system in chroot ==="
arch-chroot /mnt /bin/bash -euxo pipefail <<CHROOT
ln -sf /usr/share/zoneinfo/UTC /etc/localtime
hwclock --systohc || true
echo "en_US.UTF-8 UTF-8" >> /etc/locale.gen
locale-gen
echo "LANG=en_US.UTF-8" > /etc/locale.conf
echo "$HOSTNAME" > /etc/hostname

# Load virtio_gpu early so KMS is up at boot (virtio-gpu provides /dev/dri).
sed -i 's/^MODULES=.*/MODULES=(virtio_gpu)/' /etc/mkinitcpio.conf
mkinitcpio -P

# Users
echo "root:$PASSWORD" | chpasswd
useradd -m -G wheel,video,input,seat -s /bin/bash $USERNAME
echo "$USERNAME:$PASSWORD" | chpasswd
echo "%wheel ALL=(ALL:ALL) NOPASSWD: ALL" > /etc/sudoers.d/wheel

# Services
systemctl enable NetworkManager
systemctl enable seatd
systemctl enable sshd
systemctl enable qemu-guest-agent || true

# Bootloader: aarch64 UEFI, removable so UTM's edk2 finds BOOTAA64.EFI.
grub-install --target=arm64-efi --efi-directory=/boot --bootloader-id=GRUB --removable
sed -i 's/^GRUB_TIMEOUT=.*/GRUB_TIMEOUT=1/' /etc/default/grub
grub-mkconfig -o /boot/grub/grub.cfg

# Autologin retro on tty1 so the VM lands in a shell.
mkdir -p /etc/systemd/system/getty@tty1.service.d
cat > /etc/systemd/system/getty@tty1.service.d/autologin.conf <<EOF
[Service]
ExecStart=
ExecStart=-/sbin/agetty -o '-p -f -- \\\\u' --noclear --autologin $USERNAME %I \$TERM
EOF

# Install the host's SSH public key for the retro user.
install -d -m 700 -o $USERNAME -g $USERNAME /home/$USERNAME/.ssh
curl -sL $HOST_HTTP/qa_key.pub -o /home/$USERNAME/.ssh/authorized_keys
chown $USERNAME:$USERNAME /home/$USERNAME/.ssh/authorized_keys
chmod 600 /home/$USERNAME/.ssh/authorized_keys
CHROOT

echo "=== clone + build RetroShell as $USERNAME ==="
arch-chroot /mnt /bin/bash -euxo pipefail <<CHROOT
su - $USERNAME -c '
  set -euxo pipefail
  git clone --branch "$REPO_BRANCH" "$REPO_URL" ~/retroshell || git clone "$REPO_URL" ~/retroshell
  mkdir -p ~/.config/retroshell
  cat > ~/.config/retroshell/settings.conf <<EOF
theme=classic
appearance=light
lock_password=retroshell
EOF
'
CHROOT

echo "=== done; rebooting into the installed system ==="
umount -R /mnt
systemctl reboot
```

2. Make it executable and shellcheck it:
   ```bash
   chmod +x packaging/vm/arch-install-arm64.sh
   shellcheck packaging/vm/arch-install-arm64.sh || true   # informational
   ```

Acceptance:
```bash
bash -n packaging/vm/arch-install-arm64.sh && echo "syntax-ok"   # → syntax-ok
grep -c '/dev/vda\|arm64-efi\|virtio_gpu' packaging/vm/arch-install-arm64.sh
# → 3 or more (the arm64 deltas are present)
```
→ expect: `syntax-ok` and a count ≥ 3.

DO NOT:
- Edit or delete the old `packaging/vm/arch-install.sh` — keep it for reference.
- Change `DISK` back to `/dev/sda` or the target to `x86_64-efi`.
- Add `vmwgfx` anywhere.

Commit: `feat(vm): arm64 UTM Arch installer (virtio-gpu KMS, SSH bridge)`

---

### Task 0.4 — Run the installer in the VM; reboot into the installed system   [UNVERIFIED]

Precondition (host HTTP server from Task 0.2 is up):
```bash
curl -s http://127.0.0.1:8000/arch-install-arm64.sh | head -1   # → #!/usr/bin/env bash
```

Steps:
1. In the VM live console (from Task 0.1), first confirm the kernel package name,
   then run the installer:
   ```bash
   pacman -Ss '^linux$' | head -1        # confirm `linux` exists for aarch64...
   pacman -Ss '^linux-aarch64$' | head -1 # ...or that this variant is the one
   # If the default `linux` is NOT available, prefix the next line with
   #   KERNEL_PKG=linux-aarch64
   curl -sL http://10.0.2.2:8000/arch-install-arm64.sh | bash
   ```
2. The script partitions `/dev/vda`, installs, configures, clones+configures
   RetroShell, then **reboots**. After reboot the VM autologs in as `retro` on
   tty1. In UTM, detach the ISO (or set disk first in boot order) so it boots
   from `/dev/vda`.

Acceptance (in the VM console after reboot):
```bash
whoami          # → retro   (autologin worked)
systemctl is-active sshd     # → active
ls /dev/dri/    # → card0 (and likely renderD128) — virtio-gpu KMS is up
```
→ expect: `retro`, `active`, and `card0` present.

DO NOT:
- Reboot back into the ISO — boot from the installed disk.
- Proceed if `/dev/dri/card0` is absent — that means virtio-gpu/KMS is not up;
  re-check the VM's display device (Task 0.1 step 4) and the `virtio_gpu` module
  (`lsmod | grep virtio_gpu`). Record the failure in `docs/qa/stage-0.md`.

Commit: _none (VM-side action; nothing in the repo changes)._

---

### Task 0.5 — Connect to the VM over SSH from the host   [UNVERIFIED]

Precondition:
```bash
test -f packaging/vm/qa_key && echo ok   # → ok
```

Steps:
1. In UTM, ensure the VM's network forwards host → VM:22. With **Shared Network**,
   add a port forward (VM Settings → Network → Port Forwarding): host port
   `2222` → guest `22`. (Emulated VLAN also allows reaching the guest IP directly;
   port-forward is the portable default.)
2. From the host:
   ```bash
   ssh -i packaging/vm/qa_key -p 2222 \
       -o StrictHostKeyChecking=accept-new retro@127.0.0.1 'uname -m'
   ```

Acceptance:
```bash
ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 'uname -m'   # → aarch64
```
→ expect: `aarch64` printed from the host, over SSH, with no password prompt.

DO NOT:
- Use password auth — the key from Task 0.2 must work. If it prompts for a
  password, the `authorized_keys` step failed; re-check Task 0.3.

Commit: _none (host-side connection; nothing tracked changes)._

---

### Task 0.6 — Sync the working tree and build the workspace in the VM   [UNVERIFIED]

The VM cloned `main` at install time. To test *local* changes, push the working
tree over SSH.

Precondition:
```bash
ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 'test -d ~/retroshell && echo ok'  # → ok
```

Steps:
1. Rsync the working tree (excluding build artifacts) from host to VM:
   ```bash
   rsync -az --delete \
     --exclude target/ --exclude target-docker/ --exclude .git/ \
     -e "ssh -i packaging/vm/qa_key -p 2222" \
     ./ retro@127.0.0.1:~/retroshell/
   ```
2. Build the release workspace in the VM:
   ```bash
   ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 \
     'cd ~/retroshell && cargo build --release --workspace 2>&1 | tail -20'
   ```

Acceptance:
```bash
ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 \
  'ls -1 ~/retroshell/target/release/retro-compositor ~/retroshell/target/release/retro-shell'
# → both paths printed (binaries exist)
```
→ expect: both `retro-compositor` and `retro-shell` release binaries exist.

DO NOT:
- Rsync the `target/` directory (huge; and host artifacts are x86 — wrong arch).
- Run `cargo build` on the host expecting it to help the VM — the VM is aarch64
  Linux; only its own build counts.

Commit: _none (this is a build verification, not a repo change)._

---

### Task 0.7 — KMS probe (Stage 0 definition-of-done)   [UNVERIFIED]

Precondition:
```bash
ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 'echo ok'   # → ok
```

Steps:
1. Probe the DRM device and driver in the VM:
   ```bash
   ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 '
     ls -l /dev/dri/card0
     lsmod | grep virtio_gpu
     cat /sys/class/drm/card0/device/uevent 2>/dev/null | head
   '
   ```

Acceptance:
```bash
ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 \
  'ls /dev/dri/card0 && cargo build --release --workspace --manifest-path ~/retroshell/Cargo.toml >/dev/null 2>&1 && echo STAGE0-DOD-PASS'
# → /dev/dri/card0
# → STAGE0-DOD-PASS
```
→ expect: `card0` exists AND the workspace builds — the exact Stage 0 DoD from
`PROGRAM.md`. Paste this transcript into `docs/qa/stage-0.md` and mark Stage 0
tasks VERIFIED.

DO NOT:
- Declare Stage 0 done without `STAGE0-DOD-PASS` actually printing.

Commit: _none (record the transcript in qa/stage-0.md instead)._

---

### Task 0.8 — Confirm Linux CI covers the workspace build   [UNVERIFIED→quick]

CI already builds on Linux (`.github/workflows/ci.yml`). This task only confirms
it and records the fact — do **not** rewrite a working workflow.

Precondition:
```bash
test -f .github/workflows/ci.yml && echo ok   # → ok
```

Steps:
1. Confirm the workflow builds the workspace on Linux:
   ```bash
   grep -n 'runs-on: ubuntu-latest' .github/workflows/ci.yml
   grep -n 'cargo build --workspace' .github/workflows/ci.yml
   ```

Acceptance:
```bash
grep -q 'ubuntu-latest' .github/workflows/ci.yml \
  && grep -q 'cargo build --workspace' .github/workflows/ci.yml \
  && echo CI-LINUX-BUILD-PRESENT
# → CI-LINUX-BUILD-PRESENT
```
→ expect: `CI-LINUX-BUILD-PRESENT`.

DO NOT:
- Rewrite or "improve" the CI workflow in this task. It works; leave it.

Commit: _none (verification only)._
