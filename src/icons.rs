// Icônes de Filigrane (thème « Filigrane-Icons » de scripts/icon-theme.sh: dossiers et glyphes de Nordic; le reste vient
// d'Adwaita, en symboliques recolorés selon le mode): type MIME d'après l'extension (shared-mime-info: globs2,
// generic-icons), puis nom d'icône freedesktop -> fichier. Tables lues une fois, fichiers trouvés gardés en mémoire.
use crate::fs::Entry;
use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Mutex, OnceLock},
};

const OWN_THEME: &str = "Filigrane-Icons";
const THEMES: [&str; 2] = ["Adwaita", "hicolor"];
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
        if let Some(p) = lookup_user(name) {
            return format!("file://{p}");
        }
    }
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

// Racine du thème Filigrane-Icons (~/.local/share/icons, à défaut /usr/share/icons), rangé par contexte puis taille
// (<ctx>/<taille>/)
fn own_root() -> Option<&'static str> {
    static T: OnceLock<Option<String>> = OnceLock::new();
    T.get_or_init(|| {
        let home = std::env::var("HOME").unwrap_or_default();
        [format!("{home}/.local/share/icons/{OWN_THEME}"), format!("/usr/share/icons/{OWN_THEME}")].into_iter().find(|r| Path::new(r).is_dir())
    })
    .as_deref()
}

// Icône du thème Filigrane, prioritaire pour toutes les propositions
fn lookup_user(name: &str) -> Option<String> {
    let root = own_root()?;
    for ctx in CONTEXTS {
        for size in ["scalable", "32", "24", "22", "16"] {
            for ext in ["svg", "png"] {
                let p = format!("{root}/{ctx}/{size}/{name}.{ext}");
                if Path::new(&p).exists() {
                    return Some(p);
                }
            }
        }
    }
    None
}

// Symbolique « <nom>-symbolic.svg »: thème Filigrane puis Adwaita
fn lookup_symbolic(name: &str) -> Option<String> {
    let mut roots: Vec<String> = own_root().map(String::from).into_iter().collect();
    roots.push("/usr/share/icons/Adwaita/symbolic".into());
    for root in &roots {
        let own = root.ends_with(OWN_THEME);
        for ctx in ["places", "mimetypes", "devices", "status", "actions"] {
            let p = if own { format!("{root}/{ctx}/symbolic/{name}-symbolic.svg") } else { format!("{root}/{ctx}/{name}-symbolic.svg") };
            if Path::new(&p).exists() {
                return Some(p);
            }
        }
    }
    None
}

fn lookup(name: &str) -> Option<String> {
    if let Some(p) = lookup_symbolic(name) {
        return Some(p);
    }
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

// Toutes les couleurs hexadécimales d'un SVG remplacées par `color`
fn recolor_svg(svg: &str, color: &str) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(i) = rest.find('#') {
        let hex = rest[i + 1..].bytes().take_while(u8::is_ascii_hexdigit).count();
        out.push_str(&rest[..i]);
        if matches!(hex, 3 | 6 | 8) {
            out.push_str(color);
            rest = &rest[i + 1 + hex..];
        } else {
            out.push('#');
            rest = &rest[i + 1..];
        }
    }
    out.push_str(rest);
    out
}

// Copie recolorée d'un SVG dans ~/.cache/fili-file/tint, nommée d'après la couleur et le chemin; l'URL d'origine si la
// copie échoue
fn recolored(path: &str, color: &str) -> String {
    let Ok(home) = std::env::var("HOME") else { return format!("file://{path}") };
    let dir = format!("{home}/.cache/fili-file/tint");
    let out = format!("{dir}/{}-{}", &color[1..], path.trim_start_matches('/').replace('/', "_"));
    if !Path::new(&out).exists() {
        let Ok(svg) = fs::read_to_string(path) else { return format!("file://{path}") };
        if fs::create_dir_all(&dir).and_then(|_| fs::write(&out, recolor_svg(&svg, color))).is_err() {
            return format!("file://{path}");
        }
    }
    format!("file://{out}")
}

// Glyphe de la barre latérale: symbolique du thème (dossier « folder » à défaut), en blanc pour que le QML le teinte
// à l'accent par colorisation (qui garde la luminance: un glyphe noir resterait noir)
pub fn symbolic(name: &str) -> String {
    let base = if name == "user-trash-full" { "user-trash" } else { name };
    // Glyphes pleins de 16 px du thème (places/16), plus lisibles que les symboliques à traits fins
    let full = own_root().map(|r| format!("{r}/places/16/{name}.svg")).filter(|p| Path::new(p).exists());
    let found = full
        .or_else(|| lookup_symbolic(name)).or_else(|| lookup_symbolic(base)).or_else(|| lookup_symbolic("folder"));
    found.map(|p| recolored(&p, "#ffffff")).unwrap_or_default()
}

