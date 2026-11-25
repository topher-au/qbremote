#!/bin/sh
# macOS .dmg recipe (scaffold: needs Qt 6.8+ for macOS and an Apple Developer ID for signing).
set -eu
cd "$(dirname "$0")/../.."
cargo build --release -p qbremote --no-default-features
mkdir -p build/qbremote.app/Contents/MacOS
cp target/release/qbremote build/qbremote.app/Contents/MacOS/
macdeployqt build/qbremote.app -qmldir=crates/qbt-qt/qml   # embeds Qt frameworks (LGPL)
# codesign --deep --options runtime --sign "Developer ID Application: …" build/qbremote.app
hdiutil create -volname qbremote -srcfolder build/qbremote.app -ov -format UDZO build/qbremote.dmg
