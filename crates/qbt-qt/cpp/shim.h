#pragma once
#include <QtCore/QString>

// Save the first Qt Quick window to `path` (PNG). Used by the headless screenshot test hook.
bool qbt_grab_window(const QString& path);

// Install `qbremote_<locale>.qm` if one is found in `extra_dir`, `<exe dir>/translations` or the
// user's data directory. Returns whether a translator was installed (English is the fallback).
bool qbt_install_translator(const QString& extra_dir);

// Attach a fatal-mode QAbstractItemModelTester to every item model reachable from the Qt Quick
// windows. Returns how many testers were attached (models already tested are skipped).
int qbt_attach_model_testers();
