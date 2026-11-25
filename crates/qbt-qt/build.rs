use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    // Every file in qml/ belongs to the `org.qbremote` module, so adding a component needs no
    // build-script edit. Rebuild when the directory's contents change.
    println!("cargo:rerun-if-changed=qml");
    println!("cargo:rerun-if-changed=cpp");
    let mut qml: Vec<String> = std::fs::read_dir("qml")
        .expect("qml directory")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "qml"))
        .map(|p| format!("qml/{}", p.file_name().unwrap().to_string_lossy()))
        .collect();
    qml.sort();

    // `test-hooks` links Qt Test (fatal-mode model testers for the UI self-test). Release builds
    // leave it off so only the LGPL module set from the plan is linked.
    let test_hooks = std::env::var_os("CARGO_FEATURE_TEST_HOOKS").is_some();

    // Linking: cxx-qt does not link with GNU ld.bfd; the workspace .cargo/config.toml passes -fuse-ld=mold.
    let mut builder = CxxQtBuilder::new_qml_module(QmlModule::new("org.qbremote").qml_files(qml))
        .qt_module("Gui")
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .files(["src/model.rs", "src/app.rs"])
        .include_dir("cpp")
        .cpp_file("cpp/shim.cpp");
    if test_hooks {
        builder = builder.qt_module("Test");
        // SAFETY: only adds a preprocessor define; no flags that change ABI or linking.
        builder = unsafe {
            builder.cc_builder(|cc| {
                cc.define("QBT_TEST_HOOKS", None);
            })
        };
    }
    builder.build();
}
