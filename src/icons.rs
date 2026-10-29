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

// Glyphe monochrome 16 px de la barre latérale (Papirus-Dark: clair, pour le mode sombre; Papirus: sombre), que le
// QML teinte; à défaut, l'icône en couleur
pub fn symbolic(name: &str, dark: bool) -> String {
    let theme = if dark { "Papirus-Dark" } else { "Papirus" };
    for ctx in ["places", "devices", "actions"] {
        let p = format!("/usr/share/icons/{theme}/16x16/{ctx}/{name}.svg");
        if Path::new(&p).exists() {
            return format!("file://{p}");
        }
    }
    named(name)
}

// Dossiers bleus (variante Papirus « folder-blue », comme le Finder): dossiers personnels (XDG) reconnus par leur
// chemin, sinon dossier générique
pub fn folder(path: &str, special: &HashMap<String, &'static str>) -> String {
    match special.get(path) {
        Some(name) => {
            let kind = match name.trim_start_matches("folder-").trim_start_matches("user-") {
                "download" => "downloads",
                k => k,
            };
            find(&[&format!("folder-blue-{kind}"), name, "folder-blue", "folder"])
        }
        None => find(&["folder-blue", "folder"]),
    }
}

// Type MIME d'un fichier d'après son extension
pub fn mime_type(name: &str) -> Option<String> {
    let ext = name.rsplit_once('.').filter(|(stem, _)| !stem.is_empty())?.1.to_lowercase();
    mime().by_ext.get(&ext).cloned()
}

// Type de fichier lisible (« Image PNG »): commentaire de shared-mime-info dans la langue de la session, à défaut le
// type MIME, à défaut « FICHIER »; « Dossier » pour un dossier
pub fn kind(name: &str, is_dir: bool) -> String {
    use crate::i18n::{tr, trf};
    if is_dir {
        return tr("Folder").to_string();
    }
    if let Some(t) = mime_type(name) {
        static CACHE: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);
        let mut guard = CACHE.lock().unwrap_or_else(|e| e.into_inner());
        let cache = guard.get_or_insert_with(HashMap::new);
        return cache.entry(t.clone()).or_insert_with(|| mime_comment(&t).unwrap_or(t)).clone();
    }
    match name.rsplit_once('.').filter(|(stem, ext)| !stem.is_empty() && !ext.is_empty()) {
        Some((_, ext)) => trf("%1 file", &[&ext.to_uppercase()]),
        None => tr("File").to_string(),
    }
}

fn mime_comment(t: &str) -> Option<String> {
    let text = fs::read_to_string(format!("/usr/share/mime/{t}.xml")).ok()?;
    let lang = std::env::var("FILI_LANG").or_else(|_| std::env::var("LANG")).unwrap_or_default();
    let lang = lang.get(..2).unwrap_or("en");
    let pick = |open: &str| -> Option<String> {
        let i = text.find(open)? + open.len();
        let j = text[i..].find("</comment>")?;
        Some(text[i..i + j].to_string())
    };
    pick(&format!("<comment xml:lang=\"{lang}\">")).or_else(|| pick("<comment>"))
}

const IMAGES: [&str; 8] = ["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "avif"];

// Miniature d'une image: le fichier lui-même, que Qt décode à la taille de l'icône (pas de cache ni de processus);
// au-delà de 20 Mio l'icône du type
pub fn is_image(e: &Entry) -> bool {
    e.size < 20 << 20 && e.name.rsplit_once('.').is_some_and(|(_, x)| IMAGES.contains(&x.to_lowercase().as_str()))
}

pub fn is_pdf(e: &Entry) -> bool {
    e.name.to_lowercase().ends_with(".pdf")
}

pub fn file(e: &Entry) -> String {
    if is_image(e) {
        return format!("file://{}", e.path);
    }
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
