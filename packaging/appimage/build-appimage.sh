#!/bin/sh
# Builds qbremote-x86_64.AppImage without linuxdeploy: bundles the release binary, the Qt shared
# libraries it needs (LGPL: shared, replaceable), the Qt plugins and the QML modules it imports.
# Needs: Qt 6 + Rust on the build machine, and `appimagetool` on PATH.
set -eu
cd "$(dirname "$0")/../.."
ROOT=$(pwd)
APPDIR="$ROOT/build/AppDir"
QT_PLUGINS=$(qmake6 -query QT_INSTALL_PLUGINS)
QT_QML=$(qmake6 -query QT_INSTALL_QML)

cargo build --release -p qbremote --no-default-features
tools/check-qt-modules.sh target/release/qbremote

rm -rf "$APPDIR" && mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/lib" "$APPDIR/usr/plugins" "$APPDIR/usr/qml"
cp target/release/qbremote "$APPDIR/usr/bin/"

# Copy a library and everything it links, except the base system (glibc, GL/driver stack, X/Wayland
# client libs, which must come from the host).
bundle_deps() {
  ldd "$1" 2>/dev/null | awk '/=> \// {print $3}' | while read -r lib; do
    base=$(basename "$lib")
    case "$base" in
      libc.so*|libm.so*|libdl.so*|libpthread.so*|librt.so*|libutil.so*|ld-linux*|libstdc++*|libgcc_s*|libGL*|libOpenGL*|libEGL*|libGLX*|libdrm*|libgbm*|libvulkan*|libX11*|libxcb*|libwayland*|libxkbcommon*|libfontconfig*|libfreetype*|libharfbuzz*|libdbus-1*|libglib-2*|libgobject*|libgio*|libgmodule*) continue ;;
    esac
    [ -e "$APPDIR/usr/lib/$base" ] || { cp -L "$lib" "$APPDIR/usr/lib/$base"; bundle_deps "$lib"; }
  done
}
bundle_deps "$APPDIR/usr/bin/qbremote"

# Plugins: platforms, image formats, TLS/network, styles and the platform theme.
for d in platforms imageformats iconengines platformthemes xcbglintegrations wayland-decoration-client wayland-graphics-integration-client wayland-shell-integration tls networkinformation; do
  [ -d "$QT_PLUGINS/$d" ] && cp -r "$QT_PLUGINS/$d" "$APPDIR/usr/plugins/"
done
find "$APPDIR/usr/plugins" -name '*.so' | while read -r p; do bundle_deps "$p"; done

# QML modules the UI imports (Controls styles beyond Fusion/Basic are left out to keep the image small).
for m in QtQml QtQuick QtCore Qt/labs/platform Qt/labs/folderlistmodel; do
  [ -d "$QT_QML/$m" ] && mkdir -p "$APPDIR/usr/qml/$(dirname "$m")" && cp -r "$QT_QML/$m" "$APPDIR/usr/qml/$(dirname "$m")/"
done
rm -rf "$APPDIR/usr/qml/QtQuick/Controls/"{Material,Universal,Imagine,FluentWinUI3,designer} 2>/dev/null || true
find "$APPDIR/usr/qml" -name '*.so' | while read -r p; do bundle_deps "$p"; done

cat > "$APPDIR/AppRun" <<'RUN'
#!/bin/sh
HERE=$(dirname "$(readlink -f "$0")")
export LD_LIBRARY_PATH="$HERE/usr/lib:${LD_LIBRARY_PATH:-}"
export QT_PLUGIN_PATH="$HERE/usr/plugins"
export QML_IMPORT_PATH="$HERE/usr/qml"
export QML2_IMPORT_PATH="$HERE/usr/qml"
exec "$HERE/usr/bin/qbremote" "$@"
RUN
chmod +x "$APPDIR/AppRun"
cp packaging/org.qbremote.qbremote.desktop "$APPDIR/"
cp packaging/org.qbremote.qbremote.svg "$APPDIR/"
mkdir -p "$APPDIR/usr/share/metainfo" && cp packaging/org.qbremote.qbremote.metainfo.xml "$APPDIR/usr/share/metainfo/"

ARCH=x86_64 appimagetool "$APPDIR" "build/qbremote-x86_64.AppImage"
echo "built build/qbremote-x86_64.AppImage"
