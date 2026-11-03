// Réglages de fili-file: ~/.config/filigrane/fili-file.toml (clés plates `clé = valeur`, sans dépendance).
// Les défauts vivent ici; le fichier est facultatif. L'état changé à la main (vue, colonnes) reste dans
// ~/.local/state/filigrane/ et passe devant la config.
use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub view: Option<String>,    // "list" | "grid"
    pub show_hidden: bool,
    pub sort: i32,               // index de fs::Key
    pub sort_desc: bool,
    pub columns: Option<String>, // "size,modified"
    pub start: Option<String>,   // folder opened when started without an argument
    pub icon_theme: Option<String>, // freedesktop icon theme; default: the desktop's (see icon_theme.rs)
    pub folder_icon: Option<String>, // icon name for every folder; default: the theme's X-Fili-FolderIcon, else "folder"
    pub file_icon: Option<String>,   // icon name for every regular file; default: the theme's X-Fili-FileIcon, else by type
}

impl Default for Config {
    fn default() -> Self {
        Config { view: None, show_hidden: false, sort: 0, sort_desc: false, columns: None, start: None, icon_theme: None, folder_icon: None, file_icon: None }
    }
}

pub fn path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_default();
    let dir = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{home}/.config"));
    PathBuf::from(dir).join("filigrane/fili-file.toml")
}

pub fn load() -> Config {
    parse(&std::fs::read_to_string(path()).unwrap_or_default())
}

pub fn parse(text: &str) -> Config {
    let mut c = Config::default();
    let mut in_section = false;
    for line in text.lines() {
        let line = strip_comment(line).trim();
        if line.starts_with('[') && !line.contains('=') {
            in_section = true;
            continue;
        }
        if in_section {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else { continue };
        let v = v.trim().trim_matches('"').trim();
        match k.trim() {
            "view" if v == "list" || v == "grid" => c.view = Some(v.into()),
            "show_hidden" => c.show_hidden = v == "true",
            "sort" => {
                c.sort = ["name", "size", "date", "kind", "created", "permissions", "owner"].iter().position(|n| *n == v).unwrap_or(0) as i32
            }
            "sort_desc" => c.sort_desc = v == "true",
            "columns" if !v.is_empty() => c.columns = Some(v.replace(' ', "")),
            "start" if !v.is_empty() => c.start = Some(v.into()),
            "icon_theme" if !v.is_empty() => c.icon_theme = Some(v.into()),
            "folder_icon" if !v.is_empty() => c.folder_icon = Some(v.into()),
            "file_icon" if !v.is_empty() => c.file_icon = Some(v.into()),
            _ => {}
        }
    }
    c
}

// Coupe au premier « # » hors guillemets (les couleurs « "#1c222b" » en contiennent)
fn strip_comment(line: &str) -> &str {
    let mut quoted = false;
    for (i, ch) in line.char_indices() {
        match ch {
            '"' => quoted = !quoted,
            '#' if !quoted => return &line[..i],
            _ => {}
        }
    }
    line
}

// ---- Thème --------------------------------------------------------------------------------------------------
// Les défauts sont dans theme.toml (embarqué); ~/.config/filigrane/fili-file.toml les surcharge clé par clé.
// Ordre de recherche d'une clé: [theme.<mode>] de l'utilisateur, son [theme], [theme.<mode>] des défauts, leur [theme].

#[derive(Clone, Debug, PartialEq)]
enum Val {
    Str(String),
    List(Vec<String>),
}

type Sections = BTreeMap<String, BTreeMap<String, Val>>;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Color,
    Int,
    Str,
}

// (clé TOML, type): toutes valent dans [theme] et dans chaque mode
const KEYS: &[(&str, Kind)] = &[
    ("background", Kind::Color),
    ("background_deep", Kind::Color),
    ("surface", Kind::Color),
    ("surface_hover", Kind::Color),
    ("surface_active", Kind::Color),
    ("border", Kind::Color),
    ("muted", Kind::Color),
    ("text_secondary", Kind::Color),
    ("text", Kind::Color),
    ("warning", Kind::Color),
    ("danger", Kind::Color),
    ("success", Kind::Color),
    ("accent", Kind::Color),        // facultatif: sans valeur, l'accent suit fili-theme
    ("accent_bright", Kind::Color), // facultatif
    ("font", Kind::Str),
    ("mono", Kind::Str),
    ("icons", Kind::Str),
    ("size", Kind::Int),
    ("size_list", Kind::Int),
    ("size_small", Kind::Int),
    ("size_label", Kind::Int),
    ("size_title", Kind::Int),
    ("weight_accent", Kind::Int),
    ("radius_mark", Kind::Int),
    ("radius_control", Kind::Int),
    ("radius_window", Kind::Int),
    ("border_width", Kind::Int),
    ("space_1", Kind::Int),
    ("space_2", Kind::Int),
    ("space_3", Kind::Int),
    ("space_4", Kind::Int),
    ("space_5", Kind::Int),
    ("space_6", Kind::Int),
    ("row_height", Kind::Int),
    ("side_row_height", Kind::Int),
    ("icon_size", Kind::Int),
    ("margin", Kind::Int),
    ("toolbar_height", Kind::Int),
    ("status_height", Kind::Int),
    ("sidebar_width", Kind::Int),
];

