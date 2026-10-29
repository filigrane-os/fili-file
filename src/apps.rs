// Applications qui ouvrent un type de fichier (« Ouvrir avec »): fichiers .desktop, mimeapps.list (défaut et
// associations ajoutées), mimeinfo.cache. Lancement par l'interprétation du champ Exec. Sans Qt: testé par cargo test.
use std::{
    collections::HashMap,
    env, fs,
    path::PathBuf,
};

#[derive(Clone, Debug, PartialEq)]
pub struct App {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub terminal: bool,
    pub hidden: bool,
}

fn home() -> String {
    env::var("HOME").unwrap_or_else(|_| "/".into())
}

fn app_dirs() -> Vec<PathBuf> {
    let data = env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{}/.local/share", home()));
    let mut v = vec![PathBuf::from(data).join("applications")];
    v.push(PathBuf::from("/usr/local/share/applications"));
    v.push(PathBuf::from("/usr/share/applications"));
    v
}

pub fn parse_desktop(id: &str, text: &str) -> Option<App> {
    let mut section = "";
    let (mut name, mut exec, mut terminal, mut hidden) = (None, None, false, false);
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            section = line;
            continue;
        }
        if section != "[Desktop Entry]" {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else { continue };
        match k {
            "Name" => name = Some(v.to_string()),
            "Exec" => exec = Some(v.to_string()),
            "Terminal" => terminal = v == "true",
            "NoDisplay" | "Hidden" => hidden |= v == "true",
            "Type" if v != "Application" => return None,
            _ => {}
        }
    }
    Some(App { id: id.to_string(), name: name?, exec: exec?, terminal, hidden })
}

pub fn desktop(id: &str) -> Option<App> {
    app_dirs().iter().find_map(|d| parse_desktop(id, &fs::read_to_string(d.join(id)).ok()?))
}

// Lignes « type=a.desktop;b.desktop » d'un fichier à sections, pour la section demandée
fn section_map(text: &str, wanted: &str) -> HashMap<String, Vec<String>> {
    let mut out = HashMap::new();
    let mut in_section = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_section = line == wanted;
        } else if in_section && let Some((mime, ids)) = line.split_once('=') {
            out.insert(mime.to_string(), ids.split(';').filter(|s| !s.is_empty()).map(String::from).collect());
        }
    }
    out
}

// Applications pour un type MIME: celle par défaut d'abord, puis les associations ajoutées, puis le cache
pub fn for_mime(mime: &str) -> Vec<App> {
    let config = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home()));
    let list = fs::read_to_string(format!("{config}/mimeapps.list")).unwrap_or_default();
    let mut ids: Vec<String> = Vec::new();
    for wanted in ["[Default Applications]", "[Added Associations]"] {
        ids.extend(section_map(&list, wanted).remove(mime).unwrap_or_default());
    }
    for d in app_dirs() {
        let cache = fs::read_to_string(d.join("mimeinfo.cache")).unwrap_or_default();
        ids.extend(section_map(&cache, "[MIME Cache]").remove(mime).unwrap_or_default());
    }
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for (i, id) in ids.into_iter().enumerate() {
        if seen.contains(&id) {
            continue;
        }
        seen.push(id.clone());
        // Les entrées cachées (NoDisplay) ne sont proposées que si l'utilisateur les a choisies par défaut
        if let Some(app) = desktop(&id).filter(|a| !a.hidden || i == 0) {
            out.push(app);
        }
    }
    out
}

// Découpe un champ Exec (guillemets doubles, antislash) en arguments, codes %f %F %u %U remplacés par path
pub fn command(app: &App, path: &str) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    let (mut cur, mut quoted, mut any) = (String::new(), false, false);
    let mut chars = app.exec.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                quoted = !quoted;
                any = true;
            }
            '\\' if quoted => {
                if let Some(n) = chars.next() {
                    cur.push(n);
                }
            }
            ' ' if !quoted => {
                if any || !cur.is_empty() {
                    args.push(std::mem::take(&mut cur));
                }
                any = false;
            }
            c => cur.push(c),
        }
    }
    if any || !cur.is_empty() {
        args.push(cur);
    }
    let mut out: Vec<String> = args
        .into_iter()
        .map(|a| {
            let mut s = String::new();
            let mut it = a.chars();
            while let Some(c) = it.next() {
                if c != '%' {
                    s.push(c);
                    continue;
                }
                match it.next() {
                    Some('f' | 'F' | 'u' | 'U') => s.push_str(path),
                    Some('%') => s.push('%'),
                    _ => {}
                }
            }
            s
        })
        .filter(|a| !a.is_empty())
        .collect();
    if app.terminal {
        out.splice(0..0, ["foot".to_string(), "-e".to_string()]);
    }
    out
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_desktop_file() {
        let text = "[Desktop Entry]\nType=Application\nName=Visionneuse\nExec=imv %F\nNoDisplay=true\n[Desktop Action x]\nName=Autre\n";
        let a = parse_desktop("imv.desktop", text).unwrap();
        assert_eq!((a.name.as_str(), a.exec.as_str(), a.hidden, a.terminal), ("Visionneuse", "imv %F", true, false));
        assert!(parse_desktop("x.desktop", "[Desktop Entry]\nType=Link\nName=x\nExec=y\n").is_none());
    }

    #[test]
    fn exec_codes_and_quotes() {
        let app = |exec: &str, terminal| App { id: "a".into(), name: "a".into(), exec: exec.into(), terminal, hidden: false };
        assert_eq!(command(&app("viewer --file=%f %i", false), "/tmp/a b.png"), ["viewer", "--file=/tmp/a b.png"]);
        assert_eq!(command(&app("\"/opt/my app/run\" --x %U 100%%", false), "/p"), ["/opt/my app/run", "--x", "/p", "100%"]);
        assert_eq!(command(&app("vim %F", true), "/p"), ["foot", "-e", "vim", "/p"]);
    }

    #[test]
    fn sections() {
        let m = section_map("[Default Applications]\nimage/png=imv.desktop;b.desktop;\n[Added Associations]\nimage/png=c.desktop;\n", "[Default Applications]");
        assert_eq!(m["image/png"], ["imv.desktop", "b.desktop"]);
    }
}