// Dossiers bleus (variante « folder-blue » du thème, comme le Finder): dossiers personnels (XDG) reconnus par leur
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
    // file.svg du thème Filigrane passe avant tout (même icône pour tous les fichiers); sans le thème, le symbolique
    // Adwaita du type
    names.extend(["file", "text-x-generic"]);
    find(&names)
}

// Teintes d'accentuation [sombre, clair] (FiliTheme.qml, accentTones; vérifiées par un test)
const ACCENTS: [(&str, &str, &str); 7] = [
    ("default", "#dbe7f4", "#1c222b"),
    ("voile", "#9dbbd6", "#3f5a78"),
    ("ardoise", "#7f9ab6", "#4f6884"),
    ("sauge", "#a3bfa8", "#4a7354"),
    ("sable", "#d2bf92", "#80662b"),
    ("cendre", "#c98f93", "#9a4f55"),
    ("brume", "#dbe7f4", "#4f5e70"),
];

// Dossier « folder-blue* » (deux aplats, #5294e2 devant et #4877b1 derrière) recoloré à l'accent: copie
// dans ~/.cache/fili-file/tint, nommée d'après l'icône et la couleur. Toute autre icône est rendue telle quelle.
pub fn tint(url: &str, accent: &str, light: bool) -> String {
    // Icône de type de fichier symbolique (Adwaita): gris du texte secondaire du mode
    if let Some(path) = url.strip_prefix("file://").filter(|p| p.ends_with("-symbolic.svg")) {
        return recolored(path, if light { "#4f5e70" } else { "#a9b8c8" });
    }
    let Some(path) = url.strip_prefix("file://").filter(|p| p.contains("/folder-blue") && p.ends_with(".svg")) else {
        return url.to_string();
    };
    // Thème qui fournit déjà ses dossiers « folder-accent-<accent>.svg » (scripts/icon-theme.sh): on les prend
    if let Some(dir) = path.strip_suffix("/folder-blue.svg") {
        let kind = match (accent, light) {
            ("default", false) => "dark-default".to_string(),
            ("default", true) => "light-default".to_string(),
            _ => format!("accent-{accent}"),
        };
        let own = format!("{dir}/folder-{kind}.svg");
        if Path::new(&own).exists() {
            return format!("file://{own}");
        }
    }
    let (_, dark, clear) = ACCENTS.iter().find(|a| a.0 == accent).unwrap_or(&ACCENTS[0]);
    let front = if light { clear } else { dark };
    let rgb: Vec<u32> = (0..3).map(|i| u32::from_str_radix(&front[1 + 2 * i..3 + 2 * i], 16).unwrap_or(0)).collect();
    let back = format!("#{:02x}{:02x}{:02x}", rgb[0] * 85 / 100, rgb[1] * 85 / 100, rgb[2] * 85 / 100);
    let Ok(home) = std::env::var("HOME") else { return url.to_string() };
    let dir = format!("{home}/.cache/fili-file/tint");
    let out = format!("{dir}/{}-{}.svg", &front[1..], path.trim_start_matches('/').replace('/', "_").trim_end_matches(".svg"));
    if !Path::new(&out).exists() {
        let Ok(svg) = fs::read_to_string(path) else { return url.to_string() };
        let svg = svg.replace("#5294e2", front).replace("#4877b1", &back);
        if fs::create_dir_all(&dir).and_then(|_| fs::write(&out, svg)).is_err() {
            return url.to_string();
        }
    }
    format!("file://{out}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accents_match_the_shell() {
        let qml = fs::read_to_string(format!("{}/qml/FiliTheme.qml", env!("CARGO_MANIFEST_DIR"))).unwrap();
        for (name, dark, light) in ACCENTS {
            assert!(qml.contains(&format!("{name}: [\"{dark}\"")), "{name} sombre");
            assert!(qml.contains(&format!("\"{light}\", \"")), "{name} clair");
        }
    }

    #[test]
    fn recolor_replaces_hex_colors_only() {
        assert_eq!(recolor_svg("<path fill=\"#222222\" style=\"fill:#fff;stroke:none\" d=\"M1 1\"/> #zz", "#ffffff"), "<path fill=\"#ffffff\" style=\"fill:#ffffff;stroke:none\" d=\"M1 1\"/> #zz");
    }

    #[test]
    fn tint_leaves_other_icons_alone() {
        assert_eq!(tint("file:///usr/share/icons/hicolor/22x22/apps/x.png", "sable", false), "file:///usr/share/icons/hicolor/22x22/apps/x.png");
    }

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
