// Applications that open a file type ("Open with"): .desktop files, mimeapps.list (default and
// added associations), mimeinfo.cache. Launched by interpreting the Exec field. Without Qt: tested by cargo test.
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
    pub wm_class: String, // StartupWMClass: the app-id its windows carry, when the file says so
    pub categories: Vec<String>,
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
    let (mut name, mut exec, mut terminal, mut hidden, mut wm_class) = (None, None, false, false, String::new());
    let mut categories = Vec::new();
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
            "StartupWMClass" => wm_class = v.to_string(),
            "Categories" => categories = v.split(';').filter(|c| !c.is_empty()).map(String::from).collect(),
            "NoDisplay" | "Hidden" => hidden |= v == "true",
            "Type" if v != "Application" => return None,
            _ => {}
        }
    }
    Some(App { id: id.to_string(), name: name?, exec: exec?, terminal, hidden, wm_class, categories })
}

pub fn desktop(id: &str) -> Option<App> {
    app_dirs().iter().find_map(|d| parse_desktop(id, &fs::read_to_string(d.join(id)).ok()?))
}

// "type=a.desktop;b.desktop" lines of a file with sections, for the requested section
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

// Applications for a MIME type: the default one first, then the added associations, then the cache
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
        // Hidden entries (NoDisplay) are only offered if the user chose them as the default
        if let Some(app) = desktop(&id).filter(|a| !a.hidden || i == 0) {
            out.push(app);
        }
    }
    out
}

// Every application that can be picked in the "Other…" dialog: the visible .desktop files of the application
// folders (the first folder wins for a given id), sorted by name
pub fn all() -> Vec<App> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for d in app_dirs() {
        let Ok(entries) = fs::read_dir(&d) else { continue };
        let mut names: Vec<String> = entries.flatten().filter_map(|e| e.file_name().into_string().ok()).filter(|n| n.ends_with(".desktop")).collect();
        names.sort();
        for id in names {
            if !seen.insert(id.clone()) {
                continue;
            }
            if let Some(app) = fs::read_to_string(d.join(&id)).ok().and_then(|t| parse_desktop(&id, &t)).filter(|a| !a.hidden) {
                out.push(app);
            }
        }
    }
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

// Edits the "mime=a;b;" line of a section of mimeapps.list (creating the section or the line): `f` gets the
// current ids and returns the new ones
fn edit_section(lines: &mut Vec<String>, header: &str, mime: &str, f: impl Fn(Vec<String>) -> Vec<String>) {
    let start = match lines.iter().position(|l| l.trim() == header) {
        Some(i) => i,
        None => {
            if lines.last().is_some_and(|l| !l.trim().is_empty()) {
                lines.push(String::new());
            }
            lines.push(header.to_string());
            lines.len() - 1
        }
    };
    let end = lines[start + 1..].iter().position(|l| l.trim().starts_with('[')).map_or(lines.len(), |p| start + 1 + p);
    let key = format!("{mime}=");
    let at = (start + 1..end).find(|&i| lines[i].trim().starts_with(&key));
    let current: Vec<String> = at.map(|i| lines[i].trim()[key.len()..].split(';').filter(|s| !s.is_empty()).map(String::from).collect()).unwrap_or_default();
    let line = format!("{key}{};", f(current).join(";"));
    match at {
        Some(i) => lines[i] = line,
        None => lines.insert(start + 1, line),
    }
}

// mimeapps.list text where `id` is the default application of `mime` (the former ones stay after it as fallbacks)
// and is also a known association of the type
pub fn with_default(text: &str, mime: &str, id: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(String::from).collect();
    edit_section(&mut lines, "[Default Applications]", mime, |cur| std::iter::once(id.to_string()).chain(cur.into_iter().filter(|c| c != id)).collect());
    edit_section(&mut lines, "[Added Associations]", mime, |mut cur| {
        if !cur.iter().any(|c| c == id) {
            cur.push(id.to_string());
        }
        cur
    });
    lines.join("\n") + "\n"
}

// Where the choices live: "open" is the user's mimeapps.list (shared with the desktop), "edit" is Casier's own list of
// preferred editors (the freedesktop specification has no notion of editor), same format, in the state folder
fn list_path(edit: bool) -> PathBuf {
    if edit {
        let state = env::var("XDG_STATE_HOME").unwrap_or_else(|_| format!("{}/.local/state", home()));
        return PathBuf::from(state).join("filigrane/fili-file/editors.list");
    }
    let config = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home()));
    PathBuf::from(config).join("mimeapps.list")
}

