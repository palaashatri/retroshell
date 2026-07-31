#!/usr/bin/env bash
# archiso profile definition for RetroShell

iso_name="retroshell"
iso_label="RETROSHELL"
iso_publisher="RetroShell Contributors"
iso_application="RetroShell Live Environment"
iso_version="0.1.0"
install_dir="arch"
bootmodes=('uefi-x64.systemd-boot.esp' 'uefi-ia32.systemd-boot.esp' 'bios.syslinux.mbr')
arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'xz' '-Xbcj' 'x86' '-b' '1M')
file_permissions=(
  ["/etc/shadow"]="0:0:400"
  ["/root"]="0:0:750"
  ["/root/.automated_script.sh"]="0:0:755"
  ["/usr/local/bin/start-retroshell"]="0:0:755"
)
