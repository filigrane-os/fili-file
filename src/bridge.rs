// Objets exposés au QML (module Fili.Files): FolderModel, la liste du dossier affiché (navigation, historique, tri,
// ouverture, mise à jour en direct par inotify, opérations: copier/couper/coller en arrière-plan, corbeille,
// renommage, annulation); Fili, l'environnement (thème suivi en direct, traductions, lieux de la barre latérale).
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
        #[qproperty(QString, filter, READ, NOTIFY = changed)]
        #[qproperty(bool, busy, READ, NOTIFY = changed)]
        // Avancement de l'opération en cours (0 à 1), -1 tant que le total n'est pas connu
        #[qproperty(f64, progress, READ, NOTIFY = changed)]
        #[qproperty(QString, job, READ, NOTIFY = changed)]
        #[qproperty(bool, in_trash, cxx_name = "inTrash", READ, NOTIFY = changed)]
        #[qproperty(bool, can_paste, cxx_name = "canPaste", READ, NOTIFY = changed)]
        #[qproperty(bool, can_undo, cxx_name = "canUndo", READ, NOTIFY = changed)]
        type FolderModel = super::FolderModelRust;

        #[qsignal]
        fn changed(self: Pin<&mut FolderModel>);
        // Le contenu du dossier a changé sur le disque (ou après une opération): la vue remet sa sélection en place
        #[qsignal]
        fn reloaded(self: Pin<&mut FolderModel>);
        // Miniature de ce fichier prête (génération en arrière-plan)
        #[qsignal]
        #[cxx_name = "thumbReady"]
        fn thumb_ready(self: Pin<&mut FolderModel>, path: &QString);

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
        // Ne garder que les noms qui contiennent ce texte (vide: tout)
        #[qinvokable]
        #[cxx_name = "setFilter"]
        fn set_filter(self: Pin<&mut FolderModel>, text: &QString);
        #[qinvokable]
        fn quicklook(self: &FolderModel, row: i32);
        #[qinvokable]
        fn find(self: &FolderModel, prefix: &QString) -> i32;
        #[qinvokable]
        #[cxx_name = "pathAt"]
        fn path_at(self: &FolderModel, row: i32) -> QString;
        // Ligne portant ce nom (-1: absent)
        #[qinvokable]
        #[cxx_name = "rowOf"]
        fn find_name(self: &FolderModel, name: &QString) -> i32;
        #[qinvokable]
        fn has(self: &FolderModel, path: &QString) -> bool;
        // Description de la sélection pour la barre d'état (un élément: nom et taille ou contenu)
        #[qinvokable]
        fn describe(self: &FolderModel, paths: &QString) -> QString;
        #[qinvokable]
        #[cxx_name = "pathIsDir"]
        fn path_is_dir(self: &FolderModel, path: &QString) -> bool;
        // Opérations: les chemins sont séparés par des retours à la ligne; les rendus sont la ligne à sélectionner
        // ensuite (-1: aucune). Les erreurs vont dans la barre d'état (error).
        #[qinvokable]
        fn uris(self: &FolderModel, paths: &QString) -> QString;
        #[qinvokable]
        fn copy(self: Pin<&mut FolderModel>, paths: &QString);
        #[qinvokable]
        fn cut(self: Pin<&mut FolderModel>, paths: &QString);
        #[qinvokable]
        fn paste(self: Pin<&mut FolderModel>);
        #[qinvokable]
        #[cxx_name = "dropItems"]
        fn drop_items(self: Pin<&mut FolderModel>, urls: &QString, dest: &QString, copy: bool);
        #[qinvokable]
        fn trash(self: Pin<&mut FolderModel>, paths: &QString, row: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "putBack"]
        fn put_back(self: Pin<&mut FolderModel>, paths: &QString, row: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "emptyTrash"]
        fn empty_trash(self: Pin<&mut FolderModel>);
        #[qinvokable]
        fn rename(self: Pin<&mut FolderModel>, path: &QString, name: &QString) -> i32;
        #[qinvokable]
        #[cxx_name = "newFolder"]
        fn new_folder(self: Pin<&mut FolderModel>) -> i32;
        #[qinvokable]
        fn undo(self: Pin<&mut FolderModel>);
        // Applications pour ouvrir ce fichier, en JSON: [{id, name}] (la préférée d'abord)
        #[qinvokable]
        #[cxx_name = "openWithList"]
        fn open_with_list(self: &FolderModel, path: &QString) -> QString;
        #[qinvokable]
        #[cxx_name = "openWith"]
        fn open_with(self: &FolderModel, id: &QString, path: &QString);
        // Tous les chemins sont des archives que extract sait ouvrir
        #[qinvokable]
        #[cxx_name = "allArchives"]
        fn all_archives(self: &FolderModel, paths: &QString) -> bool;
        #[qinvokable]
        fn extract(self: Pin<&mut FolderModel>, paths: &QString);
        #[qinvokable]
        fn compress(self: Pin<&mut FolderModel>, paths: &QString);
        #[qinvokable]
        fn cancel(self: Pin<&mut FolderModel>);
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginInsertRows"]
        unsafe fn begin_insert_rows(self: Pin<&mut FolderModel>, parent: &QModelIndex, first: i32, last: i32);
        #[inherit]
        #[cxx_name = "endInsertRows"]
        unsafe fn end_insert_rows(self: Pin<&mut FolderModel>);
        #[inherit]
        #[cxx_name = "beginRemoveRows"]
        unsafe fn begin_remove_rows(self: Pin<&mut FolderModel>, parent: &QModelIndex, first: i32, last: i32);
        #[inherit]
        #[cxx_name = "endRemoveRows"]
        unsafe fn end_remove_rows(self: Pin<&mut FolderModel>);
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

        // Les volumes ou les montages ont changé: la barre latérale relit ses lieux
        #[qsignal]
        #[cxx_name = "placesChanged"]
        fn places_changed(self: Pin<&mut Fili>);
        // Volume monté à la demande de mount_volume (chemin) ou erreur (texte)
        #[qsignal]
        fn mounted(self: Pin<&mut Fili>, path: &QString);
        #[qsignal]
        #[cxx_name = "volumeError"]
        fn volume_error(self: Pin<&mut Fili>, message: &QString);
        // Favoris de l'utilisateur (signets GTK) et presse-papiers (wl-copy)
        #[qinvokable]
        #[cxx_name = "addFavorite"]
        fn add_favorite(self: Pin<&mut Fili>, path: &QString);
        #[qinvokable]
        #[cxx_name = "removeFavorite"]
        fn remove_favorite(self: Pin<&mut Fili>, path: &QString);
        #[qinvokable]
        #[cxx_name = "isFavorite"]
        fn is_favorite(self: &Fili, path: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "copyText"]
        fn copy_text(self: &Fili, text: &QString);
        #[qinvokable]
        #[cxx_name = "mountVolume"]
        fn mount_volume(self: Pin<&mut Fili>, dev: &QString);
        #[qinvokable]
        #[cxx_name = "unmountVolume"]
        fn unmount_volume(self: Pin<&mut Fili>, dev: &QString);
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
    impl cxx_qt::Initialize for FolderModel {}
    impl cxx_qt::Threading for FolderModel {}
}

