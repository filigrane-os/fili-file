// Icons of files and folders, from whatever freedesktop icon theme is in use (icon_theme.rs): MIME type from the
// extension (shared-mime-info: globs2, generic-icons), then freedesktop icon name -> file. Tables are read once, the
// files found are kept in memory.
use crate::{fs::Entry, icon_theme};
use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Mutex, OnceLock},
};

// Icons for the lists and the grid: scalable icons fit any size, fixed ones are scaled by Qt
const ICON_SIZE: u32 = 48;
// Sidebar glyphs
const GLYPH_SIZE: u32 = 16;

struct Mime {
    by_ext: HashMap<String, String>,
    generic: HashMap<String, String>,
}

fn mime() -> &'static Mime {
    static M: OnceLock<Mime> = OnceLock::new();
    M.get_or_init(|| {
        let mut by_ext = HashMap::new();
        let mut generic = HashMap::new();
        // XDG_DATA_HOME first, then XDG_DATA_DIRS: the first definition wins
        for dir in icon_theme::data_dirs() {
            // globs2: "weight:type:pattern", by decreasing weight: the first type of an extension wins
            for line in fs::read_to_string(dir.join("mime/globs2")).unwrap_or_default().lines() {
                let mut f = line.splitn(3, ':');
                let (Some(_), Some(t), Some(glob)) = (f.next(), f.next(), f.next()) else { continue };
                let Some(ext) = glob.strip_prefix("*.") else { continue };
                if !ext.contains(['*', '?', '[']) {
                    by_ext.entry(ext.to_lowercase()).or_insert_with(|| t.to_string());
                }
            }
            for line in fs::read_to_string(dir.join("mime/generic-icons")).unwrap_or_default().lines() {
                if let Some((a, b)) = line.split_once(':') {
                    generic.entry(a.to_string()).or_insert_with(|| b.to_string());
                }
            }
        }
        Mime { by_ext, generic }
    })
}

fn lookup(name: &str, size: u32) -> Option<String> {
    icon_theme::system().lookup(icon_theme::current(), name, size).map(|p| p.to_string_lossy().into_owned())
}

// File of the icon as a file:// URL for QML, trying the names in order; "" when none exists
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
        let url = lookup(name, ICON_SIZE).map(|p| format!("file://{p}")).unwrap_or_default();
        cache.insert(name.to_string(), url.clone());
        if !url.is_empty() {
            return url;
        }
    }
    String::new()
}

// Icon name set for every folder / every regular file: fili-file.toml first, then the hint of the theme
// (X-Fili-FolderIcon, X-Fili-FileIcon, see icon_theme.rs)
fn hint(configured: Option<String>, key: &str) -> Option<String> {
    configured.or_else(|| icon_theme::system().key(icon_theme::current(), key))
}

fn folder_name() -> Option<&'static str> {
    static N: OnceLock<Option<String>> = OnceLock::new();
    N.get_or_init(|| hint(crate::config::load().folder_icon, "X-Fili-FolderIcon")).as_deref()
}

fn file_name() -> Option<&'static str> {
    static N: OnceLock<Option<String>> = OnceLock::new();
    N.get_or_init(|| hint(crate::config::load().file_icon, "X-Fili-FileIcon")).as_deref()
}

// Cache of the recolored copies of icons: $XDG_CACHE_HOME/fili-file/tint
fn tint_dir() -> Option<String> {
    let cache = std::env::var("XDG_CACHE_HOME").ok().filter(|v| !v.is_empty()).or_else(|| std::env::var("HOME").ok().map(|h| format!("{h}/.cache")))?;
    Some(format!("{cache}/fili-file/tint"))
}

// All the hexadecimal colors of an SVG replaced by `color`
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

// Recolored copy of an SVG in the tint cache, named after the color and the path; the original URL if the copy fails
fn recolored(path: &str, color: &str) -> String {
    let Some(dir) = tint_dir() else { return format!("file://{path}") };
    let out = format!("{dir}/{}-{}", &color[1..], path.trim_start_matches('/').replace('/', "_"));
    if !Path::new(&out).exists() {
        let Ok(svg) = fs::read_to_string(path) else { return format!("file://{path}") };
        if fs::create_dir_all(&dir).and_then(|_| fs::write(&out, recolor_svg(&svg, color))).is_err() {
            return format!("file://{path}");
        }
    }
    format!("file://{out}")
}

