# Maintainer: Ozan Ozdil
pkgname=quicknews
pkgver=0.2.3
pkgrel=1
pkgdesc="Guvenli, reklamsiz, resimsiz ve yapay zeka destekli minimalist haber okuyucu"
arch=('x86_64' 'aarch64')
url="https://github.com/omarchy/quicknews"
license=('MIT')
depends=('quickshell' 'qt6-declarative' 'qt6-svg' 'ttf-jetbrains-mono-nerd')
makedepends=('cargo' 'rust')
source=()
sha256sums=()

build() {
    cargo build --release --locked
}

package() {
    install -Dm755 "target/release/quicknews-engine" "${pkgdir}/usr/bin/quicknews-engine"
    install -Dm755 "quicknews" "${pkgdir}/usr/bin/quicknews"
    install -Dm644 "quicknews.desktop" "${pkgdir}/usr/share/applications/quicknews.desktop"
    
    # QML assets
    install -d "${pkgdir}/usr/share/quicknews"
    cp -r qml "${pkgdir}/usr/share/quicknews/"
    install -Dm644 "Panel.qml" "${pkgdir}/usr/share/quicknews/Panel.qml"
    
    # Documentation & License
    install -Dm644 "CONTRIBUTING.md" "${pkgdir}/usr/share/doc/${pkgname}/CONTRIBUTING.md"
    install -Dm644 "README.md" "${pkgdir}/usr/share/doc/${pkgname}/README.md"
}