use crate::{apps, fs, i18n, icons, ops, volumes, watch::Watcher};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant};
use qobject::*;
use std::{
    collections::{HashMap, HashSet},
    env,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

const ROLE_NAME: i32 = 256;
const ROLE_PATH: i32 = 257;
const ROLE_DIR: i32 = 258;
const ROLE_SIZE: i32 = 259;
const ROLE_DATE: i32 = 260;
const ROLE_ICON: i32 = 261;
const ROLE_HIDDEN: i32 = 262;
const ROLE_THUMB: i32 = 263;
const ROLE_THUMB_READY: i32 = 264;
const ROLE_IMAGE: i32 = 265;

fn home() -> String {
    env::var("HOME").unwrap_or_else(|_| "/".into())
}

// Dossiers personnels XDG (user-dirs.dirs, à défaut ~/Desktop, ~/Documents...) -> nom d'icône Papirus
fn xdg_dirs() -> Vec<(String, &'static str)> {
    let home = home();
    let config = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{home}/.config"));
    let text = std::fs::read_to_string(format!("{config}/user-dirs.dirs")).unwrap_or_default();
    let dirs = [
        ("XDG_DESKTOP_DIR", "Desktop", "user-desktop"),
        ("XDG_DOCUMENTS_DIR", "Documents", "folder-documents"),
        ("XDG_DOWNLOAD_DIR", "Downloads", "folder-download"),
        ("XDG_MUSIC_DIR", "Music", "folder-music"),
        ("XDG_PICTURES_DIR", "Pictures", "folder-pictures"),
        ("XDG_VIDEOS_DIR", "Videos", "folder-videos"),
    ];
    let mut out = Vec::new();
    for (key, fallback, icon) in dirs {
        let v = text.lines().find_map(|l| l.strip_prefix(key)?.strip_prefix('=')).map(|v| v.trim().trim_matches('"'));
        let p = v.map_or_else(|| format!("{home}/{fallback}"), |v| v.replace("$HOME", &home));
        if p != home && Path::new(&p).is_dir() {
            out.push((p, icon));
        }
    }
    out
}

// Favoris par défaut, dans l'ordre du Finder: Applications, Bureau, Documents, Téléchargements, dossier personnel,
// puis Images, Musique, Vidéos. (chemin, nom d'icône)
fn default_favorites() -> Vec<(String, &'static str)> {
    let home = home();
    let xdg = xdg_dirs();
    let by_icon = |icon: &str| xdg.iter().find(|(_, i)| *i == icon).cloned();
    let mut out = Vec::new();
    let apps = format!("{home}/Applications");
    if Path::new(&apps).is_dir() {
        out.push((apps, "folder-applications"));
    }
    for icon in ["user-desktop", "folder-documents", "folder-download"] {
        out.extend(by_icon(icon));
    }
    out.push((home.clone(), "user-home"));
    for icon in ["folder-pictures", "folder-music", "folder-videos"] {
        out.extend(by_icon(icon));
    }
    out
}

fn bookmarks_file() -> PathBuf {
    let config = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home()));
    PathBuf::from(config).join("gtk-3.0/bookmarks")
}

// Favoris de l'utilisateur: signets GTK (partagés avec le sélecteur de fichiers), « file:///chemin libellé »
fn bookmarks() -> Vec<(String, String)> {
    let text = std::fs::read_to_string(bookmarks_file()).unwrap_or_default();
    text.lines()
        .filter_map(|line| {
            let (uri, label) = line.split_once(' ').unwrap_or((line, ""));
            Some((percent_decode(uri.strip_prefix("file://")?), label.to_string()))
        })
        .collect()
}

