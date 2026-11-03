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

    // A file of the fili-shell (directory FILI_SHELL_DIR). The parity tests below compare the application with the
    // shell: they run from the Filigrane repository (tests/fili-shell/run.sh sets the variable) and are skipped elsewhere.
    fn shell(file: &str) -> Option<String> {
        let dir = std::env::var_os("FILI_SHELL_DIR")?;
        Some(fs::read_to_string(std::path::Path::new(&dir).join(file)).unwrap())
    }

    // "readonly property <type> <name>: <value>" -> value
    fn tokens(qml: &str) -> HashMap<String, String> {
        qml.lines()
            .filter_map(|l| l.trim().strip_prefix("readonly property "))
            .filter_map(|l| {
                let (decl, value) = l.split_once(':')?;
                Some((decl.split_whitespace().last()?.to_string(), value.trim().to_string()))
            })
            .collect()
    }

    // "[section] key = "value" # comment" of theme.toml -> (section, key) = value
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

    // The defaults of theme.toml are the ones of the fili-shell: a shared token keeps the same value
    #[test]
    fn shell_parity_default_theme() {
        let Some(shell_qml) = shell("FiliTheme.qml") else { return };
        let shell = tokens(&shell_qml);
        let d = defaults();
        let mut common = 0;
        // (shell token, theme.toml key): colors per mode ("light ? light : dark"), the rest shared
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
            assert_eq!(d[&("theme.light".to_string(), key.to_string())], light, "{token} light");
            assert_eq!(d[&("theme.dark".to_string(), key.to_string())], dark, "{token} dark");
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
        // Accent tones: each one of theme.toml appears as it is in the shell
        let tones = &shell["accentTones"];
        let toml = read("theme.toml");
        let mut n = 0;
        for line in toml.lines().skip_while(|l| !l.starts_with("[theme.accents]")).skip(1) {
            let Some((name, list)) = line.split_once('=') else { continue };
            assert!(tones.contains(&format!("{}: {}", name.trim(), list.trim())), "accent {}", name.trim());
            n += 1;
        }
        assert!(n >= 7);
        assert!(common >= 25, "{common} shared tokens");
    }

    // A misspelled token does not fail at run time: the item just stays transparent
    #[test]
    fn every_theme_token_used_in_qml_is_defined() {
        let defined: Vec<String> = read("qml/FiliTheme.qml")
            .lines()
            .filter_map(|l| l.trim().strip_prefix("readonly property "))
            .filter_map(|l| Some(l.split(':').next()?.split_whitespace().last()?.to_string()))
            .chain(["alpha".to_string()])
            .collect();
        for entry in fs::read_dir(format!("{}/qml", env!("CARGO_MANIFEST_DIR"))).unwrap() {
            let path = entry.unwrap().path();
            if path.file_name().is_some_and(|n| n == "FiliTheme.qml") {
                continue;
            }
            let text = fs::read_to_string(&path).unwrap();
            for (i, _) in text.match_indices("FiliTheme.") {
                let name: String = text[i + 10..].chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                assert!(defined.contains(&name), "{}: FiliTheme.{name} is not defined", path.display());
            }
        }
    }

    #[test]
    fn shell_parity_kinetic_scroll() {
        let Some(shell_qml) = shell("KineticScroll.qml") else { return };
        assert_eq!(read("qml/KineticScroll.qml"), shell_qml);
    }
}
