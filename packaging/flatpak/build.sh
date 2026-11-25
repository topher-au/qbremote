#!/bin/sh
# Build and install the Flatpak for the current user.
# Needs flatpak-builder and the KDE 6.10 runtime/SDK + rust-stable extension (flatpak --user install flathub
# org.kde.Platform//6.10 org.kde.Sdk//6.10 org.freedesktop.Sdk.Extension.rust-stable//25.08).
set -eu
cd "$(dirname "$0")/../.."
SRC=build/flatpak-src
rm -rf "$SRC" && mkdir -p "$SRC"
# Source tree without build output; crates vendored because the sandbox has no network.
tar --exclude=./target --exclude=./build --exclude=./.git -cf - . | tar -xf - -C "$SRC"
mkdir -p "$SRC/.cargo"
cargo vendor --locked "$SRC/vendor" > "$SRC/.cargo/vendor-config.toml"
# cxx-qt does not link with GNU ld.bfd: use the SDK's lld, and the vendored sources.
cat > "$SRC/.cargo/config.toml" <<'CFG'
[source.crates-io]
replace-with = "vendored-sources"
[source.vendored-sources]
directory = "vendor"
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
CFG
flatpak-builder --user --force-clean --install --disable-rofiles-fuse build/flatpak-build packaging/flatpak/org.qbremote.qbremote.yml
echo "installed: flatpak run org.qbremote.qbremote"
