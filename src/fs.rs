// Lecture d'un dossier, tri et mise en forme des colonnes (taille, date). Sans Qt: testé par cargo test.
use crate::i18n::{num, tr, trf};
use std::{
    cmp::Ordering,
    fs, io,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
};

#[derive(Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub mtime: i64,
    pub exec: bool,
}

impl Entry {
    pub fn hidden(&self) -> bool {
        self.name.starts_with('.')
    }
}

pub fn list(dir: &Path) -> io::Result<Vec<Entry>> {
    let mut out = Vec::new();
    for e in fs::read_dir(dir)? {
        let Ok(e) = e else { continue };
        let path = e.path();
        // Un lien prend le type de sa cible (lien vers un dossier = dossier); lien cassé: le lien lui-même
        let Ok(m) = fs::metadata(&path).or_else(|_| fs::symlink_metadata(&path)) else { continue };
        out.push(Entry {
            name: e.file_name().to_string_lossy().into_owned(),
            path: path.to_string_lossy().into_owned(),
            is_dir: m.is_dir(),
            size: m.len(),
            mtime: m.mtime(),
            exec: m.is_file() && m.permissions().mode() & 0o111 != 0,
        });
    }
    Ok(out)
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Key {
    Name,
    Size,
    Date,
}

impl Key {
    pub fn from(i: i32) -> Key {
        match i {
            1 => Key::Size,
            2 => Key::Date,
            _ => Key::Name,
        }
    }
}

// Dossiers d'abord dans tous les ordres (Finder); les dossiers n'ont pas de taille utile, triés par nom
pub fn sort(v: &mut [Entry], key: Key, desc: bool) {
    v.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then_with(|| {
            let o = match key {
                Key::Size if !a.is_dir => a.size.cmp(&b.size),
                Key::Date => a.mtime.cmp(&b.mtime),
                _ => Ordering::Equal,
            }
            .then_with(|| natural(&a.name, &b.name));
            if desc { o.reverse() } else { o }
        })
    });
}

// Ordre naturel: sans casse ni accents, nombres comparés par valeur (« photo 2 » avant « photo 10 »)
pub fn natural(a: &str, b: &str) -> Ordering {
    let (mut x, mut y) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (x.peek().copied(), y.peek().copied()) {
            (None, None) => return a.cmp(b),
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(c), Some(d)) if c.is_ascii_digit() && d.is_ascii_digit() => {
                let (n, m) = (digits(&mut x), digits(&mut y));
                let (n, m) = (n.trim_start_matches('0'), m.trim_start_matches('0'));
                let o = n.len().cmp(&m.len()).then_with(|| n.cmp(m));
                if o != Ordering::Equal {
                    return o;
                }
            }
            (Some(c), Some(d)) => {
                let o = fold(c).cmp(&fold(d));
                if o != Ordering::Equal {
                    return o;
                }
                x.next();
                y.next();
            }
        }
    }
}

// Le nom commence par prefix, sans casse ni accents (saisie au clavier dans la liste)
pub fn starts_with(name: &str, prefix: &str) -> bool {
    let mut n = name.chars().map(fold);
    prefix.chars().map(fold).all(|c| n.next() == Some(c))
}

fn digits(it: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut s = String::new();
    while let Some(c) = it.next_if(char::is_ascii_digit) {
        s.push(c);
    }
    s
}

fn fold(c: char) -> char {
    match c.to_lowercase().next().unwrap_or(c) {
        'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => 'a',
        'ç' => 'c',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ñ' => 'n',
        'ó' | 'ò' | 'ô' | 'ö' | 'õ' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ÿ' => 'y',
        l => l,
    }
}

// Tailles en unités binaires, comme le fili-shell (Gio, Mio)
pub fn size(n: u64) -> String {
    const UNITS: [&str; 4] = ["%1 KiB", "%1 MiB", "%1 GiB", "%1 TiB"];
    if n < 1024 {
        return trf("%1 B", &[&n]);
    }
    let mut v = n as f64 / 1024.0;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    let d = if v < 10.0 { 1 } else { 0 };
    match i {
        0 => trf("%1 KiB", &[&num(v, d)]),
        1 => trf("%1 MiB", &[&num(v, d)]),
        2 => trf("%1 GiB", &[&num(v, d)]),
        _ => trf("%1 TiB", &[&num(v, d)]),
    }
}

