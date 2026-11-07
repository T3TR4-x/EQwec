#!/usr/bin/env bash
# Builds an ad-hoc-signed EQwec Hub .app, wraps it in a DMG, mounts the DMG,
# and runs --self-check from the mounted app. Ad-hoc signing is developer
# preview only; a release needs a Developer ID identity and notarization.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$root"

version="$(sed -n 's/^version = "\(.*\)"$/\1/p' apps/eqwec-hub/Cargo.toml | head -n 1)"
short_version="${version%%-*}"
arch="$(uname -m)"
dist="$root/dist"
stage="$dist/dmg"
app="$stage/EQwec Hub.app"
dmg="$dist/EQwec-Hub-${version}-macos-${arch}.dmg"

cargo build --release --locked -p eqwec-hub

rm -rf "$stage" "$dmg"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
install -m 0755 "$root/target/release/eqwec-hub" "$app/Contents/MacOS/eqwec-hub"
ln -s /Applications "$stage/Applications"

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key><string>en</string>
    <key>CFBundleDisplayName</key><string>EQwec Hub</string>
    <key>CFBundleExecutable</key><string>eqwec-hub</string>
    <key>CFBundleIdentifier</key><string>com.eqwec.hub</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>CFBundleName</key><string>EQwec Hub</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>${short_version}</string>
    <key>CFBundleVersion</key><string>${short_version}</string>
    <key>LSMinimumSystemVersion</key><string>11.0</string>
    <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

codesign --force --sign - "$app"
codesign --verify --strict "$app"

hdiutil create -quiet -volname "EQwec Hub" -srcfolder "$stage" -ov -format UDZO "$dmg"

mount_point="$(mktemp -d)"
data_dir="$(mktemp -d)"
cleanup() {
    hdiutil detach "$mount_point" -quiet >/dev/null 2>&1 || true
    rm -rf "$mount_point" "$data_dir"
}
trap cleanup EXIT

hdiutil attach "$dmg" -quiet -nobrowse -readonly -mountpoint "$mount_point"
EQWEC_DATA_DIR="$data_dir" \
    "$mount_point/EQwec Hub.app/Contents/MacOS/eqwec-hub" --self-check

echo "DMG: $dmg"
