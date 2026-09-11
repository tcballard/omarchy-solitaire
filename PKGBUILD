# Build from this checkout's root as an ordinary user: makepkg -si
pkgname=omarchy-solitaire
pkgver=0.2.0
pkgrel=1
pkgdesc='Native Rust Klondike with Omarchy theme integration'
arch=('x86_64' 'aarch64')
url='https://github.com/tcballard/omarchy-solitaire'
license=('MIT' 'LicenseRef-Omarchy-Artwork')
depends=('gcc-libs' 'glibc' 'libglvnd' 'libx11' 'libxcursor' 'libxi' 'libxrandr' 'libxkbcommon' 'libxkbcommon-x11' 'wayland' 'dbus')
optdepends=('xdg-desktop-portal: special card-back file chooser'
            'xdg-desktop-portal-hyprland: file chooser integration on Omarchy')
makedepends=('rust')
source=()
sha256sums=()

build() {
  cd "$startdir"
  cargo build --release --locked
}

check() {
  cd "$startdir"
  cargo test --locked
}

package() {
  cd "$startdir"
  install -Dm755 target/release/omarchy-solitaire "$pkgdir/usr/bin/omarchy-solitaire"
  install -Dm644 packaging/io.github.tcballard.omarchy-solitaire.desktop \
    "$pkgdir/usr/share/applications/io.github.tcballard.omarchy-solitaire.desktop"
  install -Dm644 assets/solitaire.svg \
    "$pkgdir/usr/share/icons/hicolor/scalable/apps/io.github.tcballard.omarchy-solitaire.svg"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm644 NOTICE "$pkgdir/usr/share/licenses/$pkgname/NOTICE"
  install -Dm644 THIRD_PARTY.md "$pkgdir/usr/share/licenses/$pkgname/THIRD_PARTY.md"
  install -Dm644 THIRD_PARTY_LICENSES.txt "$pkgdir/usr/share/licenses/$pkgname/THIRD_PARTY_LICENSES.txt"
  install -Dm644 docs/CARD-BACKS.md "$pkgdir/usr/share/doc/$pkgname/CARD-BACKS.md"
}
