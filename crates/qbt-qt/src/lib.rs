//! Qt bridge: the only crate that depends on cxx-qt.
pub mod app;
pub mod columns;
pub mod model;
pub mod state;
pub mod tables;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

/// Qt Quick Controls style: the user's override, else the platform's native one
/// (`org.kde.desktop` on Linux when installed, otherwise `Fusion`).
pub fn default_style(user_override: &str) -> String {
    if !user_override.is_empty() {
        return user_override.to_owned();
    }
    if cfg!(target_os = "windows") {
        "Windows".into()
    } else if cfg!(target_os = "macos") {
        "macOS".into()
    } else if on_kde() && kde_style_installed() {
        "org.kde.desktop".into()
    } else {
        "Fusion".into()
    }
}

fn on_kde() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .is_ok_and(|d| d.split(':').any(|p| p.eq_ignore_ascii_case("KDE")))
}

/// Whether the KDE style is importable *by this process*. When the import path is overridden (an
/// AppImage or Flatpak sets `QML_IMPORT_PATH`), only those directories count: the system's copy
/// would not be found at load time and the UI would fail to start.
fn kde_style_installed() -> bool {
    let overridden = ["QML_IMPORT_PATH", "QML2_IMPORT_PATH"]
        .iter()
        .filter_map(|v| std::env::var(v).ok())
        .flat_map(|d| {
            d.split(':')
                .filter(|p| !p.is_empty())
                .map(std::path::PathBuf::from)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let candidates: Vec<std::path::PathBuf> = if overridden.is_empty() {
        [
            "/usr/lib/qt6/qml",
            "/usr/lib64/qt6/qml",
            "/usr/lib/x86_64-linux-gnu/qt6/qml",
            "/usr/lib/qt/qml",
        ]
        .iter()
        .map(Into::into)
        .collect()
    } else {
        overridden
    };
    candidates
        .iter()
        .any(|d| d.join("org/kde/desktop/qmldir").exists())
}

/// Start the application; returns the process exit code.
pub fn run() -> i32 {
    cxx_qt::init_crate!(qbt_qt);
    cxx_qt::init_qml_module!("org.qbremote");
    let mut app = QGuiApplication::new();
    if let Some(mut a) = app.as_mut() {
        a.as_mut().set_application_name(&QString::from("qbremote"));
        a.as_mut().set_organization_name(&QString::from("qbremote"));
    }
    // Translations are optional: without a catalogue the English source strings are used.
    let dir = std::env::var("QBREMOTE_I18N_DIR").unwrap_or_default();
    app::qobject::qbt_install_translator(&QString::from(dir.as_str()));
    // An explicit QT_QUICK_CONTROLS_STYLE always wins over our default.
    if std::env::var_os("QT_QUICK_CONTROLS_STYLE").is_none() {
        let style = default_style(&app::configured_style());
        cxx_qt_lib::QQuickStyle::set_style(&QString::from(style.as_str()));
    }
    let mut engine = QQmlApplicationEngine::new();
    if let Some(mut e) = engine.as_mut() {
        // A QML load failure is fatal; without this the event loop would run with no window.
        e.as_mut()
            .on_object_creation_failed(|_, url| {
                eprintln!("qbremote: failed to load {url}");
                std::process::exit(1);
            })
            .release();
        e.as_mut()
            .load(&QUrl::from("qrc:/qt/qml/org/qbremote/qml/Main.qml"));
    }
    app.as_mut().map_or(1, |a| a.exec())
}

#[cfg(test)]
mod tests {
    #[test]
    fn style_override_wins() {
        assert_eq!(super::default_style("Material"), "Material");
        assert!(!super::default_style("").is_empty());
    }
}