// Makes `id` the default application (or editor) of `mime` for the user, file replaced atomically
pub fn set_default(mime: &str, id: &str, edit: bool) -> std::io::Result<()> {
    let path = list_path(edit);
    let text = with_default(&fs::read_to_string(&path).unwrap_or_default(), mime, id);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("list.tmp");
    fs::write(&tmp, text)?;
    fs::rename(tmp, path)
}

// Whether an application is made to change files (not only to look at them)
fn is_editor(app: &App) -> bool {
    const EDITING: [&str; 7] = ["TextEditor", "Graphics", "AudioVideoEditing", "Office", "WordProcessor", "Spreadsheet", "Presentation"];
    app.categories.iter().any(|c| EDITING.contains(&c.as_str())) && !app.categories.iter().any(|c| c == "Viewer")
}

// Editors for a MIME type: the ones the user chose, then the applications of the type made for editing and, for a text
// type, every text editor installed
pub fn editors_for_mime(mime: &str) -> Vec<App> {
    let chosen = fs::read_to_string(list_path(true)).unwrap_or_default();
    let mut ids = section_map(&chosen, "[Default Applications]").remove(mime).unwrap_or_default();
    let mut found: Vec<App> = for_mime(mime).into_iter().filter(is_editor).collect();
    if mime.starts_with("text/") || mime.ends_with("+xml") || mime.ends_with("/json") {
        found.extend(all().into_iter().filter(|a| a.categories.iter().any(|c| c == "TextEditor")));
    }
    ids.extend(found.iter().map(|a| a.id.clone()));
    let mut seen = Vec::new();
    ids.into_iter().filter(|id| !seen.contains(id) && { seen.push(id.clone()); true }).filter_map(|id| desktop(&id)).collect()
}

// Splits an Exec field (double quotes, backslash) into arguments, codes %f %F %u %U replaced by path
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
        let app = |exec: &str, terminal| App { id: "a".into(), name: "a".into(), exec: exec.into(), terminal, hidden: false, wm_class: String::new(), categories: Vec::new() };
        assert_eq!(command(&app("viewer --file=%f %i", false), "/tmp/a b.png"), ["viewer", "--file=/tmp/a b.png"]);
        assert_eq!(command(&app("\"/opt/my app/run\" --x %U 100%%", false), "/p"), ["/opt/my app/run", "--x", "/p", "100%"]);
        assert_eq!(command(&app("vim %F", true), "/p"), ["foot", "-e", "vim", "/p"]);
    }

    #[test]
    fn default_is_set_and_added() {
        let t = with_default("", "image/png", "a.desktop");
        assert_eq!(t, "[Default Applications]\nimage/png=a.desktop;\n\n[Added Associations]\nimage/png=a.desktop;\n");
        let t = with_default("[Default Applications]\nimage/png=old.desktop;\ntext/plain=e.desktop;\n[Added Associations]\nimage/png=old.desktop;\n", "image/png", "a.desktop");
        let m = section_map(&t, "[Default Applications]");
        assert_eq!(m["image/png"], ["a.desktop", "old.desktop"]);
        assert_eq!(m["text/plain"], ["e.desktop"]);
        assert_eq!(section_map(&t, "[Added Associations]")["image/png"], ["old.desktop", "a.desktop"]);
        assert_eq!(with_default(&t, "image/png", "a.desktop"), t);
    }

    #[test]
    fn editors_are_told_from_viewers() {
        let t = |c: &str| parse_desktop("a.desktop", &format!("[Desktop Entry]\nName=a\nExec=a\nCategories={c}\n")).unwrap();
        assert!(is_editor(&t("Utility;TextEditor;")));
        assert!(is_editor(&t("Graphics;RasterGraphics;")));
        assert!(!is_editor(&t("Graphics;Viewer;")));
        assert!(!is_editor(&t("Network;")));
    }

    #[test]
    fn sections() {
        let m = section_map("[Default Applications]\nimage/png=imv.desktop;b.desktop;\n[Added Associations]\nimage/png=c.desktop;\n", "[Default Applications]");
        assert_eq!(m["image/png"], ["imv.desktop", "b.desktop"]);
    }
}

// Names the windows of an application may carry as app-id: its StartupWMClass, the id of its .desktop file without the
// extension, and the program it runs (lowercase, to compare without case)
pub fn window_names(app: &App) -> Vec<String> {
    let program = app.exec.split_whitespace().next().unwrap_or("").rsplit('/').next().unwrap_or("");
    [app.wm_class.as_str(), app.id.trim_end_matches(".desktop"), program]
        .iter()
        .filter(|n| !n.is_empty())
        .map(|n| n.to_lowercase())
        .collect()
}
