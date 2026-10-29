// fili-file: explorateur de fichiers de Filigrane. Moteur en Rust (lecture des dossiers, tri, icônes, ouverture),
// interface en QML avec les tokens du fili-shell (qml/FiliTheme.qml). Usage: fili-file [dossier | fichier | file://...]
mod bridge;
mod fs;
#[allow(dead_code)]
#[path = "../../bar-stats/src/i18n.rs"]
mod i18n;
mod icons;
mod ops;
mod watch;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

fn main() {
    let mut app = QGuiApplication::new();
    // app-id Wayland (règles niri, fili-file.desktop)
    QGuiApplication::set_desktop_file_name(&QString::from("fili-file"));
    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/Fili/Files/qml/Main.qml"));
    }
    if let Some(app) = app.as_mut() {
        std::process::exit(app.exec());
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs};

    fn read(rel: &str) -> String {
        fs::read_to_string(format!("{}/{rel}", env!("CARGO_MANIFEST_DIR"))).unwrap()
    }

    // « readonly property <type> <nom>: <valeur> » -> valeur
    fn tokens(qml: &str) -> HashMap<String, String> {
        qml.lines()
            .filter_map(|l| l.trim().strip_prefix("readonly property "))
            .filter_map(|l| {
                let (decl, value) = l.split_once(':')?;
                Some((decl.split_whitespace().last()?.to_string(), value.trim().to_string()))
            })
            .collect()
    }

    // Les tokens de fili-file sont ceux du fili-shell: un token commun garde la même valeur
    #[test]
    fn same_tokens_as_the_shell() {
        let shell = tokens(&read("../../dotfiles/config/quickshell/FiliTheme.qml"));
        let ours = tokens(&read("qml/FiliTheme.qml"));
        let mut common = 0;
        // light: même rôle, autre source (Fili suit l'état de fili-theme)
        for (name, value) in ours.iter().filter(|(n, _)| *n != "light") {
            if let Some(v) = shell.get(name) {
                assert_eq!(v, value, "token {name}");
                common += 1;
            }
        }
        assert!(common >= 25, "{common} tokens en commun");
    }

    #[test]
    fn same_kinetic_scroll_as_the_shell() {
        assert_eq!(read("qml/KineticScroll.qml"), read("../../dotfiles/config/quickshell/KineticScroll.qml"));
    }
}
