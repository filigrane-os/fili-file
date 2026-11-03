// Freedesktop icon theme lookup (Icon Theme Specification): index.theme parsing, directory matching by size, theme
// inheritance (`Inherits`), hicolor as the last theme. Works with any installed theme (Adwaita, Papirus, Breeze, ...).
//
// Which theme is used: `icon_theme` in fili-file.toml, else the desktop setting (gsettings, then GTK settings.ini),
// else Adwaita. A theme may carry a few optional hints for fili-file in the `[Icon Theme]` section of its
// index.theme (other applications ignore unknown `X-` keys):
//   X-Fili-FolderIcon  icon name used for every folder (default: folder)
//   X-Fili-FileIcon    icon name used for every regular file instead of the one of its type
//   X-Fili-GlyphDir    directory of the theme with filled glyphs for the sidebar (default: <name>-symbolic icons)
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind {
    Fixed,
    Scalable,
    Threshold,
}

#[derive(Debug, Clone)]
struct Dir {
    path: String,
    size: u32,
    min: u32,
    max: u32,
    threshold: u32,
    kind: Kind,
}

impl Dir {
    fn matches(&self, size: u32) -> bool {
        match self.kind {
            Kind::Fixed => self.size == size,
            Kind::Scalable => self.min <= size && size <= self.max,
            Kind::Threshold => self.size.saturating_sub(self.threshold) <= size && size <= self.size + self.threshold,
        }
    }

    fn distance(&self, size: u32) -> u32 {
        match self.kind {
            Kind::Fixed => self.size.abs_diff(size),
            Kind::Scalable => {
                if size < self.min {
                    self.min - size
                } else {
                    size.saturating_sub(self.max)
                }
            }
            Kind::Threshold => {
                let (lo, hi) = (self.size.saturating_sub(self.threshold), self.size + self.threshold);
                if size < lo {
                    lo - size
                } else {
                    size.saturating_sub(hi)
                }
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct Index {
    dirs: Vec<Dir>,
    inherits: Vec<String>,
    keys: HashMap<String, String>, // keys of the [Icon Theme] section
}

fn number(v: Option<&String>, default: u32) -> u32 {
    v.and_then(|v| v.trim().parse().ok()).unwrap_or(default)
}

pub fn parse_index(text: &str) -> Index {
    let mut sections: Vec<(String, HashMap<String, String>)> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            sections.push((name.to_string(), HashMap::new()));
        } else if let (Some((_, map)), Some((k, v))) = (sections.last_mut(), line.split_once('=')) {
            map.entry(k.trim().to_string()).or_insert_with(|| v.trim().to_string());
        }
    }
    let mut index = Index::default();
    let Some((_, head)) = sections.iter().find(|(n, _)| n == "Icon Theme") else { return index };
    index.keys = head.clone();
    let list = |k: &str| -> Vec<String> {
        head.get(k).map(|v| v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()).unwrap_or_default()
    };
    index.inherits = list("Inherits");
    // ScaledDirectories (Scale > 1) are for high-density screens; Qt scales the plain directories instead
    for name in list("Directories") {
        let Some((_, d)) = sections.iter().find(|(n, _)| *n == name) else { continue };
        if number(d.get("Scale"), 1) != 1 {
            continue;
        }
        let size = number(d.get("Size"), 0);
        let kind = match d.get("Type").map(String::as_str) {
            Some("Fixed") => Kind::Fixed,
            Some("Scalable") => Kind::Scalable,
            _ => Kind::Threshold,
        };
        index.dirs.push(Dir {
            path: name,
            size,
            min: number(d.get("MinSize"), size),
            max: number(d.get("MaxSize"), size),
            threshold: number(d.get("Threshold"), 2),
            kind,
        });
    }
    index
}

// Icon extensions by preference: scalable first, Qt scales it to any size
const EXTENSIONS: [&str; 3] = ["svg", "png", "xpm"];

pub struct Themes {
    bases: Vec<PathBuf>,
    indexes: Mutex<HashMap<String, Option<Arc<Index>>>>,
}

impl Themes {
    pub fn new(bases: Vec<PathBuf>) -> Self {
        Themes { bases, indexes: Mutex::new(HashMap::new()) }
    }

    // $XDG_DATA_HOME/icons, ~/.icons, then <dir>/icons for each of XDG_DATA_DIRS
    pub fn system() -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let mut bases: Vec<PathBuf> = data_dirs().into_iter().map(|d| d.join("icons")).collect();
        bases.insert(1.min(bases.len()), PathBuf::from(format!("{home}/.icons")));
        Themes::new(bases)
    }

    // index.theme of the first directory that has the theme
    fn index(&self, theme: &str) -> Option<Arc<Index>> {
        let mut cache = self.indexes.lock().unwrap_or_else(|e| e.into_inner());
        cache
            .entry(theme.to_string())
            .or_insert_with(|| {
                self.bases.iter().find_map(|b| fs::read_to_string(b.join(theme).join("index.theme")).ok()).map(|t| Arc::new(parse_index(&t)))
            })
            .clone()
    }

    pub fn exists(&self, theme: &str) -> bool {
        self.index(theme).is_some()
    }

    // `X-` hint of a theme (its own index.theme only, not the inherited ones)
    pub fn key(&self, theme: &str, key: &str) -> Option<String> {
        self.index(theme)?.keys.get(key).cloned().filter(|v| !v.is_empty())
    }

    fn file_in(&self, theme: &str, dir: &str, name: &str) -> Option<PathBuf> {
        for base in &self.bases {
            for ext in EXTENSIONS {
                let p = base.join(theme).join(dir).join(format!("{name}.{ext}"));
                if p.is_file() {
                    return Some(p);
                }
            }
        }
        None
    }

    // A file of the theme itself (no inheritance): <theme>/<dir>/<name>.<svg|png|xpm>
    pub fn in_dir(&self, theme: &str, dir: &str, name: &str) -> Option<PathBuf> {
        self.file_in(theme, dir, name)
    }

    fn lookup_theme(&self, theme: &str, name: &str, size: u32, seen: &mut HashSet<String>) -> Option<PathBuf> {
        if !seen.insert(theme.to_string()) {
            return None; // inheritance cycle
        }
        let index = self.index(theme)?;
        if let Some(p) = index.dirs.iter().filter(|d| d.matches(size)).find_map(|d| self.file_in(theme, &d.path, name)) {
            return Some(p);
        }
        // No directory of the right size: the closest one that has the icon
        let closest = index
            .dirs
            .iter()
            .filter_map(|d| self.file_in(theme, &d.path, name).map(|p| (d.distance(size), p)))
            .min_by_key(|(distance, _)| *distance);
        if let Some((_, p)) = closest {
            return Some(p);
        }
        index.inherits.iter().find_map(|parent| self.lookup_theme(parent, name, size, seen))
    }

    // The icon `name` for `size` px: the theme, the themes it inherits, then hicolor
    pub fn lookup(&self, theme: &str, name: &str, size: u32) -> Option<PathBuf> {
        let mut seen = HashSet::new();
        self.lookup_theme(theme, name, size, &mut seen).or_else(|| self.lookup_theme("hicolor", name, size, &mut seen))
    }
}

// $XDG_DATA_HOME (default ~/.local/share), then XDG_DATA_DIRS (default /usr/local/share:/usr/share)
pub fn data_dirs() -> Vec<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    let nonempty = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
    let mut dirs = vec![PathBuf::from(nonempty("XDG_DATA_HOME").unwrap_or_else(|| format!("{home}/.local/share")))];
    dirs.extend(nonempty("XDG_DATA_DIRS").unwrap_or_else(|| "/usr/local/share:/usr/share".into()).split(':').filter(|d| !d.is_empty()).map(PathBuf::from));
    dirs
}

