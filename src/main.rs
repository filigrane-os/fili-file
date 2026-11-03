// fili-file: explorateur de fichiers de Filigrane. Moteur en Rust (lecture des dossiers, tri, icônes, ouverture),
// interface en QML avec les tokens du fili-shell (qml/FiliTheme.qml). Usage: fili-file [dossier | fichier | file://...]
mod about;
mod appearance;
mod apps;
mod bridge;
mod config;
mod context;
mod fs;
mod icon_theme;
#[allow(dead_code)]
#[path = "../../bar-stats/src/i18n.rs"]
mod i18n;
mod icons;
mod ops;
mod pick;
mod portal;
mod tools;
mod visits;
mod volumes;
mod watch;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

fn main() {
    // Commande à une instance ouverte (assistant): fili-file ctl goto|show ...
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "ctl") {
        match context::ctl(&args[2..]) {
            Ok(answer) => println!("{answer}"),
            Err(e) => {
                eprintln!("fili-file ctl: {e}");
                std::process::exit(1);
            }
        }
        return;
    }
    // Service du portail FileChooser: sans interface, lancé par D-Bus (voir portal.rs)
    if std::env::args().any(|a| a == "--portal") {
        if let Err(e) = portal::serve() {
            eprintln!("fili-file --portal: {e}");
            std::process::exit(1);
        }
        return;
    }
    let mut app = QGuiApplication::new();
    // app-id Wayland (règles niri, fili-file.desktop)
    QGuiApplication::set_desktop_file_name(&QString::from(if pick::get().is_some() { "fili-file-picker" } else { "fili-file" }));
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

    // « [section] clé = "valeur" # c » de theme.toml -> (section, clé) = valeur
    fn defaults() -> HashMap<(String, String), String> {
        let mut out = HashMap::new();
        let mut section = String::new();
        for line in read("theme.toml").lines() {
            let line = line.split(" #").next().unwrap_or("").trim();
            if let Some(n) = line.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
                section = n.to_string();
            } else if let Some((k, v)) = line.split_once('=') {
                out.insert((section.clone(), k.trim().to_string()), v.trim().trim_matches('"').to_string());
            }
        }
        out
    }

    // Les valeurs par défaut de theme.toml sont celles du fili-shell: un jeton commun garde la même valeur
    #[test]
    fn default_theme_matches_the_shell() {
        let shell = tokens(&read("../../dotfiles/config/quickshell/FiliTheme.qml"));
        let d = defaults();
        let mut common = 0;
        // (jeton du shell, clé de theme.toml): couleurs par mode (« light ? clair : sombre »), le reste commun
        let colors = [
            ("nuit", "background"), ("nuitProfond", "background_deep"), ("nuit2", "surface"), ("nuit3", "surface_hover"),
            ("nuit4", "surface_active"), ("ardoise", "muted"), ("brume2", "text_secondary"), ("brume", "text"),
            ("sable", "warning"), ("cendre", "danger"), ("sauge", "success"), ("border", "border"),
        ];
        for (token, key) in colors {
            let v = &shell[token];
            assert!(v.starts_with(&format!("ov(\"{key}\", ")), "{token}: ov(\"{key}\", ...)");
            let hexes: Vec<&str> = v.split('"').skip(1).step_by(2).filter(|h| h.starts_with('#')).collect();
            let (light, dark) = if hexes.len() == 2 { (hexes[0], hexes[1]) } else { (hexes[0], hexes[0]) };
            assert_eq!(d[&("theme.light".to_string(), key.to_string())], light, "{token} clair");
            assert_eq!(d[&("theme.dark".to_string(), key.to_string())], dark, "{token} sombre");
            common += 1;
        }
        let plain = [
            ("font", "font"), ("mono", "mono"), ("size", "size"), ("sizeList", "size_list"), ("sizeSmall", "size_small"),
            ("sizeLabel", "size_label"), ("sizeTitle", "size_title"), ("weightAccent", "weight_accent"),
            ("radiusMark", "radius_mark"), ("radiusControl", "radius_control"), ("radiusWindow", "radius_window"),
            ("space1", "space_1"), ("space2", "space_2"), ("space3", "space_3"), ("space4", "space_4"),
            ("space5", "space_5"), ("space6", "space_6"), ("borderWidth", "border_width"),
        ];
        for (token, key) in plain {
            let v = &shell[token];
            assert!(v.starts_with(&format!("ov(\"{key}\", ")), "{token}: ov(\"{key}\", ...)");
            let v = v.split_once(", ").unwrap().1.trim_end_matches(')');
            let want = match v {
                "Font.DemiBold" => "600".to_string(),
                v => v.trim_matches('"').to_string(),
            };
            assert_eq!(d[&("theme".to_string(), key.to_string())], want, "token {token}");
            common += 1;
        }
        // Teintes d'accentuation: chacune de theme.toml figure telle quelle dans le shell
        let tones = &shell["accentTones"];
        let toml = read("theme.toml");
        let mut n = 0;
        for line in toml.lines().skip_while(|l| !l.starts_with("[theme.accents]")).skip(1) {
            let Some((name, list)) = line.split_once('=') else { continue };
            assert!(tones.contains(&format!("{}: {}", name.trim(), list.trim())), "accent {}", name.trim());
            n += 1;
        }
        assert!(n >= 7);
        assert!(common >= 25, "{common} tokens en commun");
    }

    #[test]
    fn same_kinetic_scroll_as_the_shell() {
        assert_eq!(read("qml/KineticScroll.qml"), read("../../dotfiles/config/quickshell/KineticScroll.qml"));
    }
}
