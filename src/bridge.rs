// Objets exposés au QML (module Fili.Files): FolderModel, la liste du dossier affiché (navigation, historique, tri,
// ouverture); Fili, l'environnement (thème suivi en direct, traductions, lieux de la barre latérale).
#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++Qt" {
        include!(<QtCore/QAbstractListModel>);
        #[qobject]
        type QAbstractListModel;
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(QString, path, READ, NOTIFY = changed)]
        #[qproperty(bool, show_hidden, cxx_name = "showHidden", READ, NOTIFY = changed)]
        #[qproperty(i32, sort_key, cxx_name = "sortKey", READ, NOTIFY = changed)]
        #[qproperty(bool, sort_desc, cxx_name = "sortDesc", READ, NOTIFY = changed)]
        #[qproperty(bool, can_back, cxx_name = "canBack", READ, NOTIFY = changed)]
        #[qproperty(bool, can_forward, cxx_name = "canForward", READ, NOTIFY = changed)]
        #[qproperty(QString, status, READ, NOTIFY = changed)]
        #[qproperty(QString, error, READ, NOTIFY = changed)]
        type FolderModel = super::FolderModelRust;

        #[qsignal]
        fn changed(self: Pin<&mut FolderModel>);

        // Les déplacements rendent la ligne à sélectionner ensuite (-1: rien n'a changé)
        #[qinvokable]
        fn go(self: Pin<&mut FolderModel>, path: &QString) -> i32;
        #[qinvokable]
        fn activate(self: Pin<&mut FolderModel>, row: i32) -> i32;
        #[qinvokable]
        fn up(self: Pin<&mut FolderModel>) -> i32;
        #[qinvokable]
        fn back(self: Pin<&mut FolderModel>) -> i32;
        #[qinvokable]
        fn forward(self: Pin<&mut FolderModel>) -> i32;
        #[qinvokable]
        fn refresh(self: Pin<&mut FolderModel>, row: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "sortBy"]
        fn sort_by(self: Pin<&mut FolderModel>, key: i32, row: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "toggleHidden"]
        fn toggle_hidden(self: Pin<&mut FolderModel>, row: i32) -> i32;
        #[qinvokable]
        fn quicklook(self: &FolderModel, row: i32);
        #[qinvokable]
        fn find(self: &FolderModel, prefix: &QString) -> i32;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        unsafe fn begin_reset_model(self: Pin<&mut FolderModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        unsafe fn end_reset_model(self: Pin<&mut FolderModel>);
    }

    extern "RustQt" {
        #[cxx_override]
        fn data(self: &FolderModel, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &FolderModel) -> QHash_i32_QByteArray;
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &FolderModel, parent: &QModelIndex) -> i32;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(bool, light, READ, NOTIFY = theme_changed)]
        #[qproperty(bool, key_nav, cxx_name = "keyNav")]
        type Fili = super::FiliRust;

        #[qsignal]
        #[cxx_name = "themeChanged"]
        fn theme_changed(self: Pin<&mut Fili>);

        #[qinvokable]
        fn tr(self: &Fili, text: &QString) -> QString;
        // Lieux de la barre latérale, en JSON: [{section, name, path, icon}]
        #[qinvokable]
        fn places(self: &Fili) -> QString;
        // Premier argument de la ligne de commande (dossier, fichier ou URL file://), sinon le dossier personnel
        #[qinvokable]
        #[cxx_name = "startPath"]
        fn start_path(self: &Fili) -> QString;
        #[qinvokable]
        fn home(self: &Fili) -> QString;
    }

    impl cxx_qt::Initialize for Fili {}
    impl cxx_qt::Threading for Fili {}
}