// "gtk-icon-theme-name=Papirus" of a GTK settings.ini
fn from_settings_ini(text: &str) -> Option<String> {
    text.lines().find_map(|l| l.trim().strip_prefix("gtk-icon-theme-name").and_then(|r| r.trim_start().strip_prefix('=')).map(|v| v.trim().trim_matches('"').to_string())).filter(|v| !v.is_empty())
}

// The desktop's icon theme: gsettings (what GTK follows under Wayland), then GTK 4 and 3 settings.ini
fn desktop_theme() -> Option<String> {
    if let Ok(out) = std::process::Command::new("gsettings").args(["get", "org.gnome.desktop.interface", "icon-theme"]).stderr(std::process::Stdio::null()).output() {
        let v = String::from_utf8_lossy(&out.stdout).trim().trim_matches('\'').to_string();
        if out.status.success() && !v.is_empty() {
            return Some(v);
        }
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let config = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{home}/.config"));
    ["gtk-4.0", "gtk-3.0"].iter().find_map(|d| fs::read_to_string(format!("{config}/{d}/settings.ini")).ok().and_then(|t| from_settings_ini(&t)))
}

pub fn system() -> &'static Themes {
    static T: OnceLock<Themes> = OnceLock::new();
    T.get_or_init(Themes::system)
}

