// Opérations sur les fichiers: copie, déplacement, renommage, nouveau dossier, corbeille freedesktop. Sans Qt: testé
// par cargo test. Jamais d'écrasement: un nom pris reçoit un suffixe (« nom copie », « nom copie 2 »).
use crate::i18n::tr;
use std::{
    fs,
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};

// Retour false: l'utilisateur annule
pub type Progress<'a> = &'a mut dyn FnMut(u64) -> bool;

fn cancelled() -> io::Error {
    io::Error::new(io::ErrorKind::Interrupted, "cancelled")
}

pub fn is_cancel(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::Interrupted
}

pub fn exists(p: &Path) -> bool {
    fs::symlink_metadata(p).is_ok()
}

// Premier nom libre dans dir: « stem suffix.ext », « stem suffix 2.ext »... (une extension de dossier n'en est pas une)
pub fn unique(dir: &Path, name: &str, suffix: &str, is_dir: bool) -> PathBuf {
    let first = dir.join(name);
    if !exists(&first) {
        return first;
    }
    let (stem, ext) = match name.rfind('.') {
        Some(i) if i > 0 && !is_dir => (&name[..i], &name[i..]),
        _ => (name, ""),
    };
    let tag = if suffix.is_empty() { String::new() } else { format!(" {suffix}") };
    let mut n = 1;
    loop {
        let num = if n == 1 { String::new() } else { format!(" {n}") };
        let p = dir.join(format!("{stem}{tag}{num}{ext}"));
        if !exists(&p) {
            return p;
        }
        n += 1;
    }
}

pub fn tree_size(p: &Path) -> u64 {
    let Ok(m) = fs::symlink_metadata(p) else { return 0 };
    if !m.is_dir() {
        return m.len();
    }
    fs::read_dir(p).map_or(0, |d| d.flatten().map(|e| tree_size(&e.path())).sum())
}

fn copy_file(src: &Path, dst: &Path, progress: Progress) -> io::Result<()> {
    let mut from = fs::File::open(src)?;
    let mut to = fs::OpenOptions::new().write(true).create_new(true).open(dst)?;
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = from.read(&mut buf)?;
        if n == 0 {
            break;
        }
        to.write_all(&buf[..n])?;
        if !progress(n as u64) {
            return Err(cancelled());
        }
    }
    fs::set_permissions(dst, fs::Permissions::from_mode(from.metadata()?.permissions().mode()))
}

// Copie récursive de src vers dst (dst n'existe pas). Les liens symboliques sont recopiés tels quels.
pub fn copy_tree(src: &Path, dst: &Path, progress: Progress) -> io::Result<()> {
    let m = fs::symlink_metadata(src)?;
    if m.file_type().is_symlink() {
        std::os::unix::fs::symlink(fs::read_link(src)?, dst)
    } else if m.is_dir() {
        fs::create_dir(dst)?;
        for e in fs::read_dir(src)? {
            let e = e?;
            copy_tree(&e.path(), &dst.join(e.file_name()), progress)?;
        }
        fs::set_permissions(dst, m.permissions())
    } else if m.is_file() {
        copy_file(src, dst, progress)
    } else {
        // Tubes, sockets, périphériques: ignorés
        Ok(())
    }
}

pub fn remove_tree(p: &Path) -> io::Result<()> {
    if fs::symlink_metadata(p)?.is_dir() { fs::remove_dir_all(p) } else { fs::remove_file(p) }
}

// Copie sous un nom libre de dir; la copie interrompue ou en échec est retirée
pub fn copy_into(src: &Path, dir: &Path, progress: Progress) -> io::Result<PathBuf> {
    let name = src.file_name().ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?.to_string_lossy();
    let dst = unique(dir, &name, tr("copy"), src.is_dir());
    if let Err(e) = copy_tree(src, &dst, progress) {
        let _ = remove_tree(&dst);
        return Err(e);
    }
    Ok(dst)
}