use crate::{fs, i18n, icons};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant};
use qobject::*;
use std::{
    collections::HashMap,
    env,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

const ROLE_NAME: i32 = 256;
const ROLE_PATH: i32 = 257;
const ROLE_DIR: i32 = 258;
const ROLE_SIZE: i32 = 259;
const ROLE_DATE: i32 = 260;
const ROLE_ICON: i32 = 261;
const ROLE_HIDDEN: i32 = 262;

fn home() -> String {
    env::var("HOME").unwrap_or_else(|_| "/".into())
}

// Dossiers personnels XDG (user-dirs.dirs) -> nom d'icône Papirus
fn xdg_dirs() -> Vec<(String, &'static str)> {
    let home = home();
    let config = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{home}/.config"));
    let text = std::fs::read_to_string(format!("{config}/user-dirs.dirs")).unwrap_or_default();
    let icons = [
        ("XDG_DESKTOP_DIR", "user-desktop"),
        ("XDG_DOCUMENTS_DIR", "folder-documents"),
        ("XDG_DOWNLOAD_DIR", "folder-download"),
        ("XDG_MUSIC_DIR", "folder-music"),
        ("XDG_PICTURES_DIR", "folder-pictures"),
        ("XDG_VIDEOS_DIR", "folder-videos"),
    ];
    let mut out = Vec::new();
    for (key, icon) in icons {
        let v = text.lines().find_map(|l| l.strip_prefix(key)?.strip_prefix('=')).map(|v| v.trim().trim_matches('"'));
        if let Some(v) = v {
            let p = v.replace("$HOME", &home);
            if p != home && Path::new(&p).is_dir() {
                out.push((p, icon));
            }
        }
    }
    out
}

// Dossiers spéciaux pour les icônes de la liste
fn special() -> &'static HashMap<String, &'static str> {
    static S: std::sync::OnceLock<HashMap<String, &'static str>> = std::sync::OnceLock::new();
    S.get_or_init(|| {
        let mut m: HashMap<String, &'static str> = xdg_dirs().into_iter().collect();
        m.insert(home(), "user-home");
        m
    })
}

fn percent_decode(s: &str) -> String {
    let hex = |c: u8| (c as char).to_digit(16).map(|d| d as u8);
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && let (Some(h), Some(l)) = (b.get(i + 1).copied().and_then(hex), b.get(i + 2).copied().and_then(hex))
        {
            out.push(h * 16 + l);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

// Chemin absolu nettoyé (~, file://, ., ..), sans suivre les liens: le chemin affiché reste celui parcouru
fn normalize(p: &str, base: &str) -> PathBuf {
    let p = p.trim();
    let p = match p.strip_prefix("file://") {
        Some(rest) => percent_decode(rest),
        None => p.to_string(),
    };
    let p = if p == "~" {
        home()
    } else if let Some(rest) = p.strip_prefix("~/") {
        format!("{}/{rest}", home())
    } else {
        p
    };
    let full = if p.starts_with('/') { PathBuf::from(p) } else { Path::new(base).join(p) };
    let mut out = PathBuf::from("/");
    for c in full.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::Normal(n) => out.push(n),
            _ => {}
        }
    }
    out
}

fn json(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

// Lance un programme détaché (ne meurt pas avec fili-file), ramassé par un fil pour ne pas laisser de zombie
fn spawn(program: &str, arg: &str, dir: &Path) {
    let child = Command::new(program)
        .arg(arg)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn();
    if let Ok(mut child) = child {
        std::thread::spawn(move || child.wait());
    }
}

fn bin(name: &str) -> String {
    let p = format!("{}/.local/bin/{name}", home());
    if Path::new(&p).exists() { p } else { name.to_string() }
}

#[derive(Default)]
pub struct FolderModelRust {
    path: QString,
    show_hidden: bool,
    sort_key: i32,
    sort_desc: bool,
    can_back: bool,
    can_forward: bool,
    status: QString,
    error: QString,
    dir: String,
    all: Vec<fs::Entry>,
    rows: Vec<fs::Entry>,
    history_back: Vec<String>,
    history_forward: Vec<String>,
    now: i64,
}

impl FolderModel {
    fn row(&self, row: i32) -> Option<&fs::Entry> {
        usize::try_from(row).ok().and_then(|r| self.rows.get(r))
    }

    fn row_of(&self, name: &str) -> i32 {
        self.rows.iter().position(|e| e.name == name).map_or(0, |i| i as i32)
    }

    // Filtre (cachés) et tri de la liste courante, sans relire le disque
    fn rebuild(mut self: Pin<&mut Self>) {
        let (hidden, key, desc) = (self.show_hidden, fs::Key::from(self.sort_key), self.sort_desc);
        let mut rows: Vec<fs::Entry> = self.all.iter().filter(|e| hidden || !e.hidden()).cloned().collect();
        fs::sort(&mut rows, key, desc);
        let free = fs::free(Path::new(&self.dir)).map(fs::size);
        let status = match free {
            Some(f) => i18n::trf("%1, %2 available", &[&fs::count(rows.len()), &f]),
            None => fs::count(rows.len()),
        };
        unsafe { self.as_mut().begin_reset_model() };
        let mut r = self.as_mut().rust_mut();
        r.rows = rows;
        r.status = QString::from(&status);
        r.now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
        unsafe { self.as_mut().end_reset_model() };
    }

    // Charge dir; select: nom à sélectionner. Échec: le dossier affiché ne change pas, l'erreur va dans la barre d'état
    fn load(mut self: Pin<&mut Self>, dir: &Path, select: Option<&str>) -> Result<i32, ()> {
        match fs::list(dir) {
            Ok(all) => {
                let dir = dir.to_string_lossy().into_owned();
                let mut r = self.as_mut().rust_mut();
                r.all = all;
                r.path = QString::from(&dir);
                r.dir = dir;
                r.error = QString::default();
                self.as_mut().rebuild();
                let row = select.map_or(0, |n| self.row_of(n));
                self.as_mut().update_history_flags();
                Ok(row)
            }
            Err(e) => {
                let name = dir.file_name().map_or("/".into(), |n| n.to_string_lossy().into_owned());
                let msg = i18n::trf("Cannot open “%1”: %2", &[&name, &fs::error(&e)]);
                self.as_mut().rust_mut().error = QString::from(&msg);
                self.as_mut().changed();
                Err(())
            }
        }
    }

    fn update_history_flags(mut self: Pin<&mut Self>) {
        let mut r = self.as_mut().rust_mut();
        r.can_back = !r.history_back.is_empty();
        r.can_forward = !r.history_forward.is_empty();
        self.as_mut().changed();
    }

    // Va dans target en gardant l'historique; un fichier ouvre son dossier et le sélectionne
    fn navigate(mut self: Pin<&mut Self>, target: PathBuf, select: Option<String>) -> i32 {
        let (dir, select) = if !target.is_dir() && target.parent().is_some() && target.exists() {
            let name = target.file_name().map(|n| n.to_string_lossy().into_owned());
            (target.parent().unwrap_or(Path::new("/")).to_path_buf(), name)
        } else {
            (target, select)
        };
        if dir.to_string_lossy() == self.dir {
            return select.map_or(-1, |n| self.row_of(&n));
        }
        let previous = self.dir.clone();
        match self.as_mut().load(&dir, select.as_deref()) {
            Ok(row) => {
                if !previous.is_empty() {
                    let mut r = self.as_mut().rust_mut();
                    r.history_back.push(previous);
                    r.history_forward.clear();
                    self.as_mut().update_history_flags();
                }
                row
            }
            Err(()) => -1,
        }
    }

    pub fn go(mut self: Pin<&mut Self>, path: &QString) -> i32 {
        let first = self.dir.is_empty();
        let base = if first { home() } else { self.dir.clone() };
        let target = normalize(&path.to_string(), &base);
        let row = self.as_mut().navigate(target, None);
        // Premier dossier introuvable: le dossier personnel, l'erreur reste affichée
        if row < 0 && first {
            let error = self.error.clone();
            let row = self.as_mut().navigate(PathBuf::from(home()), None);
            self.as_mut().rust_mut().error = error;
            self.as_mut().changed();
            return row;
        }
        row
    }

    pub fn activate(self: Pin<&mut Self>, row: i32) -> i32 {
        let Some(e) = self.row(row).cloned() else { return -1 };
        if e.is_dir {
            return self.navigate(PathBuf::from(&e.path), None);
        }
        spawn("xdg-open", &e.path, Path::new(&self.dir));
        -1
    }

    // Dossier parent, le dossier quitté sélectionné (Finder)
    pub fn up(self: Pin<&mut Self>) -> i32 {
        let here = PathBuf::from(&self.dir);
        let Some(parent) = here.parent() else { return -1 };
        let name = here.file_name().map(|n| n.to_string_lossy().into_owned());
        self.navigate(parent.to_path_buf(), name)
    }

    // Retour et avance: le dossier quitté est sélectionné s'il est dans celui où l'on arrive
    fn jump(mut self: Pin<&mut Self>, forward: bool) -> i32 {
        let target = {
            let mut r = self.as_mut().rust_mut();
            let stack = if forward { &mut r.history_forward } else { &mut r.history_back };
            match stack.pop() {
                Some(t) => t,
                None => return -1,
            }
        };
        let here = self.dir.clone();
        let select = Path::new(&here)
            .strip_prefix(&target)
            .ok()
            .and_then(|rel| rel.components().next())
            .map(|c| c.as_os_str().to_string_lossy().into_owned());
        match self.as_mut().load(Path::new(&target), select.as_deref()) {
            Ok(row) => {
                let mut r = self.as_mut().rust_mut();
                if forward { r.history_back.push(here) } else { r.history_forward.push(here) }
                self.as_mut().update_history_flags();
                row
            }
            Err(()) => {
                // Dossier disparu: retiré de l'historique
                self.as_mut().update_history_flags();
                -1
            }
        }
    }

    pub fn back(self: Pin<&mut Self>) -> i32 {
        self.jump(false)
    }

    pub fn forward(self: Pin<&mut Self>) -> i32 {
        self.jump(true)
    }

    fn selected_name(&self, row: i32) -> Option<String> {
        self.row(row).map(|e| e.name.clone())
    }

    pub fn refresh(mut self: Pin<&mut Self>, row: i32) -> i32 {
        let name = self.selected_name(row);
        let dir = PathBuf::from(&self.dir);
        self.as_mut().load(&dir, name.as_deref()).unwrap_or(-1)
    }

    // Même colonne: ordre inversé; autre colonne: croissant
    pub fn sort_by(mut self: Pin<&mut Self>, key: i32, row: i32) -> i32 {
        let name = self.selected_name(row);
        let mut r = self.as_mut().rust_mut();
        if r.sort_key == key {
            r.sort_desc = !r.sort_desc;
        } else {
            r.sort_key = key;
            r.sort_desc = false;
        }
        self.as_mut().rebuild();
        self.as_mut().changed();
        name.map_or(0, |n| self.row_of(&n))
    }

    pub fn toggle_hidden(mut self: Pin<&mut Self>, row: i32) -> i32 {
        let name = self.selected_name(row);
        let mut r = self.as_mut().rust_mut();
        r.show_hidden = !r.show_hidden;
        self.as_mut().rebuild();
        self.as_mut().changed();
        name.map_or(0, |n| self.row_of(&n))
    }

    // Aperçu rapide (~/.local/bin/quicklook): relancé sur le même fichier, il se referme
    pub fn quicklook(&self, row: i32) {
        if let Some(e) = self.row(row) {
            spawn(&bin("quicklook"), &e.path, Path::new(&self.dir));
        }
    }

    // Saisie au clavier: première ligne dont le nom commence par prefix (sans casse ni accents)
    pub fn find(&self, prefix: &QString) -> i32 {
        let p = prefix.to_string();
        self.rows.iter().position(|e| fs::starts_with(&e.name, &p)).map_or(-1, |i| i as i32)
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(e) = self.row(index.row()) else { return QVariant::default() };
        match role {
            ROLE_NAME => QVariant::from(&QString::from(&e.name)),
            ROLE_PATH => QVariant::from(&QString::from(&e.path)),
            ROLE_DIR => QVariant::from(&e.is_dir),
            ROLE_SIZE => QVariant::from(&QString::from(&if e.is_dir { String::from("—") } else { fs::size(e.size) })),
            ROLE_DATE => QVariant::from(&QString::from(&fs::date(e.mtime, self.now))),
            ROLE_ICON => QVariant::from(&QString::from(&if e.is_dir {
                icons::folder(&e.path, special())
            } else {
                icons::file(e)
            })),
            ROLE_HIDDEN => QVariant::from(&e.hidden()),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        for (role, name) in [
            (ROLE_NAME, "name"),
            (ROLE_PATH, "path"),
            (ROLE_DIR, "isDir"),
            (ROLE_SIZE, "size"),
            (ROLE_DATE, "modified"),
            (ROLE_ICON, "icon"),
            (ROLE_HIDDEN, "hidden"),
        ] {
            roles.insert(role, QByteArray::from(name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rows.len() as i32
    }
}

#[derive(Default)]
pub struct FiliRust {
    light: bool,
    key_nav: bool,
}

// État de fili-theme (« light » ou « dark »), comme FiliTheme.qml du fili-shell
fn theme_file() -> PathBuf {
    let state = env::var("XDG_STATE_HOME").unwrap_or_else(|_| format!("{}/.local/state", home()));
    PathBuf::from(state).join("filigrane/theme")
}

fn read_light() -> bool {
    std::fs::read_to_string(theme_file()).is_ok_and(|t| t.trim() == "light")
}

impl cxx_qt::Initialize for Fili {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().light = read_light();
        // Suivi du dossier d'état (inotify, fil bloqué sur read: aucun réveil tant que rien ne change)
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let file = theme_file();
            let Some(dir) = file.parent() else { return };
            let _ = std::fs::create_dir_all(dir);
            let Ok(c) = std::ffi::CString::new(dir.as_os_str().as_encoded_bytes()) else { return };
            let fd = unsafe { libc::inotify_init1(libc::IN_CLOEXEC) };
            if fd < 0
                || unsafe { libc::inotify_add_watch(fd, c.as_ptr(), libc::IN_CLOSE_WRITE | libc::IN_MOVED_TO | libc::IN_CREATE) } < 0
            {
                return;
            }
            let mut buf = [0u8; 4096];
            loop {
                if unsafe { libc::read(fd, buf.as_mut_ptr().cast(), buf.len()) } <= 0 {
                    return;
                }
                let light = read_light();
                let sent = qt.queue(move |mut f| {
                    if f.light != light {
                        f.as_mut().rust_mut().light = light;
                        f.as_mut().theme_changed();
                    }
                });
                if sent.is_err() {
                    return;
                }
            }
        });
    }
}

impl Fili {
    pub fn tr(&self, text: &QString) -> QString {
        QString::from(i18n::tr(&text.to_string()))
    }

    pub fn places(&self) -> QString {
        let home = home();
        let (places, devices) = (i18n::tr("Places"), i18n::tr("Devices"));
        let mut seen = vec![home.clone()];
        let mut items = vec![(
            places,
            Path::new(&home).file_name().map_or("~".into(), |n| n.to_string_lossy().into_owned()),
            home.clone(),
            icons::named("user-home"),
        )];
        let name = |p: &str| Path::new(p).file_name().map_or(p.to_string(), |n| n.to_string_lossy().into_owned());
        for (p, _) in xdg_dirs() {
            items.push((places, name(&p), p.clone(), icons::folder(&p, special())));
            seen.push(p);
        }
        // Signets GTK (partagés avec Thunar et le sélecteur de fichiers): « file:///chemin libellé »
        let config = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{home}/.config"));
        for line in std::fs::read_to_string(format!("{config}/gtk-3.0/bookmarks")).unwrap_or_default().lines() {
            let (uri, label) = line.split_once(' ').unwrap_or((line, ""));
            let Some(rest) = uri.strip_prefix("file://") else { continue };
            let p = percent_decode(rest);
            if seen.contains(&p) || !Path::new(&p).is_dir() {
                continue;
            }
            let label = if label.is_empty() { name(&p) } else { label.to_string() };
            items.push((places, label, p.clone(), icons::folder(&p, special())));
            seen.push(p);
        }
        items.push((devices, i18n::tr("Computer").to_string(), "/".into(), icons::named("drive-harddisk")));
        let body: Vec<String> = items
            .iter()
            .map(|(s, n, p, i)| {
                format!(
                    "{{\"section\":{},\"name\":{},\"path\":{},\"icon\":{}}}",
                    json(s),
                    json(n),
                    json(p),
                    json(i)
                )
            })
            .collect();
        QString::from(&format!("[{}]", body.join(",")))
    }

    pub fn start_path(&self) -> QString {
        let arg = env::args().skip(1).find(|a| !a.starts_with('-'));
        let p = normalize(&arg.unwrap_or_else(home), &env::current_dir().map_or(home(), |d| d.to_string_lossy().into_owned()));
        QString::from(&p.to_string_lossy().into_owned())
    }

    pub fn home(&self) -> QString {
        QString::from(&home())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_paths() {
        let h = home();
        assert_eq!(normalize("~/a/../b/./c", "/"), PathBuf::from(format!("{h}/b/c")));
        assert_eq!(normalize("file:///tmp/a%20b", "/"), PathBuf::from("/tmp/a b"));
        assert_eq!(normalize("../x", "/usr/share"), PathBuf::from("/usr/x"));
        assert_eq!(normalize("/..", "/"), PathBuf::from("/"));
    }

    #[test]
    fn decodes_percent() {
        assert_eq!(percent_decode("caf%C3%A9%2"), "café%2");
    }
}
