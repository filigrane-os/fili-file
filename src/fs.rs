// Lecture d'un dossier, tri et mise en forme des colonnes (taille, date). Sans Qt: testé par cargo test.
use crate::i18n::{num, tr, trf};
use std::{
    cmp::Ordering,
    fs, io,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub mtime: i64,
    pub exec: bool,
    // Création (date de naissance du fichier, à défaut la modification), droits (rwxr-xr-x), propriétaire, type lisible
    pub created: i64,
    pub mode: u32,
    pub uid: u32,
    pub kind: String,
}

impl Entry {
    pub fn hidden(&self) -> bool {
        self.name.starts_with('.')
    }
}

pub fn entry(path: &Path) -> Option<Entry> {
    // Un lien prend le type de sa cible (lien vers un dossier = dossier); lien cassé: le lien lui-même
    let m = fs::metadata(path).or_else(|_| fs::symlink_metadata(path)).ok()?;
    let name = path.file_name().map_or("/".into(), |n| n.to_string_lossy().into_owned());
    Some(Entry {
        path: path.to_string_lossy().into_owned(),
        is_dir: m.is_dir(),
        size: m.len(),
        mtime: m.mtime(),
        exec: m.is_file() && m.permissions().mode() & 0o111 != 0,
        created: m.created().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(m.mtime(), |d| d.as_secs() as i64),
        mode: m.permissions().mode() & 0o777,
        uid: m.uid(),
        kind: crate::icons::kind(&name, m.is_dir()),
        name,
    })
}

pub fn list(dir: &Path) -> io::Result<Vec<Entry>> {
    Ok(fs::read_dir(dir)?.filter_map(|e| entry(&e.ok()?.path())).collect())
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Key {
    Name,
    Size,
    Date,
    Kind,
    Created,
    Permissions,
    Owner,
}

impl Key {
    pub fn from(i: i32) -> Key {
        match i {
            1 => Key::Size,
            2 => Key::Date,
            3 => Key::Kind,
            4 => Key::Created,
            5 => Key::Permissions,
            6 => Key::Owner,
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
                Key::Kind => natural(&a.kind, &b.kind),
                Key::Created => a.created.cmp(&b.created),
                Key::Permissions => a.mode.cmp(&b.mode),
                Key::Owner => natural(&owner(a.uid), &owner(b.uid)),
                _ => Ordering::Equal,
            }
            .then_with(|| natural(&a.name, &b.name));
            if desc { o.reverse() } else { o }
        })
    });
}

// Nom du propriétaire (passwd), gardé en mémoire; à défaut le numéro
pub fn owner(uid: u32) -> String {
    static CACHE: std::sync::Mutex<Vec<(u32, String)>> = std::sync::Mutex::new(Vec::new());
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((_, n)) = cache.iter().find(|(u, _)| *u == uid) {
        return n.clone();
    }
    let p = unsafe { libc::getpwuid(uid) };
    let name = if p.is_null() {
        uid.to_string()
    } else {
        unsafe { std::ffi::CStr::from_ptr((*p).pw_name) }.to_string_lossy().into_owned()
    };
    cache.push((uid, name.clone()));
    name
}