// Name of the theme in use: the configured one if installed, else the desktop's, else Adwaita
pub fn current() -> &'static str {
    static T: OnceLock<String> = OnceLock::new();
    T.get_or_init(|| {
        let themes = system();
        [crate::config::load().icon_theme, desktop_theme()].into_iter().flatten().find(|t| themes.exists(t)).unwrap_or_else(|| "Adwaita".into())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // A throw-away icon base directory with the given files (relative paths) and index.theme texts
    struct Fixture(PathBuf);

    impl Fixture {
        fn new(tag: &str, files: &[&str], indexes: &[(&str, &str)]) -> Self {
            let root = std::env::temp_dir().join(format!("fili-icons-{tag}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            for f in files {
                let p = root.join(f);
                fs::create_dir_all(p.parent().unwrap()).unwrap();
                fs::write(p, "<svg/>").unwrap();
            }
            for (theme, text) in indexes {
                fs::create_dir_all(root.join(theme)).unwrap();
                fs::write(root.join(theme).join("index.theme"), text).unwrap();
            }
            Fixture(root)
        }

        fn themes(&self) -> Themes {
            Themes::new(vec![self.0.clone()])
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const PAPER: &str = "[Icon Theme]\nName=Paper\nInherits=Base, hicolor\nDirectories=16x16/places,scalable/places\nX-Fili-FolderIcon=folder-blue\n\n[16x16/places]\nSize=16\nType=Fixed\n\n[scalable/places]\nSize=48\nMinSize=16\nMaxSize=512\nType=Scalable\n";
    const BASE: &str = "[Icon Theme]\nName=Base\nDirectories=32x32/mimetypes\n\n[32x32/mimetypes]\nSize=32\nType=Threshold\nThreshold=4\n";
    const HICOLOR: &str = "[Icon Theme]\nName=hicolor\nDirectories=48x48/apps\n\n[48x48/apps]\nSize=48\nType=Fixed\n";

    #[test]
    fn exact_size_then_scalable_range() {
        let f = Fixture::new("exact", &["Paper/16x16/places/folder.png", "Paper/scalable/places/folder.svg"], &[("Paper", PAPER)]);
        let t = f.themes();
        assert!(t.lookup("Paper", "folder", 16).unwrap().ends_with("16x16/places/folder.png"));
        assert!(t.lookup("Paper", "folder", 48).unwrap().ends_with("scalable/places/folder.svg"));
    }

    #[test]
    fn closest_directory_when_no_size_matches() {
        let f = Fixture::new("closest", &["Paper/16x16/places/home.png"], &[("Paper", PAPER)]);
        assert!(f.themes().lookup("Paper", "home", 64).unwrap().ends_with("16x16/places/home.png"));
    }

    #[test]
    fn inherited_theme_and_hicolor_are_searched() {
        let f = Fixture::new(
            "inherit",
            &["Base/32x32/mimetypes/text-x-generic.png", "hicolor/48x48/apps/app.png"],
            &[("Paper", PAPER), ("Base", BASE), ("hicolor", HICOLOR)],
        );
        let t = f.themes();
        assert!(t.lookup("Paper", "text-x-generic", 32).unwrap().ends_with("Base/32x32/mimetypes/text-x-generic.png"));
        assert!(t.lookup("Paper", "app", 48).unwrap().ends_with("hicolor/48x48/apps/app.png"));
    }

    #[test]
    fn theme_files_win_over_inherited_ones() {
        let f = Fixture::new("wins", &["Paper/scalable/places/x.svg", "Base/32x32/mimetypes/x.png"], &[("Paper", PAPER), ("Base", BASE)]);
        assert!(f.themes().lookup("Paper", "x", 32).unwrap().ends_with("Paper/scalable/places/x.svg"));
    }

    #[test]
    fn missing_icon_and_inheritance_cycles_end() {
        let looping = "[Icon Theme]\nInherits=B\nDirectories=\n";
        let f = Fixture::new("cycle", &[], &[("A", looping), ("B", "[Icon Theme]\nInherits=A\nDirectories=\n")]);
        assert_eq!(f.themes().lookup("A", "nothing", 16), None);
        assert_eq!(f.themes().lookup("Unknown", "nothing", 16), None);
    }

    #[test]
    fn theme_hints_are_read_from_its_own_index() {
        let f = Fixture::new("hints", &[], &[("Paper", PAPER), ("Base", BASE)]);
        let t = f.themes();
        assert_eq!(t.key("Paper", "X-Fili-FolderIcon").as_deref(), Some("folder-blue"));
        assert_eq!(t.key("Base", "X-Fili-FolderIcon"), None);
        assert!(t.exists("Paper") && !t.exists("Nope"));
    }

    #[test]
    fn dir_matching_follows_the_spec() {
        let idx = parse_index(BASE);
        let d = &idx.dirs[0];
        assert!(d.matches(28) && d.matches(36) && !d.matches(37));
        assert_eq!(d.distance(40), 4);
        let fixed = Dir { path: "x".into(), size: 22, min: 22, max: 22, threshold: 2, kind: Kind::Fixed };
        assert_eq!(fixed.distance(16), 6);
    }

    #[test]
    fn gtk_settings_ini_icon_theme() {
        assert_eq!(from_settings_ini("[Settings]\ngtk-font-name=Jost 10\ngtk-icon-theme-name = Papirus-Dark\n").as_deref(), Some("Papirus-Dark"));
        assert_eq!(from_settings_ini("[Settings]\ngtk-font-name=Jost 10\n"), None);
    }
}
