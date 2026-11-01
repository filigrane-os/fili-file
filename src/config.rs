// Réglages de fili-file: ~/.config/filigrane/fili-file.toml (clés plates `clé = valeur`, sans dépendance).
// Les défauts vivent ici; le fichier est facultatif. L'état changé à la main (vue, colonnes) reste dans
// ~/.local/state/filigrane/ et passe devant la config.
use std::env;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub view: Option<String>,    // "list" | "grid"
    pub show_hidden: bool,
    pub sort: i32,               // index de fs::Key
    pub sort_desc: bool,
    pub columns: Option<String>, // "size,modified"
    pub start: Option<String>,   // dossier d'ouverture sans argument
}

impl Default for Config {
    fn default() -> Self {
        Config { view: None, show_hidden: false, sort: 0, sort_desc: false, columns: None, start: None }
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
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
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
            _ => {}
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_settings() {
        let c = parse("# x\nview = \"grid\"\nshow_hidden = true # oui\nsort = \"date\"\nsort_desc = true\ncolumns = \"size, kind\"\nstart = \"~/Documents\"\nbad\nview2 = 1\n");
        assert_eq!(c, Config { view: Some("grid".into()), show_hidden: true, sort: 2, sort_desc: true, columns: Some("size,kind".into()), start: Some("~/Documents".into()) });
        assert_eq!(parse("view = \"x\"\nsort = \"zz\""), Config::default());
    }
}