const DEFAULT_THEME: &str = include_str!("../theme.toml");

fn parse_value(v: &str) -> Val {
    let v = v.trim();
    if let Some(inner) = v.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        Val::List(inner.split(',').map(|x| x.trim().trim_matches('"').trim().to_string()).filter(|x| !x.is_empty()).collect())
    } else {
        Val::Str(v.trim_matches('"').trim().to_string())
    }
}

fn parse_sections(text: &str) -> Sections {
    let mut out = Sections::new();
    let mut section = String::new();
    for line in text.lines() {
        let line = strip_comment(line).trim();
        if let Some(name) = line.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
            section = name.trim().to_string();
        } else if let Some((k, v)) = line.split_once('=') {
            out.entry(section.clone()).or_default().insert(k.trim().to_string(), parse_value(v));
        }
    }
    out
}

fn valid_color(s: &str) -> bool {
    s.strip_prefix('#').is_some_and(|h| matches!(h.len(), 3 | 6 | 8) && h.chars().all(|c| c.is_ascii_hexdigit()))
}

fn valid(kind: Kind, s: &str) -> bool {
    match kind {
        Kind::Color => valid_color(s),
        Kind::Int => s.parse::<u32>().is_ok(),
        Kind::Str => !s.is_empty(),
    }
}

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push(' '),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

// Teintes d'accentuation: celles de l'utilisateur (4 couleurs valides) remplacent celles des défauts, par nom
fn accents_from(layers: &[&Sections]) -> BTreeMap<String, Vec<String>> {
    let mut accents = BTreeMap::new();
    for sections in layers {
        for (name, v) in sections.get("theme.accents").into_iter().flatten() {
            if let Val::List(l) = v {
                if l.len() == 4 && l.iter().all(|c| valid_color(c)) {
                    accents.insert(name.clone(), l.clone());
                }
            }
        }
    }
    accents
}

// Teinte [sombre, sombre vif, clair, clair vif] d'un nom d'accent (lue une fois), `default` si le nom est inconnu
pub fn accent_tone(name: &str) -> Vec<String> {
    static TONES: std::sync::OnceLock<BTreeMap<String, Vec<String>>> = std::sync::OnceLock::new();
    let tones = TONES.get_or_init(|| {
        let read = |p: PathBuf| parse_sections(&std::fs::read_to_string(p).unwrap_or_default());
        accents_from(&[&parse_sections(DEFAULT_THEME), &read(global_path()), &read(path())])
    });
    tones.get(name).or_else(|| tones.get("default")).cloned().unwrap_or_default()
}

// Thème fusionné en JSON: {"dark": {clé: valeur}, "light": {...}, "accents": {nom: [4 couleurs]}}
#[cfg(test)]
pub fn theme_json_from(user: &str) -> String {
    theme_json_layers("", user)
}

// `global`: ~/.config/filigrane/theme.toml (commun à Filigrane); `user`: fili-file.toml, qui passe devant
pub fn theme_json_layers(global: &str, user: &str) -> String {
    let defaults = parse_sections(DEFAULT_THEME);
    let global = parse_sections(global);
    let user = parse_sections(user);
    let get = |sections: &Sections, name: &str, key: &str| match sections.get(name).and_then(|s| s.get(key)) {
        Some(Val::Str(v)) => Some(v.clone()),
        _ => None,
    };
    let mut out = String::from("{");
    for (n, mode) in ["dark", "light"].iter().enumerate() {
        if n > 0 {
            out.push(',');
        }
        out.push_str(&format!("{}:{{", json_str(mode)));
        let section = format!("theme.{mode}");
        let mut first = true;
        for (key, kind) in KEYS {
            let found = [(&user, section.as_str()), (&user, "theme"), (&global, section.as_str()), (&global, "theme"), (&defaults, section.as_str()), (&defaults, "theme")]
                .iter()
                .filter_map(|(s, name)| get(s, name, key))
                .find(|v| valid(*kind, v));
            let Some(v) = found else { continue };
            if !first {
                out.push(',');
            }
            first = false;
            out.push_str(&json_str(key));
            out.push(':');
            out.push_str(&if *kind == Kind::Int { v } else { json_str(&v) });
        }
        out.push('}');
    }
    let accents = accents_from(&[&defaults, &global, &user]);
    let list: Vec<String> =
        accents.iter().map(|(n, l)| format!("{}:[{}]", json_str(n), l.iter().map(|c| json_str(c)).collect::<Vec<_>>().join(","))).collect();
    out.push_str(&format!(",\"accents\":{{{}}}}}", list.join(",")));
    out
}

