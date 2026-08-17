#!/usr/bin/env bash
# provision-rvkit.sh - Provisionne un environnement de dev/test pour rvkit.
# Cible : Ubuntu/Debian. Installe la toolchain, les outils de flash et les regles udev.
set -euo pipefail

ZIG_VERSION="0.14.0"
ARCH="$(uname -m)"

log() { printf '[rvkit-setup] %s\n' "$1"; }

log "1/5 Dependances systeme"
sudo apt-get update -y
sudo apt-get install -y curl build-essential pkg-config libudev-dev python3-pip

log "2/5 Toolchain Rust (rustup)"
if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
fi

log "3/5 Compilateur Zig ${ZIG_VERSION}"
if ! command -v zig >/dev/null 2>&1; then
  ZIG_DIR="zig-linux-${ARCH}-${ZIG_VERSION}"
  curl -sSL "https://ziglang.org/download/${ZIG_VERSION}/${ZIG_DIR}.tar.xz" -o /tmp/zig.tar.xz
  sudo tar -xJf /tmp/zig.tar.xz -C /opt
  sudo ln -sf "/opt/${ZIG_DIR}/zig" /usr/local/bin/zig
fi

log "4/5 Outils de flash (wlink + esptool)"
cargo install wlink || true
pip3 install --user esptool

log "5/5 Regles udev (flash sans root)"
sudo tee /etc/udev/rules.d/99-rvkit.rules >/dev/null <<'RULES'
# WCH-LinkE (CH32V003)
SUBSYSTEM=="usb", ATTR{idVendor}=="1a86", ATTR{idProduct}=="8010", MODE="0666"
# Adaptateurs USB-serie usuels (ESP32-C3 / FT232)
SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", MODE="0666"
RULES
sudo udevadm control --reload-rules && sudo udevadm trigger

log "Termine. Verifie : cargo --version ; zig version ; wlink --version"
