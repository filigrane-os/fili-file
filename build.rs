// Interface QML compilée dans le binaire (module Fili.Files, qrc:/qt/qml/Fili/Files/qml/), objets Rust de src/bridge.rs
use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("Fili.Files")
            .qml_files([
                "qml/Main.qml",
                "qml/Sidebar.qml",
                "qml/PathBar.qml",
                "qml/FileList.qml",
                "qml/IconButton.qml",
                "qml/FocusRing.qml",
                "qml/KineticScroll.qml",
            ])
            .qml_file(QmlFile::from("qml/FiliTheme.qml").singleton(true))
            .depend("QtQuick"),
    )
    .qt_module("Quick")
    .files(["src/bridge.rs"])
    .build();
}
