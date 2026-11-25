# Translations

Every user-visible string in the QML is wrapped in `qsTr()`. To (re)generate the source catalogue:

```sh
cargo xtask i18n          # runs lupdate over crates/qbt-qt/qml → i18n/qbremote_en.ts
```

Translate with Qt Linguist, then compile with `lrelease` (also done by `cargo xtask i18n --release`). At
startup the app loads `qbremote_<locale>.qm` from `$QBREMOTE_I18N_DIR`, `<exe dir>/translations` or the
user's data directory, falling back to English. No translations ship yet; this is the scaffolding the plan
asks for in M8.