// Droits à la façon de ls: rwxr-xr-x
pub fn permissions(mode: u32) -> String {
    "rwxrwxrwx".chars().enumerate().map(|(i, c)| if mode & (0o400 >> i) != 0 { c } else { '-' }).collect()
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

// Motif de sélection: « * » (suite de caractères) et « ? » (un caractère), sans casse; sans joker, le nom doit contenir le motif
pub fn glob_match(name: &str, pattern: &str) -> bool {
    let n: Vec<char> = name.to_lowercase().chars().collect();
    let p: Vec<char> = pattern.trim().to_lowercase().chars().collect();
    if p.is_empty() {
        return false;
    }
    if !p.iter().any(|c| matches!(c, '*' | '?')) {
        return name.to_lowercase().contains(&p.iter().collect::<String>());
    }
    // Retour arrière sur la dernière « * »
    let (mut i, mut j, mut star, mut mark) = (0, 0, None, 0);
    while i < n.len() {
        if j < p.len() && (p[j] == '?' || p[j] == n[i]) {
            i += 1;
            j += 1;
        } else if j < p.len() && p[j] == '*' {
            star = Some(j);
            mark = i;
            j += 1;
        } else if let Some(s) = star {
            j = s + 1;
            mark += 1;
            i = mark;
        } else {
            return false;
        }
    }
    p[j..].iter().all(|&c| c == '*')
}

#[cfg(test)]
// Le nom contient needle, sans casse ni accents (filtre du dossier)
pub fn contains(name: &str, needle: &str) -> bool {
    let n: String = name.chars().map(fold).collect();
    n.contains(&needle.chars().map(fold).collect::<String>())
}

// Recherche floue façon fzf: chaque mot de needle (séparés par des espaces) doit apparaître dans l'ordre dans le
// chemin, sans casse ni accents. Score plus haut = meilleur (lettres consécutives, débuts de mots, nom de fichier).
pub fn fuzzy(path: &str, needle: &str) -> Option<i64> {
    let hay: Vec<char> = path.chars().map(fold).collect();
    let base = hay.iter().rposition(|&c| c == '/').map_or(0, |i| i + 1);
    let mut total = 0;
    let mut any = false;
    for word in needle.split_whitespace() {
        any = true;
        let w: Vec<char> = word.chars().map(fold).collect();
        // Fin du plus court appariement vers l'avant, puis début le plus serré en revenant en arrière
        let mut j = 0;
        let mut end = 0;
        for (i, &c) in hay.iter().enumerate() {
            if c == w[j] {
                j += 1;
                if j == w.len() {
                    end = i;
                    break;
                }
            }
        }
        if j < w.len() {
            return None;
        }
        let mut pos = vec![0; w.len()];
        let mut k = w.len();
        let mut i = end + 1;
        while k > 0 {
            i -= 1;
            if hay[i] == w[k - 1] {
                k -= 1;
                pos[k] = i;
            }
        }
        let mut score = 0i64;
        for (n, &i) in pos.iter().enumerate() {
            score += 16;
            let prev = if i == 0 { '/' } else { hay[i - 1] };
            if matches!(prev, '/' | '-' | '_' | '.' | ' ') {
                score += 10;
            }
            if i == base {
                score += 10;
            }
            if n > 0 {
                score -= (i - pos[n - 1] - 1) as i64;
                if i == pos[n - 1] + 1 {
                    score += 8;
                }
            }
        }
        if pos[0] >= base {
            score += 30;
        }
        total += score;
    }
    // Chemins courts d'abord à égalité
    any.then_some(total * 4 - (hay.len() - base) as i64)
}

// Tout le contenu de root, niveau par niveau (les liens vers des dossiers ne sont pas suivis), sans lire les
// métadonnées (name = chemin relatif, le reste est complété pour les seuls résultats affichés). Les dossiers cachés ne
// sont parcourus que si hidden. emit reçoit les éléments par paquets; s'arrête à cap éléments ou quand emit rend false.
pub fn walk(root: &Path, hidden: bool, cap: usize, mut emit: impl FnMut(Vec<Entry>) -> bool) {
    let mut batch = Vec::new();
    let mut total = 0;
    let mut queue = std::collections::VecDeque::from([PathBuf::new()]);
    while let Some(rel) = queue.pop_front() {
        let Ok(rd) = fs::read_dir(root.join(&rel)) else { continue };
        for d in rd.flatten() {
            let name = d.file_name().to_string_lossy().into_owned();
            let sub = rel.join(&name);
            let is_dir = d.file_type().is_ok_and(|t| t.is_dir());
            if is_dir && (hidden || !name.starts_with('.')) {
                queue.push_back(sub.clone());
            }
            batch.push(Entry {
                path: root.join(&sub).to_string_lossy().into_owned(),
                name: sub.to_string_lossy().into_owned(),
                is_dir,
                size: 0,
                mtime: 0,
                exec: false,
                created: 0,
                mode: 0,
                uid: 0,
                kind: String::new(),
            });
            total += 1;
            if batch.len() >= 3000 && !emit(std::mem::take(&mut batch)) {
                return;
            }
        }
        if total >= cap {
            break;
        }
    }
    if !batch.is_empty() {
        emit(batch);
    }
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
        Entry { name: name.into(), path: format!("/x/{name}"), is_dir, size, mtime, exec: false, created: mtime, mode: 0o644, uid: 0, kind: String::new() }
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
    fn filter_ignores_case_and_accents() {
        assert!(contains("Écoles-2024.PDF", "ecole"));
        assert!(contains("Écoles-2024.PDF", "pdf"));
        assert!(!contains("notes.txt", "pdf"));
        assert!(fuzzy("src/Écoles-2024.PDF", "ec24").is_some());
        assert!(fuzzy("src/main.rs", "mrs").is_some());
        assert!(fuzzy("src/main.rs", "rsm").is_none());
        assert!(fuzzy("a/budget.txt", "bud txt").is_some());
        assert!(fuzzy("a/budget.txt", "bud pdf").is_none());
        assert!(fuzzy("docs/readme.md", "readme") > fuzzy("readme/docs/a.md", "readme"));
    }

    #[test]
    fn permissions_like_ls() {
        assert_eq!(permissions(0o755), "rwxr-xr-x");
        assert_eq!(permissions(0o640), "rw-r-----");
    }

    #[test]
    fn glob() {
        assert!(glob_match("Rapport.PDF", "*.pdf"));
        assert!(glob_match("a1.txt", "a?.txt"));
        assert!(!glob_match("a12.txt", "a?.txt"));
        assert!(glob_match("photo vacances.jpg", "vacances"));
        assert!(glob_match("x", "*"));
        assert!(!glob_match("notes.txt", "*.pdf"));
        assert!(glob_match("a.tar.gz", "*.tar.*"));
        assert!(!glob_match("a", ""));
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

#[cfg(test)]
mod walk_tests {
    use super::*;

    #[test]
    fn walk_goes_all_levels() {
        let root = std::env::temp_dir().join(format!("fili-walk-{}", std::process::id()));
        let deep = root.join("a/b/c/d");
        fs::create_dir_all(&deep).unwrap();
        fs::write(deep.join("target.txt"), "x").unwrap();
        let mut all = Vec::new();
        walk(&root, false, 1000, |b| {
            all.extend(b);
            true
        });
        let _ = fs::remove_dir_all(&root);
        let names: Vec<_> = all.iter().map(|e| e.name.clone()).collect();
        assert!(names.contains(&"a/b/c/d/target.txt".to_string()), "{names:?}");
    }
}