// Déplace src dans dir (nom libre): renommage, ou copie puis suppression d'un autre volume
pub fn move_into(src: &Path, dir: &Path, progress: Progress) -> io::Result<PathBuf> {
    let name = src.file_name().ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?.to_string_lossy();
    let dst = unique(dir, &name, tr("copy"), src.is_dir());
    match fs::rename(src, &dst) {
        Ok(()) => Ok(dst),
        Err(e) if e.raw_os_error() == Some(libc::EXDEV) => {
            if let Err(e) = copy_tree(src, &dst, progress) {
                let _ = remove_tree(&dst);
                return Err(e);
            }
            remove_tree(src)?;
            Ok(dst)
        }
        Err(e) => Err(e),
    }
}

pub fn same_volume(a: &Path, b: &Path) -> bool {
    matches!((fs::metadata(a), fs::metadata(b)), (Ok(x), Ok(y)) if x.dev() == y.dev())
}

pub fn valid_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\0'])
}

// Renomme src en new_name dans son dossier; un nom pris est refusé (AlreadyExists)
pub fn rename(src: &Path, new_name: &str) -> io::Result<PathBuf> {
    if !valid_name(new_name) {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    let dst = src.parent().unwrap_or(Path::new("/")).join(new_name);
    if dst == src {
        return Ok(dst);
    }
    if exists(&dst) {
        return Err(io::Error::from(io::ErrorKind::AlreadyExists));
    }
    fs::rename(src, &dst)?;
    Ok(dst)
}

pub fn new_folder(dir: &Path) -> io::Result<PathBuf> {
    let p = unique(dir, tr("untitled folder"), "", true);
    fs::create_dir(&p)?;
    Ok(p)
}

// --- Corbeille (spécification freedesktop.org Trash 1.0) ---

pub struct Trashed {
    pub trashed: PathBuf,
}

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".into()))
}

fn data_home() -> PathBuf {
    std::env::var("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|_| home().join(".local/share"))
}

// Corbeille personnelle: dossier de base (files/ et info/)
pub fn home_trash() -> PathBuf {
    data_home().join("Trash")
}

pub fn trash_files() -> PathBuf {
    home_trash().join("files")
}

pub fn ensure_trash(base: &Path) -> io::Result<()> {
    fs::create_dir_all(base.join("files"))?;
    fs::create_dir_all(base.join("info"))
}

pub fn trash_is_empty() -> bool {
    fs::read_dir(trash_files()).is_ok_and(|mut d| d.next().is_none())
}

// Racine du volume qui contient p (premier ancêtre dont le parent est sur un autre volume)
fn topdir(p: &Path) -> PathBuf {
    let Ok(dev) = fs::metadata(p).map(|m| m.dev()) else { return PathBuf::from("/") };
    let mut top = p.to_path_buf();
    while let Some(parent) = top.parent() {
        if fs::metadata(parent).map(|m| m.dev()).ok() != Some(dev) {
            break;
        }
        top = parent.to_path_buf();
    }
    top
}

// Corbeille qui reçoit p: celle de l'utilisateur si p est sur le volume du dossier personnel, sinon .Trash-UID
// à la racine du volume. Le second rendu est la base des chemins de .trashinfo (None: chemins absolus).
fn trash_for(p: &Path) -> io::Result<(PathBuf, Option<PathBuf>)> {
    let home_trash = home_trash();
    fs::create_dir_all(&home_trash)?;
    let parent = p.parent().unwrap_or(Path::new("/"));
    if same_volume(parent, &home_trash) {
        return Ok((home_trash, None));
    }
    let top = topdir(parent);
    let base = top.join(format!(".Trash-{}", unsafe { libc::getuid() }));
    Ok((base, Some(top)))
}

pub fn encode(path: &str) -> String {
    let mut o = String::with_capacity(path.len());
    for b in path.bytes() {
        if b.is_ascii_alphanumeric() || b"/-_.~".contains(&b) {
            o.push(b as char);
        } else {
            o.push_str(&format!("%{b:02X}"));
        }
    }
    o
}

pub fn decode(s: &str) -> String {
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

fn now_iso() -> String {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs()) as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe { libc::localtime_r(&t, &mut tm) };
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min,
        tm.tm_sec
    )
}

