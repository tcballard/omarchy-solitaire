# Build from the checkout root with: makepkg -s
pkgname=omarchy-solitaire
pkgver=0.1.0
pkgrel=1
pkgdesc='Native Klondike solitaire with Omarchy theme integration'
arch=('any')
url='https://github.com/tcballard/omarchy-solitaire'
license=('MIT' 'LicenseRef-Omarchy-Artwork')
depends=('python' 'pyside6' 'qt6-declarative' 'qt6-svg' 'qt6-wayland' 'ttf-dejavu')
makedepends=('python-build' 'python-installer' 'python-setuptools' 'python-wheel')
checkdepends=()
source=()
sha256sums=()

build() {
  cd "$startdir"
  python -m build --wheel --no-isolation
}

check() {
  cd "$startdir"
  QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software python -m unittest discover -s tests -v
}

package() {
  cd "$startdir"
  python -m installer --destdir="$pkgdir" dist/*.whl
  install -Dm644 packaging/io.github.tcballard.omarchy-solitaire.desktop \
    "$pkgdir/usr/share/applications/io.github.tcballard.omarchy-solitaire.desktop"
  install -Dm644 omarchy_solitaire/assets/solitaire.svg \
    "$pkgdir/usr/share/icons/hicolor/scalable/apps/io.github.tcballard.omarchy-solitaire.svg"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm644 NOTICE "$pkgdir/usr/share/licenses/$pkgname/NOTICE"
  install -Dm644 docs/CARD-BACKS.md "$pkgdir/usr/share/doc/$pkgname/CARD-BACKS.md"
}