pub fn global_path() -> PathBuf {
    path().with_file_name("theme.toml")
}

pub fn theme_json() -> String {
    let read = |p: PathBuf| std::fs::read_to_string(p).unwrap_or_default();
    theme_json_layers(&read(global_path()), &read(path()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_settings() {
        let c = parse("# x\nview = \"grid\"\nshow_hidden = true # yes\nsort = \"date\"\nsort_desc = true\ncolumns = \"size, kind\"\nstart = \"~/Documents\"\nicon_theme = \"Papirus\"\nfolder_icon = \"folder-blue\"\nfile_icon = \"text-x-generic\"\nbad\nview2 = 1\n");
        assert_eq!(
            c,
            Config {
                view: Some("grid".into()),
                show_hidden: true,
                sort: 2,
                sort_desc: true,
                columns: Some("size,kind".into()),
                start: Some("~/Documents".into()),
                icon_theme: Some("Papirus".into()),
                folder_icon: Some("folder-blue".into()),
                file_icon: Some("text-x-generic".into()),
            }
        );
        assert_eq!(parse("view = \"x\"\nsort = \"zz\""), Config::default());
    }

    #[test]
    fn theme_keys_do_not_leak_into_settings() {
        assert_eq!(parse("[theme]\nview = \"grid\"\n[theme.dark]\nstart = \"/x\"\n"), Config::default());
        assert_eq!(parse("view = \"grid\"\n[theme]\nsize = 14\n").view, Some("grid".into()));
    }

    fn val(json: &str, mode: &str, key: &str) -> Option<String> {
        let at = json.find(&format!("\"{mode}\":{{"))?;
        let rest = &json[at..json[at..].find('}')? + at];
        let k = rest.find(&format!("\"{key}\":"))? + key.len() + 3;
        let v = &rest[k..];
        Some(v[..v.find(',').unwrap_or(v.len())].trim_matches('"').to_string())
    }

    #[test]
    fn global_theme_sits_between_app_and_defaults() {
        let j = theme_json_layers("[theme]\nradius_control = 8\nsize = 15\n[theme.dark]\nbackground = \"#000000\"\n", "[theme]\nsize = 14\n");
        assert_eq!(val(&j, "dark", "size").as_deref(), Some("14")); // l'app passe devant le global
        assert_eq!(val(&j, "dark", "radius_control").as_deref(), Some("8")); // le global passe devant les défauts
        assert_eq!(val(&j, "dark", "background").as_deref(), Some("#000000"));
        assert_eq!(val(&j, "light", "background").as_deref(), Some("#f3f1ec"));
    }

    #[test]
    fn theme_defaults_and_overrides() {
        let d = theme_json_from("");
        assert_eq!(val(&d, "dark", "background").as_deref(), Some("#1c222b"));
        assert_eq!(val(&d, "light", "background").as_deref(), Some("#f3f1ec"));
        assert_eq!(val(&d, "dark", "radius_control").as_deref(), Some("6"));
        assert_eq!(val(&d, "dark", "accent"), None);
        // [theme] vaut pour les deux modes, un mode passe devant; valeurs invalides ignorées; # dans les guillemets
        let u = theme_json_from("[theme]\nradius_control = 9\nsize = big\ntext = \"#fff\" # c\n[theme.light]\nradius_control = 3\nbackground = \"nope\"\naccent = \"#336699\"\n[theme.accents]\nsauge = [\"#111111\", \"#222222\", \"#333333\", \"#444444\"]\nbad = [\"#111\"]\n");
        assert_eq!(val(&u, "dark", "radius_control").as_deref(), Some("9"));
        assert_eq!(val(&u, "light", "radius_control").as_deref(), Some("3"));
        assert_eq!(val(&u, "dark", "size").as_deref(), Some("13"));
        assert_eq!(val(&u, "dark", "text").as_deref(), Some("#fff"));
        assert_eq!(val(&u, "light", "background").as_deref(), Some("#f3f1ec"));
        assert_eq!(val(&u, "light", "accent").as_deref(), Some("#336699"));
        assert!(u.contains("\"sauge\":[\"#111111\""));
        assert!(!u.contains("\"bad\""));
        assert!(u.contains("\"voile\":[\"#9dbbd6\""));
    }
}
