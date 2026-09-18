#!/usr/bin/env bash
set -euo pipefail

# QuickNews - Yerel Kurulum Betigi (Local User Installation)
# Omarchy Linux ve XDG Standartlarina Tam Uyumlu

echo "[1/4] Rust motoru release modunda derleniyor..."
cargo build --release

echo "[2/4] Dizinler hazirlaniyor..."
install -d -m 755 "${HOME}/.local/bin"
install -d -m 755 "${HOME}/.local/share/applications"
install -d -m 755 "${HOME}/.local/share/quicknews"

echo "[3/4] Dosyalar kuruluyor..."
install -m 755 target/release/quicknews-engine "${HOME}/.local/bin/quicknews-engine"
install -m 755 quicknews "${HOME}/.local/bin/quicknews"
install -m 644 quicknews.desktop "${HOME}/.local/share/applications/quicknews.desktop"
rm -rf "${HOME}/.local/share/quicknews/qml"
cp -r qml "${HOME}/.local/share/quicknews/"
install -m 644 Panel.qml "${HOME}/.local/share/quicknews/Panel.qml"

echo "[4/4] Masaustu veritabani guncelleniyor..."
update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true

echo "Kurulum basariyla tamamlandi."
echo "Artik 'quicknews' komutuyla veya uygulama menusunden baslatabilirsiniz."
