// Interface QML compilée dans le binaire (module Fili.Files, qrc:/qt/qml/Fili/Files/qml/), objets Rust de src/bridge.rs
use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

fn main() {
    // Version de l'à-propos: étiquette ou commit du dépôt (absent hors dépôt)
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
                "qml/PreviewButton.qml",
                "qml/IconButton.qml",
                "qml/TextButton.qml",
                "qml/ContextMenu.qml",
                "qml/ViewSwitch.qml",
                "qml/BandSelect.qml",
                "qml/FocusRing.qml",
                "qml/KineticScroll.qml",
            ])
            .qml_file(QmlFile::from("qml/FiliTheme.qml").singleton(true))
            .depend("QtQuick")
            .depend("QtQuick.Controls"),
    )
    .qt_module("Quick")
    .files(["src/bridge.rs"])
    .build();
}
