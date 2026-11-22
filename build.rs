// QML interface compiled into the binary (module Fili.Files, qrc:/qt/qml/Fili/Files/qml/), Rust objects of src/bridge.rs
use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

fn main() {
    // Tokens of the shared kit (Fili.UI), for the test that checks the FiliTheme.* names used in the QML
    if let Ok(p) = std::env::var("DEP_FILI_UI_THEME_QML") {
        println!("cargo::rustc-env=FILI_UI_THEME_QML={p}");
    }
    // Version of the About card: tag or commit of the repository (absent outside a repository)
    if let Ok(o) = std::process::Command::new("git").args(["describe", "--tags", "--always", "--dirty"]).output() {
        if o.status.success() {
            println!("cargo:rustc-env=FILI_FILE_GIT={}", String::from_utf8_lossy(&o.stdout).trim());
        }
    }
    CxxQtBuilder::new_qml_module(
        QmlModule::new("Fili.Files")
            .qml_files([
                "qml/Main.qml",
                "qml/Sidebar.qml",
                "qml/NearbyView.qml",
                "qml/PathBar.qml",
                "qml/FileList.qml",
                "qml/Preview.qml",
                "qml/QuickLook.qml",
                "qml/PdfView.qml",
                "qml/PreviewButton.qml",
                "qml/LiaButton.qml",
                "qml/WindowPanel.qml",
                "qml/ContextMenu.qml",
                "qml/ViewSwitch.qml",
                "qml/BandSelect.qml",
                "qml/DebugPanel.qml",
            ])
            .qml_file(QmlFile::from("qml/DropMode.qml").singleton(true))
            .depend("QtQuick")
            .depend("Fili.UI")
            .depend("QtQuick.Controls")
            .depend("QtMultimedia")
            .depend("QtQuick.Pdf"),
    )
    // Fonts of the interface travel with the binary: the application looks the same on any distribution
    .qrc_resources(["fonts/Jost-Variable.ttf", "fonts/JetBrainsMono-Variable.ttf", "fonts/FiliIcons.ttf"])
    .qt_module("Quick")
    .files(["src/bridge.rs"])
    .build();
}