// Sidebar glyph, in white so that the QML tints it to the accent by colorization (which keeps the luminance: a black
// glyph would stay black). In order: the theme's filled glyphs (X-Fili-GlyphDir), its "<name>-symbolic" icons
// ("folder" when the name has none), else the plain icon as it is.
pub fn symbolic(name: &str) -> String {
    let base = if name == "user-trash-full" { "user-trash" } else { name };
    let themes = icon_theme::system();
    let theme = icon_theme::current();
    let glyph = themes.key(theme, "X-Fili-GlyphDir").and_then(|dir| themes.in_dir(theme, &dir, name)).map(|p| p.to_string_lossy().into_owned());
    let symbolic = glyph
        .or_else(|| lookup(&format!("{name}-symbolic"), GLYPH_SIZE))
        .or_else(|| lookup(&format!("{base}-symbolic"), GLYPH_SIZE))
        .filter(|p| p.ends_with(".svg"));
    if let Some(p) = symbolic {
        return recolored(&p, "#ffffff");
    }
    lookup(name, GLYPH_SIZE).or_else(|| lookup(base, GLYPH_SIZE)).or_else(|| lookup("folder", GLYPH_SIZE)).map(|p| format!("file://{p}")).unwrap_or_default()
}

// Folder icon, the same for every folder (no own icon for Documents, Downloads, Pictures...)
pub fn folder() -> String {
    match folder_name() {
        Some(n) => find(&[n, "folder"]),
        None => find(&["folder"]),
    }
}

// MIME type of a file from its extension
pub fn mime_type(name: &str) -> Option<String> {
    let ext = name.rsplit_once('.').filter(|(stem, _)| !stem.is_empty())?.1.to_lowercase();
    mime().by_ext.get(&ext).cloned()
}

// Readable file type ("PNG image"): shared-mime-info comment in the session language, else the MIME type, else
// "FILE"; "Folder" for a folder
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
    let text = icon_theme::data_dirs().iter().find_map(|d| fs::read_to_string(d.join(format!("mime/{t}.xml"))).ok())?;
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

// Thumbnail of an image: the file itself, which Qt decodes at the size of the icon (no cache, no process); above
// 20 MiB the icon of its type
pub fn is_image(e: &Entry) -> bool {
    e.size < 20 << 20 && e.name.rsplit_once('.').is_some_and(|(_, x)| IMAGES.contains(&x.to_lowercase().as_str()))
}

pub fn is_image_name(name: &str) -> bool {
    name.rsplit_once('.').is_some_and(|(_, x)| IMAGES.contains(&x.to_lowercase().as_str()))
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
    let mut names: Vec<&str> = Vec::with_capacity(5);
    // A theme (or fili-file.toml) may want the same icon for every file: it passes before the icon of the type
    names.extend(file_name());
    names.extend(exact.as_deref());
    names.extend(generic);
    if e.exec {
        names.push("application-x-executable");
    }
    names.push("text-x-generic");
    find(&names)
}

// Folder "folder-blue*" (two flat colors, #5294e2 in front and #4877b1 behind) recolored to the accent: copy in the
// tint cache, named after the icon and the color. Any other icon is returned as it is.
pub fn tint(url: &str, accent: &str, light: bool) -> String {
    // Symbolic file type icon: gray of the secondary text of the mode
    if let Some(path) = url.strip_prefix("file://").filter(|p| p.ends_with("-symbolic.svg")) {
        return recolored(path, if light { "#4f5e70" } else { "#a9b8c8" });
    }
    let Some(path) = url.strip_prefix("file://").filter(|p| p.contains("/folder-blue") && p.ends_with(".svg")) else {
        return url.to_string();
    };
    // A theme that already provides its "folder-accent-<accent>.svg" folders (scripts/icon-theme.sh): use them
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
    // Tones of [theme.accents] (theme.toml overridden by the user's config)
    let tone = crate::config::accent_tone(accent);
    let Some(front) = tone.get(if light { 2 } else { 0 }) else { return url.to_string() };
    let rgb: Vec<u32> = (0..3).map(|i| u32::from_str_radix(&front[1 + 2 * i..3 + 2 * i], 16).unwrap_or(0)).collect();
    let back = format!("#{:02x}{:02x}{:02x}", rgb[0] * 85 / 100, rgb[1] * 85 / 100, rgb[2] * 85 / 100);
    let Some(dir) = tint_dir() else { return url.to_string() };
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
