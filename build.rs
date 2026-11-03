// QML interface compiled into the binary (module Fili.Files, qrc:/qt/qml/Fili/Files/qml/), Rust objects of src/bridge.rs
use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

fn main() {
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
                "qml/PreviewButton.qml",
                "qml/IconButton.qml",
                "qml/TextButton.qml",
                "qml/ContextMenu.qml",
                "qml/ViewSwitch.qml",
                "qml/BandSelect.qml",
                "qml/FocusRing.qml",
                "qml/FiliScrollBar.qml",
                "qml/KineticScroll.qml",
            ])
            .qml_file(QmlFile::from("qml/FiliTheme.qml").singleton(true))
            .depend("QtQuick")
            .depend("QtQuick.Controls"),
    )
    // Fonts of the interface travel with the binary: the application looks the same on any distribution
    .qrc_resources(["fonts/Jost-Variable.ttf", "fonts/JetBrainsMono-Variable.ttf", "fonts/FiliIcons.ttf"])
    .qt_module("Quick")
    .files(["src/bridge.rs"])
    .build();
}
