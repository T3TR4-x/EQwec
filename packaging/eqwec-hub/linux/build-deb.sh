#!/usr/bin/env bash
# Builds the EQwec Hub Debian package and a relocatable tar.zst archive.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$root"

version="$(sed -n 's/^version = "\(.*\)"$/\1/p' apps/eqwec-hub/Cargo.toml | head -n 1)"
arch="$(dpkg --print-architecture)"
dist="$root/dist"
package="$dist/eqwec-hub_${version}_${arch}"

cargo build --release --locked -p eqwec-hub

rm -rf "$package"
mkdir -p "$package/DEBIAN" "$package/usr/bin" "$package/usr/share/applications"
install -m 0755 target/release/eqwec-hub "$package/usr/bin/eqwec-hub"
install -m 0644 packaging/eqwec-hub/linux/eqwec-hub.desktop \
    "$package/usr/share/applications/eqwec-hub.desktop"

cat > "$package/DEBIAN/control" <<CONTROL
Package: eqwec-hub
Version: ${version}
Section: utils
Priority: optional
Architecture: ${arch}
Maintainer: EQwec <noreply@eqwec.invalid>
Depends: libc6, libxkbcommon0, libwayland-client0, libx11-6, libxcursor1, libxrandr2, libxi6, libgl1, libegl1
Description: EQwec Hub developer-preview launcher
 Native EQwec launcher for installing, updating, rolling back,
 uninstalling, and launching signed product releases.
CONTROL

dpkg-deb --root-owner-group --build "$package" "$dist/eqwec-hub_${version}_${arch}.deb"
tar --zstd -cf "$dist/eqwec-hub_${version}_linux-${arch}.tar.zst" -C "$package" usr

echo "DEB: $dist/eqwec-hub_${version}_${arch}.deb"
echo "TAR: $dist/eqwec-hub_${version}_linux-${arch}.tar.zst"
