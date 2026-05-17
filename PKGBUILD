pkgname=wsl-open
pkgver=0.1.0
pkgrel=1
pkgdesc="Open URLs, domains and files with Windows default apps from WSL"
arch=('x86_64')
url="https://github.com/lonerOrz/wsl-open"
license=('MIT')
makedepends=('rust')
source=("$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$pkgname-$pkgver"
  cargo build --release
}

package() {
  install -Dm755 "$pkgname-$pkgver/target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
}
