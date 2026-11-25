# Windows (MSI)

Scaffold. Steps once a Windows build host is available:

1. `cargo build --release -p qbremote --no-default-features` with Qt 6.8+ (MSVC) on PATH.
2. `windeployqt --qmldir crates/qbt-qt/qml --release target\release\qbremote.exe` into a staging folder
   (Qt stays as DLLs: LGPL).
3. Package the folder with WiX (`candle`/`light` or `wix build`) into `qbremote.msi`; sign with `signtool`.