// Met p à la corbeille (renommage: instantané, annulable)
pub fn trash(p: &Path) -> io::Result<Trashed> {
    let p = if p.is_absolute() { p.to_path_buf() } else { std::env::current_dir()?.join(p) };
    fs::symlink_metadata(&p)?;
    let (base, top) = trash_for(&p)?;
    ensure_trash(&base)?;
    let name = p.file_name().ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?.to_string_lossy();
    // Réserver le nom par son .trashinfo (création exclusive), comme le demande la spécification
    let mut n = 0;
    let (trashed, info) = loop {
        let candidate = if n == 0 { name.to_string() } else { format!("{name}.{n}") };
        let info = base.join("info").join(format!("{candidate}.trashinfo"));
        match fs::OpenOptions::new().write(true).create_new(true).open(&info) {
            Ok(f) if !exists(&base.join("files").join(&candidate)) => break (base.join("files").join(&candidate), (info, f)),
            Ok(_) => {
                let _ = fs::remove_file(&info);
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
        n += 1;
    };
    let (info, mut file) = info;
    let recorded = match &top {
        Some(top) => p.strip_prefix(top).unwrap_or(&p).to_string_lossy().into_owned(),
        None => p.to_string_lossy().into_owned(),
    };
    let text = format!("[Trash Info]\nPath={}\nDeletionDate={}\n", encode(&recorded), now_iso());
    if let Err(e) = file.write_all(text.as_bytes()).and_then(|()| fs::rename(&p, &trashed)) {
        let _ = fs::remove_file(&info);
        return Err(e);
    }
    Ok(Trashed { trashed })
}

fn info_of(trashed: &Path) -> Option<PathBuf> {
    let files = trashed.parent()?;
    let name = trashed.file_name()?.to_string_lossy();
    Some(files.parent()?.join("info").join(format!("{name}.trashinfo")))
}

// Emplacement d'origine d'un élément de la corbeille, d'après son .trashinfo
pub fn original_of(trashed: &Path) -> Option<PathBuf> {
    let text = fs::read_to_string(info_of(trashed)?).ok()?;
    let value = text.lines().find_map(|l| l.strip_prefix("Path="))?;
    let path = PathBuf::from(decode(value.trim()));
    if path.is_absolute() {
        return Some(path);
    }
    // Chemin relatif à la racine du volume (corbeille .Trash-UID)
    let base = trashed.parent()?.parent()?.parent()?;
    Some(base.join(path))
}

// Remet un élément à sa place (sous un nom libre si elle est prise); le dossier d'origine est recréé si besoin
pub fn restore(trashed: &Path) -> io::Result<PathBuf> {
    let original = original_of(trashed).ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let dir = original.parent().unwrap_or(Path::new("/"));
    fs::create_dir_all(dir)?;
    let name = original.file_name().unwrap_or_default().to_string_lossy();
    let dst = unique(dir, &name, tr("restored"), trashed.is_dir());
    fs::rename(trashed, &dst)?;
    if let Some(info) = info_of(trashed) {
        let _ = fs::remove_file(info);
    }
    Ok(dst)
}

pub fn empty_trash() -> io::Result<()> {
    let base = home_trash();
    for sub in ["files", "info"] {
        let Ok(d) = fs::read_dir(base.join(sub)) else { continue };
        for e in d.flatten() {
            remove_tree(&e.path())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("fili-file-ops-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn all(_: u64) -> bool {
        true
    }

    #[test]
    fn unique_names() {
        unsafe { std::env::set_var("FILI_LANG", "en") };
        let d = tmp("unique");
        fs::write(d.join("a.txt"), "x").unwrap();
        fs::create_dir(d.join("v1.2")).unwrap();
        assert_eq!(unique(&d, "b.txt", "copy", false), d.join("b.txt"));
        assert_eq!(unique(&d, "a.txt", "copy", false), d.join("a copy.txt"));
        fs::write(d.join("a copy.txt"), "x").unwrap();
        assert_eq!(unique(&d, "a.txt", "copy", false), d.join("a copy 2.txt"));
        // Un dossier à point dans le nom: pas d'extension
        assert_eq!(unique(&d, "v1.2", "copy", true), d.join("v1.2 copy"));
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn copies_a_tree_without_overwriting() {
        unsafe { std::env::set_var("FILI_LANG", "en") };
        let d = tmp("copy");
        fs::create_dir_all(d.join("src/sub")).unwrap();
        fs::write(d.join("src/sub/f"), "hello").unwrap();
        std::os::unix::fs::symlink("sub/f", d.join("src/link")).unwrap();
        let mut seen = 0;
        let dst = copy_into(&d.join("src"), &d, &mut |n| {
            seen += n;
            true
        })
        .unwrap();
        assert_eq!(dst, d.join("src copy"));
        assert_eq!(fs::read_to_string(dst.join("sub/f")).unwrap(), "hello");
        assert_eq!(fs::read_link(dst.join("link")).unwrap(), PathBuf::from("sub/f"));
        assert_eq!((seen, tree_size(&d.join("src")) >= 5), (5, true));
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn cancel_removes_the_partial_copy() {
        unsafe { std::env::set_var("FILI_LANG", "en") };
        let d = tmp("cancel");
        fs::write(d.join("big"), vec![0u8; 3 << 20]).unwrap();
        let e = copy_into(&d.join("big"), &d, &mut |_| false).unwrap_err();
        assert!(is_cancel(&e));
        assert!(!exists(&d.join("big copy")));
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn moves_and_renames() {
        unsafe { std::env::set_var("FILI_LANG", "en") };
        let d = tmp("move");
        fs::create_dir(d.join("to")).unwrap();
        fs::write(d.join("f"), "1").unwrap();
        fs::write(d.join("to/f"), "2").unwrap();
        let dst = move_into(&d.join("f"), &d.join("to"), &mut all).unwrap();
        assert_eq!(dst, d.join("to/f copy"));
        assert!(!exists(&d.join("f")));
        assert_eq!(fs::read_to_string(d.join("to/f")).unwrap(), "2");
        assert!(rename(&d.join("to/f"), "g").is_ok());
        assert_eq!(rename(&d.join("to/g"), "f copy").unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(rename(&d.join("to/g"), "a/b").unwrap_err().kind(), io::ErrorKind::InvalidInput);
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn trash_and_restore() {
        unsafe { std::env::set_var("FILI_LANG", "en") };
        let d = tmp("trash");
        let data = d.join("data");
        unsafe { std::env::set_var("XDG_DATA_HOME", &data) };
        let work = d.join("work dir");
        fs::create_dir(&work).unwrap();
        fs::write(work.join("é a.txt"), "1").unwrap();
        let t = trash(&work.join("é a.txt")).unwrap();
        assert_eq!(t.trashed, data.join("Trash/files/é a.txt"));
        assert!(!exists(&work.join("é a.txt")));
        let info = fs::read_to_string(data.join("Trash/info/é a.txt.trashinfo")).unwrap();
        assert!(info.starts_with("[Trash Info]\nPath=") && info.contains("%C3%A9%20a.txt") && info.contains("DeletionDate="));
        // Même nom une seconde fois
        fs::write(work.join("é a.txt"), "2").unwrap();
        let t2 = trash(&work.join("é a.txt")).unwrap();
        assert_eq!(t2.trashed, data.join("Trash/files/é a.txt.1"));
        // Retour: le nom est repris, le second prend un suffixe
        assert_eq!(restore(&t.trashed).unwrap(), work.join("é a.txt"));
        assert_eq!(restore(&t2.trashed).unwrap(), work.join("é a restored.txt"));
        assert!(trash_is_empty());
        assert!(!exists(&data.join("Trash/info/é a.txt.trashinfo")));
        // Vider
        fs::write(work.join("x"), "").unwrap();
        trash(&work.join("x")).unwrap();
        assert!(!trash_is_empty());
        empty_trash().unwrap();
        assert!(trash_is_empty() && fs::read_dir(data.join("Trash/info")).unwrap().next().is_none());
        unsafe { std::env::remove_var("XDG_DATA_HOME") };
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn percent_round_trip() {
        assert_eq!(decode(&encode("/a b/é#%")), "/a b/é#%");
        assert_eq!(encode("/a b"), "/a%20b");
    }
}
