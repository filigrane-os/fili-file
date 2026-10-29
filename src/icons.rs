// Icônes du thème Papirus (celui de Filigrane, mêmes icônes en mode clair et sombre): type MIME d'après l'extension
// (shared-mime-info: globs2, generic-icons), puis nom d'icône freedesktop -> fichier. Tables lues une fois,
// fichiers trouvés gardés en mémoire.
use crate::fs::Entry;
use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Mutex, OnceLock},
};

const THEMES: [&str; 3] = ["Papirus", "Adwaita", "hicolor"];
// 22 px en premier: les dossiers 16 px de Papirus sont monochromes (icônes de panneau), sombres sur fond sombre
const SIZES: [&str; 6] = ["22x22", "32x32", "24x24", "16x16", "48x48", "scalable"];
const CONTEXTS: [&str; 4] = ["places", "mimetypes", "devices", "apps"];

struct Mime {
    by_ext: HashMap<String, String>,
    generic: HashMap<String, String>,
}

fn mime() -> &'static Mime {
    static M: OnceLock<Mime> = OnceLock::new();
    M.get_or_init(|| {
        // globs2: « poids:type:motif », poids décroissants: le premier type d'une extension l'emporte
        let mut by_ext = HashMap::new();
        for line in fs::read_to_string("/usr/share/mime/globs2").unwrap_or_default().lines() {
            let mut f = line.splitn(3, ':');
            let (Some(_), Some(t), Some(glob)) = (f.next(), f.next(), f.next()) else { continue };
            let Some(ext) = glob.strip_prefix("*.") else { continue };
            if !ext.contains(['*', '?', '[']) {
                by_ext.entry(ext.to_lowercase()).or_insert_with(|| t.to_string());
            }
        }
        let generic = fs::read_to_string("/usr/share/mime/generic-icons")
            .unwrap_or_default()
            .lines()
            .filter_map(|l| l.split_once(':').map(|(a, b)| (a.to_string(), b.to_string())))
            .collect();
        Mime { by_ext, generic }
    })
}

// Fichier de l'icône, en URL file:// pour QML; "" si aucune des propositions n'existe
fn find(names: &[&str]) -> String {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    let mut cache = cache.lock().unwrap_or_else(|e| e.into_inner());
    for name in names {
        if let Some(url) = cache.get(*name) {
            if !url.is_empty() {
                return url.clone();
            }
            continue;
        }
        let url = lookup(name).map(|p| format!("file://{p}")).unwrap_or_default();
        cache.insert(name.to_string(), url.clone());
        if !url.is_empty() {
            return url;
        }
    }
    String::new()
}

fn lookup(name: &str) -> Option<String> {
    for theme in THEMES {
        for size in SIZES {
            for ctx in CONTEXTS {
                for ext in ["svg", "png"] {
                    let p = format!("/usr/share/icons/{theme}/{size}/{ctx}/{name}.{ext}");
                    if Path::new(&p).exists() {
                        return Some(p);
                    }
                }
            }
        }
    }
    None
}

pub fn named(name: &str) -> String {
    find(&[name])
}

// Dossiers personnels (XDG) reconnus par leur chemin, sinon dossier générique
pub fn folder(path: &str, special: &HashMap<String, &'static str>) -> String {
    match special.get(path) {
        Some(name) => find(&[name, "folder"]),
        None => find(&["folder"]),
    }
}

pub fn file(e: &Entry) -> String {
    let m = mime();
    let ext = e.name.rsplit_once('.').filter(|(stem, _)| !stem.is_empty()).map(|(_, x)| x.to_lowercase());
    let t = ext.and_then(|x| m.by_ext.get(&x));
    let exact = t.map(|t| t.replace('/', "-"));
    let generic = t.and_then(|t| m.generic.get(t)).map(String::as_str);
    let mut names: Vec<&str> = Vec::with_capacity(4);
    names.extend(exact.as_deref());
    names.extend(generic);
    if e.exec {
        names.push("application-x-executable");
    }
    names.push("text-x-generic");
    find(&names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_by_extension() {
        let m = mime();
        if m.by_ext.is_empty() {
            return; // shared-mime-info absent
        }
        assert_eq!(m.by_ext["png"], "image/png");
        assert_eq!(m.by_ext["pdf"], "application/pdf");
    }
}