fn write_bookmarks(list: &[(String, String)]) -> std::io::Result<()> {
    let file = bookmarks_file();
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut text = String::new();
    for (p, label) in list {
        text.push_str(&format!("file://{}{}{}\n", ops::encode(p), if label.is_empty() { "" } else { " " }, label));
    }
    std::fs::write(file, text)
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

use ops::decode as percent_decode;

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

// Miniature d'un PDF: première page par pdftoppm, gardée dans ~/.cache/filigrane/thumbs (nom = chemin, date, taille)
fn thumb_path(e: &fs::Entry) -> PathBuf {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    (&e.path, e.mtime, e.size).hash(&mut h);
    let cache = env::var("XDG_CACHE_HOME").unwrap_or_else(|_| format!("{}/.cache", home()));
    PathBuf::from(cache).join(format!("filigrane/thumbs/{:016x}.png", h.finish()))
}

static THUMBING: AtomicBool = AtomicBool::new(false);

// Nom sans extension d'archive reconnue (None: ce n'est pas une archive)
fn archive_stem(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    for ext in [".tar.gz", ".tar.xz", ".tar.bz2", ".tar.zst", ".tgz", ".tar", ".zip", ".7z", ".rar", ".iso"] {
        if lower.ends_with(ext) && name.len() > ext.len() {
            return Some(name[..name.len() - ext.len()].to_string());
        }
    }
    None
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
    filter: QString,
    busy: bool,
    progress: f64,
    job: QString,
    in_trash: bool,
    can_paste: bool,
    can_undo: bool,
    dir: String,
    all: Vec<fs::Entry>,
    rows: Vec<fs::Entry>,
    history_back: Vec<String>,
    history_forward: Vec<String>,
    now: i64,
    clipboard: Vec<String>,
    clip_cut: bool,
    undo: Vec<Undo>,
    cancel: Arc<AtomicBool>,
    watcher: Option<Watcher>,
}

// Ce que Ctrl+Z défait (le plus récent en dernier, 20 au plus)
enum Undo {
    // (ancien chemin, nouveau chemin)
    Rename(PathBuf, PathBuf),
    Trash(Vec<ops::Trashed>),
    // Éléments créés (copies, nouveau dossier, éléments remis en place): défaire = les mettre à la corbeille
    Created(Vec<PathBuf>),
    // (origine, destination)
    Moved(Vec<(PathBuf, PathBuf)>),
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Copy,
    Move,
    // Déplacer sur le même volume, copier sinon (glisser-déposer)
    Auto,
}

fn paths_of(list: &QString) -> Vec<String> {
    list.to_string().split('\n').filter(|p| !p.is_empty()).map(String::from).collect()
}

fn base(p: &str) -> String {
    Path::new(p).file_name().map_or(p.to_string(), |n| n.to_string_lossy().into_owned())
}

impl FolderModel {
    fn row(&self, row: i32) -> Option<&fs::Entry> {
        usize::try_from(row).ok().and_then(|r| self.rows.get(r))
    }

    fn row_of(&self, name: &str) -> i32 {
        self.rows.iter().position(|e| e.name == name).map_or(0, |i| i as i32)
    }

    // Liste courante filtrée (cachés) et triée, sans relire le disque
    fn filtered(&self) -> Vec<fs::Entry> {
        let (hidden, key, desc) = (self.show_hidden, fs::Key::from(self.sort_key), self.sort_desc);
        let needle = self.filter.to_string();
        let mut rows: Vec<fs::Entry> = self
            .all
            .iter()
            .filter(|e| (hidden || !e.hidden()) && (needle.is_empty() || fs::contains(&e.name, &needle)))
            .cloned()
            .collect();
        fs::sort(&mut rows, key, desc);
        rows
    }

    fn set_status(mut self: Pin<&mut Self>) {
        let n = self.rows.len();
        let free = fs::free(Path::new(&self.dir)).map(fs::size);
        let status = match free {
            Some(f) => i18n::trf("%1, %2 available", &[&fs::count(n), &f]),
            None => fs::count(n),
        };
        let mut r = self.as_mut().rust_mut();
        r.status = QString::from(&status);
        r.now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    }

    // Remplace la liste (nouveau dossier, tri, cachés): la vue repart de zéro
    fn rebuild(mut self: Pin<&mut Self>) {
        let rows = self.filtered();
        unsafe { self.as_mut().begin_reset_model() };
        self.as_mut().rust_mut().rows = rows;
        self.as_mut().set_status();
        unsafe { self.as_mut().end_reset_model() };
    }

    // Même dossier relu: seules les lignes ajoutées, retirées ou modifiées bougent, la vue garde sa position
    fn apply(mut self: Pin<&mut Self>) {
        type Key = (String, u64, i64, bool);
        let key = |e: &fs::Entry| -> Key { (e.path.clone(), e.size, e.mtime, e.is_dir) };
        let new = self.filtered();
        let new_keys: HashSet<Key> = new.iter().map(key).collect();
        let old_keys: HashSet<Key> = self.rows.iter().map(key).collect();
        let changes = self.rows.iter().filter(|e| !new_keys.contains(&key(e))).count()
            + new.iter().filter(|e| !old_keys.contains(&key(e))).count();
        if changes > 200 {
            unsafe { self.as_mut().begin_reset_model() };
            self.as_mut().rust_mut().rows = new;
            unsafe { self.as_mut().end_reset_model() };
        } else {
            let none = QModelIndex::default();
            for i in (0..self.rows.len()).rev() {
                if !new_keys.contains(&key(&self.rows[i])) {
                    unsafe { self.as_mut().begin_remove_rows(&none, i as i32, i as i32) };
                    self.as_mut().rust_mut().rows.remove(i);
                    unsafe { self.as_mut().end_remove_rows() };
                }
            }
            // Les lignes restantes sont dans l'ordre de new: insérer les manquantes à leur place
            for (i, e) in new.iter().enumerate() {
                if self.rows.get(i).is_none_or(|o| key(o) != key(e)) {
                    unsafe { self.as_mut().begin_insert_rows(&none, i as i32, i as i32) };
                    self.as_mut().rust_mut().rows.insert(i, e.clone());
                    unsafe { self.as_mut().end_insert_rows() };
                }
            }
        }
        self.as_mut().set_status();
        self.as_mut().changed();
        self.as_mut().queue_thumbs();
    }

    // Génère en arrière-plan les miniatures PDF manquantes du dossier (40 au plus, une série à la fois)
    fn queue_thumbs(self: Pin<&mut Self>) {
        let todo: Vec<(String, PathBuf)> = self
            .rows
            .iter()
            .filter(|e| icons::is_pdf(e))
            .map(|e| (e.path.clone(), thumb_path(e)))
            .filter(|(_, t)| !t.exists())
            .take(40)
            .collect();
        if todo.is_empty() || THUMBING.swap(true, Ordering::SeqCst) {
            return;
        }
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            for (src, out) in todo {
                let Some(dir) = out.parent() else { continue };
                let _ = std::fs::create_dir_all(dir);
                let stem = out.with_extension("");
                let ok = Command::new("pdftoppm")
                    .args(["-png", "-f", "1", "-l", "1", "-scale-to", "96", "-singlefile", &src])
                    .arg(&stem)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .is_ok_and(|s| s.success());
                if ok {
                    let _ = qt.queue(move |mut m| m.as_mut().thumb_ready(&QString::from(&src)));
                }
            }
            THUMBING.store(false, Ordering::SeqCst);
        });
    }

    // Relit le dossier affiché sur place; false si on ne peut plus le lire
    fn reload_rows(mut self: Pin<&mut Self>) -> bool {
        match fs::list(Path::new(&self.dir)) {
            Ok(all) => {
                self.as_mut().rust_mut().all = all;
                self.as_mut().apply();
                true
            }
            Err(_) => false,
        }
    }

    // Changement vu par inotify: relecture, ou, si le dossier a disparu, le plus proche dossier qui existe
    fn live_reload(mut self: Pin<&mut Self>) {
        if self.dir.is_empty() {
            return;
        }
        if !self.as_mut().reload_rows() {
            let mut p = PathBuf::from(&self.dir);
            while !p.is_dir() && p.pop() {}
            self.as_mut().navigate(p, None);
        }
        self.as_mut().reloaded();
    }

    // Charge dir; select: nom à sélectionner. Échec: le dossier affiché ne change pas, l'erreur va dans la barre d'état
    fn load(mut self: Pin<&mut Self>, dir: &Path, select: Option<&str>) -> Result<i32, ()> {
        match fs::list(dir) {
            Ok(all) => {
                let dir = dir.to_string_lossy().into_owned();
                let mut r = self.as_mut().rust_mut();
                r.all = all;
                r.path = QString::from(&dir);
                r.in_trash = Path::new(&dir) == ops::trash_files();
                if let Some(w) = &r.watcher {
                    w.watch(Path::new(&dir));
                }
                r.dir = dir;
                r.error = QString::default();
                r.filter = QString::default();
                self.as_mut().rebuild();
                self.as_mut().queue_thumbs();
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

    pub fn set_filter(mut self: Pin<&mut Self>, text: &QString) {
        self.as_mut().rust_mut().filter = text.clone();
        self.as_mut().rebuild();
        self.as_mut().changed();
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

    pub fn path_at(&self, row: i32) -> QString {
        QString::from(self.row(row).map_or("", |e| e.path.as_str()))
    }

    pub fn find_name(&self, name: &QString) -> i32 {
        let n = name.to_string();
        self.rows.iter().position(|e| e.name == n).map_or(-1, |i| i as i32)
    }

    pub fn has(&self, path: &QString) -> bool {
        let p = path.to_string();
        self.rows.iter().any(|e| e.path == p)
    }

    pub fn describe(&self, paths: &QString) -> QString {
        let list = paths_of(paths);
        let entries: Vec<&fs::Entry> = list.iter().filter_map(|p| self.rows.iter().find(|e| e.path == *p)).collect();
        let text = match entries.as_slice() {
            [] => return QString::default(),
            [e] => {
                let what = if e.is_dir {
                    let n = std::fs::read_dir(&e.path).map_or(0, |d| d.count());
                    fs::count(n)
                } else {
                    fs::size(e.size)
                };
                i18n::trf("%1 — %2", &[&e.name, &what])
            }
            many => {
                let total: u64 = many.iter().filter(|e| !e.is_dir).map(|e| e.size).sum();
                let head = i18n::trf("%1 of %2 selected", &[&many.len(), &self.rows.len()]);
                if total > 0 { i18n::trf("%1, %2", &[&head, &fs::size(total)]) } else { head }
            }
        };
        QString::from(&text)
    }

    pub fn path_is_dir(&self, path: &QString) -> bool {
        let p = path.to_string();
        self.rows.iter().any(|e| e.path == p && e.is_dir)
    }

    pub fn uris(&self, paths: &QString) -> QString {
        let list: Vec<String> = paths_of(paths).iter().map(|p| format!("file://{}", ops::encode(p))).collect();
        QString::from(&list.join("\r\n"))
    }

    fn fail(mut self: Pin<&mut Self>, msg: String) {
        self.as_mut().rust_mut().error = QString::from(&msg);
        self.as_mut().changed();
    }

    fn push_undo(mut self: Pin<&mut Self>, u: Undo) {
        let mut r = self.as_mut().rust_mut();
        r.undo.push(u);
        if r.undo.len() > 20 {
            r.undo.remove(0);
        }
        r.can_undo = true;
    }

    // Après une opération synchrone: relire le dossier; rend la ligne portant name, sinon la plus proche de near
    fn settle(mut self: Pin<&mut Self>, name: Option<&str>, near: i32) -> i32 {
        self.as_mut().reload_rows();
        self.as_mut().changed();
        match name.map(|n| self.find_name(&QString::from(n))) {
            Some(r) if r >= 0 => r,
            _ if self.rows.is_empty() => -1,
            _ => near.clamp(0, self.rows.len() as i32 - 1),
        }
    }

    pub fn copy(mut self: Pin<&mut Self>, paths: &QString) {
        let list = paths_of(paths);
        let mut r = self.as_mut().rust_mut();
        r.can_paste = !list.is_empty();
        r.clipboard = list;
        r.clip_cut = false;
        self.as_mut().changed();
    }

    pub fn cut(mut self: Pin<&mut Self>, paths: &QString) {
        if self.in_trash {
            return self.fail(i18n::tr("Items in the Trash cannot be cut").into());
        }
        self.as_mut().copy(paths);
        self.as_mut().rust_mut().clip_cut = true;
    }

    pub fn paste(mut self: Pin<&mut Self>) {
        if self.in_trash {
            return self.fail(i18n::tr("Nothing can be pasted into the Trash").into());
        }
        let (items, dir) = (self.clipboard.clone(), self.dir.clone());
        let mode = if self.clip_cut { Mode::Move } else { Mode::Copy };
        if self.clip_cut {
            let mut r = self.as_mut().rust_mut();
            r.clipboard.clear();
            r.can_paste = false;
        }
        self.start_job(items, dir, mode);
    }

    // Glisser-déposer: copie si demandé, sinon déplacement sur le même volume et copie sur un autre. Sur la
    // corbeille: mise à la corbeille. Depuis la corbeille: copie.
    pub fn drop_items(mut self: Pin<&mut Self>, urls: &QString, dest: &QString, copy: bool) {
        let items: Vec<String> =
            paths_of(&QString::from(&urls.to_string().replace('\r', ""))).iter().map(|u| normalize(u, "/").to_string_lossy().into_owned()).collect();
        let dest = normalize(&dest.to_string(), "/");
        if dest == ops::trash_files() {
            let list = QString::from(&items.join("\n"));
            self.as_mut().trash(&list, 0);
            return;
        }
        if !dest.is_dir() {
            return;
        }
        let mode = if copy || self.in_trash { Mode::Copy } else { Mode::Auto };
        self.start_job(items, dest.to_string_lossy().into_owned(), mode);
    }

    // Copie ou déplacement en arrière-plan: avancement dans la barre d'état, annulation (Échap) qui retire la copie
    // en cours; ce qui est déjà fini reste (et Ctrl+Z le défait).
    fn start_job(mut self: Pin<&mut Self>, items: Vec<String>, dest: String, mode: Mode) {
        if items.is_empty() {
            return;
        }
        if self.busy {
            return self.fail(i18n::tr("Another operation is in progress").into());
        }
        let what = if items.len() == 1 { base(&items[0]) } else { fs::count(items.len()) };
        let label = if mode == Mode::Copy { i18n::trf("Copying %1", &[&what]) } else { i18n::trf("Moving %1", &[&what]) };
        let cancel = Arc::new(AtomicBool::new(false));
        {
            let mut r = self.as_mut().rust_mut();
            r.busy = true;
            r.progress = -1.0;
            r.job = QString::from(&label);
            r.error = QString::default();
            r.cancel = cancel.clone();
        }
        self.as_mut().changed();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let dest = PathBuf::from(dest);
            let total = items.iter().map(|p| ops::tree_size(Path::new(p))).sum::<u64>().max(1);
            let (mut done, mut last) = (0u64, Instant::now());
            let (mut moved, mut created, mut error) = (Vec::new(), Vec::new(), None);
            for item in &items {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                let src = Path::new(item);
                let parent = src.parent().unwrap_or(Path::new("/"));
                let as_move = match mode {
                    Mode::Copy => false,
                    Mode::Move => true,
                    Mode::Auto => ops::same_volume(parent, &dest),
                };
                if as_move && parent == dest {
                    continue;
                }
                if dest.starts_with(src) {
                    error = Some(i18n::trf("Cannot put “%1” inside itself", &[&base(item)]));
                    break;
                }
                let mut progress = |n: u64| {
                    done += n;
                    if last.elapsed().as_millis() > 100 {
                        last = Instant::now();
                        let f = done as f64 / total as f64;
                        let _ = qt.queue(move |mut m| {
                            m.as_mut().rust_mut().progress = f;
                            m.as_mut().changed();
                        });
                    }
                    !cancel.load(Ordering::Relaxed)
                };
                let r = if as_move { ops::move_into(src, &dest, &mut progress) } else { ops::copy_into(src, &dest, &mut progress) };
                match r {
                    Ok(dst) if as_move => moved.push((src.to_path_buf(), dst)),
                    Ok(dst) => created.push(dst),
                    Err(e) if ops::is_cancel(&e) => break,
                    Err(e) => {
                        let (name, why) = (base(item), fs::error(&e));
                        error = Some(if as_move {
                            i18n::trf("Cannot move “%1”: %2", &[&name, &why])
                        } else {
                            i18n::trf("Cannot copy “%1”: %2", &[&name, &why])
                        });
                        break;
                    }
                }
            }
            let _ = qt.queue(move |mut m| {
                {
                    let mut r = m.as_mut().rust_mut();
                    r.busy = false;
                    r.progress = 0.0;
                    r.job = QString::default();
                    r.error = QString::from(&error.unwrap_or_default());
                }
                if !moved.is_empty() {
                    m.as_mut().push_undo(Undo::Moved(moved));
                }
                if !created.is_empty() {
                    m.as_mut().push_undo(Undo::Created(created));
                }
                m.as_mut().reload_rows();
                m.as_mut().changed();
                m.as_mut().reloaded();
            });
        });
    }

    pub fn open_with_list(&self, path: &QString) -> QString {
        let path = path.to_string();
        let list = icons::mime_type(&base(&path)).map(|m| apps::for_mime(&m)).unwrap_or_default();
        let body: Vec<String> = list.iter().map(|a| format!("{{\"id\":{},\"name\":{}}}", json(&a.id), json(&a.name))).collect();
        QString::from(&format!("[{}]", body.join(",")))
    }

    pub fn open_with(&self, id: &QString, path: &QString) {
        let (id, path) = (id.to_string(), path.to_string());
        let Some(app) = apps::desktop(&id) else { return };
        let argv = apps::command(&app, &path);
        let Some((program, args)) = argv.split_first() else { return };
        let child = Command::new(program)
            .args(args)
            .current_dir(&self.dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn();
        if let Ok(mut child) = child {
            std::thread::spawn(move || child.wait());
        }
    }

    pub fn all_archives(&self, paths: &QString) -> bool {
        let list = paths_of(paths);
        !list.is_empty() && list.iter().all(|p| archive_stem(&base(p)).is_some())
    }

    pub fn extract(self: Pin<&mut Self>, paths: &QString) {
        self.start_archive(paths_of(paths), false);
    }

    pub fn compress(self: Pin<&mut Self>, paths: &QString) {
        self.start_archive(paths_of(paths), true);
    }

    // Extraction (tar, 7z) ou compression (zip) en arrière-plan, annulable; le résultat se défait (corbeille)
    fn start_archive(mut self: Pin<&mut Self>, items: Vec<String>, compress: bool) {
        if items.is_empty() {
            return;
        }
        if self.busy {
            return self.fail(i18n::tr("Another operation is in progress").into());
        }
        if self.in_trash {
            return self.fail(i18n::tr("Nothing can be created in the Trash").into());
        }
        let what = if items.len() == 1 { base(&items[0]) } else { fs::count(items.len()) };
        let label = if compress { i18n::trf("Compressing %1", &[&what]) } else { i18n::trf("Extracting %1", &[&what]) };
        let cancel = Arc::new(AtomicBool::new(false));
        {
            let mut r = self.as_mut().rust_mut();
            r.busy = true;
            r.progress = -1.0;
            r.job = QString::from(&label);
            r.error = QString::default();
            r.cancel = cancel.clone();
        }
        self.as_mut().changed();
        let dir = PathBuf::from(&self.dir);
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let msg = |name: &str, why: &str| {
                if compress {
                    i18n::trf("Cannot compress “%1”: %2", &[&name, &why])
                } else {
                    i18n::trf("Cannot extract “%1”: %2", &[&name, &why])
                }
            };
            let (mut created, mut error): (Vec<PathBuf>, Option<String>) = (Vec::new(), None);
            // Jobs: (programme, arguments, dossier ou fichier créé, nom pour les erreurs)
            let mut jobs: Vec<(&str, Vec<String>, PathBuf, String)> = Vec::new();
            if compress {
                let first = base(&items[0]);
                let name = if items.len() == 1 { first } else { i18n::tr("Archive").to_string() };
                let out = ops::unique(&dir, &format!("{name}.zip"), "", false);
                let mut args = vec!["-r".to_string(), "-q".to_string(), out.to_string_lossy().into_owned()];
                args.extend(items.iter().map(|p| base(p)));
                jobs.push(("zip", args, out, name));
            } else {
                for item in &items {
                    let name = base(item);
                    let Some(stem) = archive_stem(&name) else { continue };
                    let out = ops::unique(&dir, &stem, "", true);
                    let out_s = out.to_string_lossy().into_owned();
                    let (program, args) = if name.contains(".tar") || name.ends_with(".tgz") {
                        ("tar", vec!["-xf".to_string(), item.clone(), "-C".to_string(), out_s])
                    } else {
                        ("7z", vec!["x".to_string(), "-y".to_string(), format!("-o{out_s}"), item.clone()])
                    };
                    if !compress && std::fs::create_dir(&out).is_err() {
                        error = Some(msg(&name, i18n::tr("permission denied")));
                        break;
                    }
                    jobs.push((program, args, out, name));
                }
            }
            for (program, args, out, name) in jobs {
                let child = Command::new(program)
                    .args(&args)
                    .current_dir(&dir)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
                let mut child = match child {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = ops::remove_tree(&out);
                        error = Some(msg(&name, &fs::error(&e)));
                        break;
                    }
                };
                let ok = loop {
                    if cancel.load(Ordering::Relaxed) {
                        let _ = child.kill();
                        let _ = child.wait();
                        break None;
                    }
                    match child.try_wait() {
                        Ok(Some(status)) => break Some(status.success()),
                        Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                        Err(_) => break Some(false),
                    }
                };
                match ok {
                    Some(true) => created.push(out),
                    Some(false) => {
                        let _ = ops::remove_tree(&out);
                        let why = i18n::tr("the tool reported an error");
                        error = Some(msg(&name, why));
                        break;
                    }
                    None => {
                        let _ = ops::remove_tree(&out);
                        break;
                    }
                }
            }
            let _ = qt.queue(move |mut m| {
                {
                    let mut r = m.as_mut().rust_mut();
                    r.busy = false;
                    r.progress = 0.0;
                    r.job = QString::default();
                    r.error = QString::from(&error.unwrap_or_default());
                }
                if !created.is_empty() {
                    m.as_mut().push_undo(Undo::Created(created));
                }
                m.as_mut().reload_rows();
                m.as_mut().changed();
                m.as_mut().reloaded();
            });
        });
    }

    pub fn cancel(self: Pin<&mut Self>) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn trash(mut self: Pin<&mut Self>, paths: &QString, row: i32) -> i32 {
        if self.in_trash {
            self.as_mut().fail(i18n::tr("These items are already in the Trash").into());
            return -1;
        }
        self.as_mut().rust_mut().error = QString::default();
        let (mut done, mut error) = (Vec::new(), None);
        for p in paths_of(paths) {
            match ops::trash(Path::new(&p)) {
                Ok(t) => done.push(t),
                Err(e) => {
                    error = Some(i18n::trf("Cannot move “%1” to the Trash: %2", &[&base(&p), &fs::error(&e)]));
                    break;
                }
            }
        }
        if !done.is_empty() {
            self.as_mut().push_undo(Undo::Trash(done));
        }
        let row = self.as_mut().settle(None, row);
        if let Some(msg) = error {
            self.as_mut().fail(msg);
        }
        row
    }

    pub fn put_back(mut self: Pin<&mut Self>, paths: &QString, row: i32) -> i32 {
        self.as_mut().rust_mut().error = QString::default();
        let (mut done, mut error) = (Vec::new(), None);
        for p in paths_of(paths) {
            match ops::restore(Path::new(&p)) {
                Ok(dst) => done.push(dst),
                Err(e) => {
                    error = Some(i18n::trf("Cannot put back “%1”: %2", &[&base(&p), &fs::error(&e)]));
                    break;
                }
            }
        }
        if !done.is_empty() {
            self.as_mut().push_undo(Undo::Created(done));
        }
        let row = self.as_mut().settle(None, row);
        if let Some(msg) = error {
            self.as_mut().fail(msg);
        }
        row
    }

    pub fn empty_trash(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().error = QString::default();
        let result = ops::empty_trash();
        {
            // Plus rien à défaire dans la corbeille
            let mut r = self.as_mut().rust_mut();
            r.undo.retain(|u| !matches!(u, Undo::Trash(_)));
            r.can_undo = !r.undo.is_empty();
        }
        self.as_mut().settle(None, 0);
        if let Err(e) = result {
            self.as_mut().fail(i18n::trf("Cannot empty the Trash: %1", &[&fs::error(&e)]));
        }
    }

    pub fn rename(mut self: Pin<&mut Self>, path: &QString, name: &QString) -> i32 {
        let (path, name) = (path.to_string(), name.to_string());
        self.as_mut().rust_mut().error = QString::default();
        if self.in_trash {
            self.as_mut().fail(i18n::tr("Items in the Trash cannot be renamed").into());
            return -1;
        }
        match ops::rename(Path::new(&path), &name) {
            Ok(dst) => {
                if dst != Path::new(&path) {
                    self.as_mut().push_undo(Undo::Rename(PathBuf::from(&path), dst));
                }
                self.as_mut().settle(Some(&name), 0)
            }
            Err(e) => {
                let msg = match e.kind() {
                    std::io::ErrorKind::AlreadyExists => i18n::trf("“%1” already exists", &[&name]),
                    std::io::ErrorKind::InvalidInput => i18n::tr("This name is not valid").into(),
                    _ => i18n::trf("Cannot rename “%1”: %2", &[&base(&path), &fs::error(&e)]),
                };
                self.as_mut().fail(msg);
                -1
            }
        }
    }

    pub fn new_folder(mut self: Pin<&mut Self>) -> i32 {
        self.as_mut().rust_mut().error = QString::default();
        if self.in_trash {
            self.as_mut().fail(i18n::tr("Nothing can be created in the Trash").into());
            return -1;
        }
        match ops::new_folder(Path::new(&self.dir)) {
            Ok(p) => {
                self.as_mut().push_undo(Undo::Created(vec![p.clone()]));
                let name = base(&p.to_string_lossy());
                self.as_mut().settle(Some(&name), 0)
            }
            Err(e) => {
                let dir = base(&self.dir);
                self.as_mut().fail(i18n::trf("Cannot create a folder in “%1”: %2", &[&dir, &fs::error(&e)]));
                -1
            }
        }
    }

    pub fn undo(mut self: Pin<&mut Self>) {
        let Some(u) = self.as_mut().rust_mut().undo.pop() else { return };
        self.as_mut().rust_mut().can_undo = !self.undo.is_empty();
        self.as_mut().rust_mut().error = QString::default();
        let result = match u {
            Undo::Rename(old, new) => {
                if ops::exists(&old) { Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists)) } else { std::fs::rename(&new, &old) }
            }
            Undo::Trash(items) => items.iter().rev().try_for_each(|t| ops::restore(&t.trashed).map(|_| ())),
            Undo::Created(paths) => paths.iter().try_for_each(|p| ops::trash(p).map(|_| ())),
            Undo::Moved(pairs) => pairs.iter().rev().try_for_each(|(src, dst)| {
                ops::move_into(dst, src.parent().unwrap_or(Path::new("/")), &mut |_| true).map(|_| ())
            }),
        };
        self.as_mut().settle(None, 0);
        self.as_mut().reloaded();
        if let Err(e) = result {
            self.as_mut().fail(i18n::trf("Cannot undo: %1", &[&fs::error(&e)]));
        }
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
            ROLE_THUMB => QVariant::from(&QString::from(&if icons::is_pdf(e) { format!("file://{}", thumb_path(e).display()) } else { String::new() })),
            ROLE_THUMB_READY => QVariant::from(&(icons::is_pdf(e) && thumb_path(e).exists())),
            ROLE_IMAGE => QVariant::from(&icons::is_image(e)),
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
            (ROLE_THUMB, "thumb"),
            (ROLE_THUMB_READY, "thumbReady"),
            (ROLE_IMAGE, "isImage"),
        ] {
            roles.insert(role, QByteArray::from(name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rows.len() as i32
    }
}

impl cxx_qt::Initialize for FolderModel {
    fn initialize(mut self: Pin<&mut Self>) {
        let qt = self.qt_thread();
        self.as_mut().rust_mut().watcher = Watcher::new(move || {
            let _ = qt.queue(|m| m.live_reload());
        });
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
        // Montages: /proc/self/mountinfo signale chaque changement par POLLPRI
        let mounts = self.qt_thread();
        std::thread::spawn(move || {
            use std::{io::{Read, Seek}, os::fd::AsRawFd};
            let Ok(mut file) = std::fs::File::open("/proc/self/mountinfo") else { return };
            let mut pfd = libc::pollfd { fd: file.as_raw_fd(), events: libc::POLLPRI, revents: 0 };
            // Le changement est acquitté par une lecture complète du même descripteur
            let mut table = String::new();
            let mut ack = |file: &mut std::fs::File| {
                table.clear();
                let _ = file.rewind();
                let _ = file.read_to_string(&mut table);
            };
            ack(&mut file);
            loop {
                if unsafe { libc::poll(&mut pfd, 1, -1) } < 0 {
                    return;
                }
                // Laisser les montages en rafale finir, puis relire la table (le changement est acquitté par la lecture)
                std::thread::sleep(std::time::Duration::from_millis(300));
                ack(&mut file);
                if mounts.queue(|mut f| f.as_mut().places_changed()).is_err() {
                    return;
                }
            }
        });
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
    // Monte ou démonte en arrière-plan (udisksctl): le QML est prévenu par mounted / volumeError, puis placesChanged
    pub fn mount_volume(self: Pin<&mut Self>, dev: &QString) {
        let dev = dev.to_string();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let r = volumes::mount(&dev);
            let _ = qt.queue(move |mut f| match r {
                Ok(path) => {
                    f.as_mut().places_changed();
                    f.as_mut().mounted(&QString::from(&path));
                }
                Err(e) => {
                    let msg = i18n::trf("Cannot mount “%1”: %2", &[&dev, &e]);
                    f.as_mut().volume_error(&QString::from(&msg));
                }
            });
        });
    }

    pub fn unmount_volume(self: Pin<&mut Self>, dev: &QString) {
        let dev = dev.to_string();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let r = volumes::unmount(&dev);
            let _ = qt.queue(move |mut f| match r {
                Ok(()) => f.as_mut().places_changed(),
                Err(e) => {
                    let msg = i18n::trf("Cannot eject “%1”: %2", &[&dev, &e]);
                    f.as_mut().volume_error(&QString::from(&msg));
                }
            });
        });
    }

    pub fn add_favorite(mut self: Pin<&mut Self>, path: &QString) {
        let p = normalize(&path.to_string(), "/").to_string_lossy().into_owned();
        let mut list = bookmarks();
        if !Path::new(&p).is_dir() || default_favorites().iter().any(|(d, _)| *d == p) || list.iter().any(|(b, _)| *b == p) {
            return;
        }
        list.push((p, String::new()));
        if write_bookmarks(&list).is_ok() {
            self.as_mut().places_changed();
        }
    }

    pub fn remove_favorite(mut self: Pin<&mut Self>, path: &QString) {
        let p = path.to_string();
        let list: Vec<(String, String)> = bookmarks().into_iter().filter(|(b, _)| *b != p).collect();
        if write_bookmarks(&list).is_ok() {
            self.as_mut().places_changed();
        }
    }

    pub fn is_favorite(&self, path: &QString) -> bool {
        let p = path.to_string();
        default_favorites().iter().any(|(d, _)| *d == p) || bookmarks().iter().any(|(b, _)| *b == p)
    }

    // Presse-papiers Wayland par wl-copy (qui reste en arrière-plan tant que le texte est la sélection)
    pub fn copy_text(&self, text: &QString) {
        let text = text.to_string();
        let child = Command::new("wl-copy").arg("--").arg(text).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
        if let Ok(mut child) = child {
            std::thread::spawn(move || child.wait());
        }
    }

    pub fn tr(&self, text: &QString) -> QString {
        QString::from(i18n::tr(&text.to_string()))
    }

    pub fn places(&self) -> QString {
        let home = home();
        let (places, devices) = (i18n::tr("Favorites"), i18n::tr("Locations"));
        let name = |p: &str| Path::new(p).file_name().map_or(p.to_string(), |n| n.to_string_lossy().into_owned());
        // (section, nom, chemin, nom d'icône, volume)
        let mut items: Vec<(&str, String, String, &str, String)> = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        for (p, icon) in default_favorites() {
            let label = if p == home { name(&home) } else { name(&p) };
            items.push((places, label, p.clone(), icon, String::new()));
            seen.push(p);
        }
        // Favoris ajoutés par l'utilisateur (retirables)
        let mut removable: Vec<String> = Vec::new();
        for (p, label) in bookmarks() {
            if seen.contains(&p) || !Path::new(&p).is_dir() {
                continue;
            }
            let label = if label.is_empty() { name(&p) } else { label };
            let icon = if label.to_lowercase().contains("icloud") { "folder-cloud" } else { "folder" };
            items.push((places, label, p.clone(), icon, String::new()));
            removable.push(p.clone());
            seen.push(p);
        }
        items.push((devices, i18n::tr("Computer").to_string(), "/".into(), "computer", String::new()));
        // Volumes amovibles: non montés, un clic les monte (path vide, dev renseigné)
        for v in volumes::list() {
            items.push((devices, v.name, v.mount.unwrap_or_default(), "drive-removable-media", v.dev));
        }
        let _ = ops::ensure_trash(&ops::home_trash());
        let trash = if ops::trash_is_empty() { "user-trash" } else { "user-trash-full" };
        items.push((devices, i18n::tr("Trash").to_string(), ops::trash_files().to_string_lossy().into_owned(), trash, String::new()));
        let body: Vec<String> = items
            .iter()
            .map(|(s, n, p, i, dev)| {
                format!(
                    "{{\"section\":{},\"name\":{},\"path\":{},\"dev\":{},\"removable\":{},\"mono\":{},\"monoLight\":{}}}",
                    json(s),
                    json(n),
                    json(p),
                    json(dev),
                    removable.contains(p),
                    json(&icons::symbolic(i, true)),
                    json(&icons::symbolic(i, false))
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
    fn archive_names() {
        assert_eq!(archive_stem("photos.TAR.GZ").as_deref(), Some("photos"));
        assert_eq!(archive_stem("a.b.zip").as_deref(), Some("a.b"));
        assert_eq!(archive_stem(".zip"), None);
        assert_eq!(archive_stem("notes.txt"), None);
    }

    #[test]
    fn decodes_percent() {
        assert_eq!(percent_decode("caf%C3%A9%2"), "café%2");
    }
}
