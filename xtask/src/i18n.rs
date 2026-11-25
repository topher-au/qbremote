use std::process::Command;

use anyhow::{Context, Result, bail};

/// Qt's tool directory: `qmake6 -query QT_INSTALL_BINS`, else whatever is on PATH.
fn tool(name: &str) -> String {
    let qmake = std::env::var("QMAKE").unwrap_or_else(|_| "qmake6".into());
    Command::new(qmake)
        .args(["-query", "QT_INSTALL_BINS"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|d| format!("{}/{name}", d.trim()))
        .filter(|p| std::path::Path::new(p).exists())
        .unwrap_or_else(|| name.to_owned())
}

pub fn run(release: bool) -> Result<()> {
    let root = crate::repo_root();
    let ts = root.join("i18n/qbremote_en.ts");
    let st = Command::new(tool("lupdate"))
        .arg(root.join("crates/qbt-qt/qml"))
        .arg("-ts")
        .arg(&ts)
        .status()
        .context("running lupdate (is Qt's bin directory installed?)")?;
    if !st.success() {
        bail!("lupdate failed");
    }
    if release {
        for entry in std::fs::read_dir(root.join("i18n"))? {
            let p = entry?.path();
            if p.extension().is_some_and(|e| e == "ts") {
                let out = p.with_extension("qm");
                let st = Command::new(tool("lrelease"))
                    .arg(&p)
                    .arg("-qm")
                    .arg(&out)
                    .status()?;
                if !st.success() {
                    bail!("lrelease failed for {}", p.display());
                }
            }
        }
    }
    Ok(())
}
