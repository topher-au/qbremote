#!/bin/sh
# Licensing gate (plan §3): the shipped binary may link only LGPL-licensed Qt modules.
# Usage: tools/check-qt-modules.sh path/to/qbremote
# Fails if the binary links a Qt module from the deny-list (GPL/commercial-only add-ons, or Qt Test,
# which belongs only in `test-hooks` development builds).
set -eu
bin="${1:?usage: check-qt-modules.sh BINARY}"
deny='Qt6(Graphs|GraphsWidgets|Charts|ChartsQml|DataVisualization|VirtualKeyboard|Quick3D|WebEngine[A-Za-z]*|Test|Pdf|PdfQuick|HttpServer|Quick3DPhysics|CoAp|Mqtt|OpcUa|Scxml)\b'
if ! command -v ldd >/dev/null 2>&1; then
  echo "ldd not available on this platform; run the equivalent (otool -L / dumpbin) in CI." >&2
  exit 0
fi
linked=$(ldd "$bin" | awk '/libQt6/ {print $1}' | sed 's/^lib//; s/\.so.*//' | sort -u)
echo "linked Qt modules:" $linked
if printf '%s\n' $linked | grep -Eq "^$deny$"; then
  echo "ERROR: binary links a module that is not in the allowed LGPL set:" >&2
  printf '%s\n' $linked | grep -E "^$deny$" >&2
  exit 1
fi
echo "ok: only LGPL modules linked"