// Heure locale (y, m, j, h, min, jour de l'année)
fn local(t: i64) -> (i32, i32, i32, i32, i32, i32) {
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let t = t as libc::time_t;
    unsafe { libc::localtime_r(&t, &mut tm) };
    (tm.tm_year + 1900, tm.tm_mon + 1, tm.tm_mday, tm.tm_hour, tm.tm_min, tm.tm_yday)
}

// « Aujourd'hui, 14:32 », « Hier, 09:10 », sinon la date dans l'ordre de la langue (catalogue)
pub fn date(t: i64, now: i64) -> String {
    let (y, mo, d, h, mi, yd) = local(t);
    let hm = format!("{h:02}:{mi:02}");
    let (ny, _, _, _, _, nyd) = local(now);
    let (py, _, _, _, _, pyd) = local(now - 86400);
    if (y, yd) == (ny, nyd) {
        trf("Today, %1", &[&hm])
    } else if (y, yd) == (py, pyd) {
        trf("Yesterday, %1", &[&hm])
    } else {
        trf("%1-%2-%3 %4", &[&y, &format!("{mo:02}"), &format!("{d:02}"), &hm])
    }
}

pub fn count(n: usize) -> String {
    if n == 1 { trf("%1 item", &[&n]) } else { trf("%1 items", &[&n]) }
}

// Espace libre du système de fichiers qui contient dir
pub fn free(dir: &Path) -> Option<u64> {
    let c = std::ffi::CString::new(dir.as_os_str().as_encoded_bytes()).ok()?;
    let mut s: libc::statvfs = unsafe { std::mem::zeroed() };
    (unsafe { libc::statvfs(c.as_ptr(), &mut s) } == 0).then(|| s.f_bavail as u64 * s.f_frsize as u64)
}

pub fn error(e: &io::Error) -> String {
    match e.kind() {
        io::ErrorKind::PermissionDenied => tr("permission denied").into(),
        io::ErrorKind::NotFound => tr("not found").into(),
        io::ErrorKind::NotADirectory => tr("not a folder").into(),
        _ => e.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(name: &str, is_dir: bool, size: u64, mtime: i64) -> Entry {
        Entry { name: name.into(), path: format!("/x/{name}"), is_dir, size, mtime, exec: false }
    }

    fn names(v: &[Entry]) -> Vec<&str> {
        v.iter().map(|e| e.name.as_str()).collect()
    }

    #[test]
    fn natural_order() {
        let mut v = vec!["photo 10", "Photo 2", "école", "Zèbre", "a", "photo 02b"];
        v.sort_by(|a, b| natural(a, b));
        assert_eq!(v, ["a", "école", "Photo 2", "photo 02b", "photo 10", "Zèbre"]);
    }

    #[test]
    fn folders_first_in_every_order() {
        let mut v = vec![e("b.txt", false, 5, 3), e("Z", true, 4096, 1), e("a.txt", false, 50, 2), e("c", true, 0, 9)];
        sort(&mut v, Key::Name, false);
        assert_eq!(names(&v), ["c", "Z", "a.txt", "b.txt"]);
        sort(&mut v, Key::Name, true);
        assert_eq!(names(&v), ["Z", "c", "b.txt", "a.txt"]);
        sort(&mut v, Key::Size, true);
        assert_eq!(names(&v), ["Z", "c", "a.txt", "b.txt"]);
        sort(&mut v, Key::Date, false);
        assert_eq!(names(&v), ["Z", "c", "a.txt", "b.txt"]);
    }

    #[test]
    fn sizes() {
        // Textes en anglais quelle que soit la langue de la session (le catalogue est lu une fois)
        unsafe { std::env::set_var("FILI_LANG", "en") };
        assert_eq!(size(512), "512 B");
        assert_eq!(size(1536), "1.5 KiB");
        assert_eq!(size(20 * 1024 * 1024), "20 MiB");
    }

    #[test]
    fn lists_a_folder() {
        let dir = std::env::temp_dir().join(format!("fili-file-test-{}", std::process::id()));
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join(".cache"), "x").unwrap();
        std::os::unix::fs::symlink(dir.join("sub"), dir.join("lien")).unwrap();
        std::os::unix::fs::symlink(dir.join("absent"), dir.join("cassé")).unwrap();
        let mut v = list(&dir).unwrap();
        sort(&mut v, Key::Name, false);
        assert_eq!(names(&v), ["lien", "sub", ".cache", "cassé"]);
        assert!(v[0].is_dir && v[2].hidden());
        fs::remove_dir_all(&dir).unwrap();
    }
}
