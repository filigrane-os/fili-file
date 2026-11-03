// Translations: texts are written in English in the code (tr("Offline"), trf("Battery %1 %", &[&cap])) and translated
// by a flat JSON catalog {"English text": "translation", ...}, read once. Catalogs are looked up in
// <data dir>/fili-file/i18n/<language>.json for each XDG data directory (so a language can be added without
// recompiling), then in the ones embedded in the binary (i18n/ of the source tree). No catalog or no entry: English.
// Language: FILI_LANG, else the choice of fili-lang when Filigrane has one ($XDG_STATE_HOME/filigrane/locale), else the
// first set of LC_ALL, LC_MESSAGES, LANG ("fr_FR.UTF-8" -> fr; C, POSIX -> en).
use std::{collections::HashMap, env, fmt::Display, fs, sync::OnceLock};

// Catalogs compiled into the binary
const EMBEDDED: [(&str, &str); 1] = [("fr", include_str!("../i18n/fr.json"))];

static CATALOG: OnceLock<HashMap<String, String>> = OnceLock::new();

fn lang() -> String {
    // Tests expect the English texts of the code, whatever language the user chose (the catalog is fixed by the first
    // call: a test that does not set FILI_LANG would make the others fail)
    if cfg!(test) {
        return "en".into();
    }
    let state = env::var("XDG_STATE_HOME").unwrap_or_else(|_| format!("{}/.local/state", env::var("HOME").unwrap_or_default()));
    let choice = fs::read_to_string(format!("{state}/filigrane/locale")).map(|s| s.trim().to_string()).unwrap_or_default();
    let v = env::var("FILI_LANG")
        .ok()
        .filter(|v| !v.is_empty())
        .or(Some(choice).filter(|v| !v.is_empty()))
        .or_else(|| ["LC_ALL", "LC_MESSAGES", "LANG"].iter().find_map(|k| env::var(k).ok().filter(|v| !v.is_empty())))
        .unwrap_or_default();
    let l: String = v.chars().take_while(|c| !matches!(c, '_' | '.' | '@')).collect();
    if !l.is_empty() && l.chars().all(|c| c.is_ascii_lowercase()) { l } else { "en".into() }
}

fn catalog() -> &'static HashMap<String, String> {
    CATALOG.get_or_init(|| {
        let lang = lang();
        if lang == "en" {
            return HashMap::new();
        }
        let external = crate::icon_theme::data_dirs().iter().find_map(|d| fs::read_to_string(d.join(format!("fili-file/i18n/{lang}.json"))).ok());
        let text = external.or_else(|| EMBEDDED.iter().find(|(l, _)| *l == lang).map(|(_, t)| t.to_string()));
        text.map(|t| parse(&t)).unwrap_or_default()
    })
}

// Flat JSON object {"key": "value", ...}; anything that is not a pair of strings is ignored
fn parse(text: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let mut it = text.chars().peekable();
    let mut strings = Vec::new();
    while let Some(c) = it.next() {
        if c != '"' {
            continue;
        }
        let mut s = String::new();
        while let Some(c) = it.next() {
            match c {
                '"' => break,
                '\\' => match it.next() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('u') => {
                        let hex: String = it.by_ref().take(4).collect();
                        s.extend(u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32));
                    }
                    Some(c) => s.push(c),
                    None => break,
                },
                c => s.push(c),
            }
        }
        strings.push(s);
        if strings.len() == 2 {
            let v = strings.pop().unwrap_or_default();
            out.insert(strings.pop().unwrap_or_default(), v);
        }
    }
    out
}

pub fn tr(s: &str) -> &str {
    catalog().get(s).map(String::as_str).unwrap_or(s)
}

// Translated text with %1, %2... replaced by the arguments
pub fn trf(s: &str, args: &[&dyn Display]) -> String {
    let t = tr(s);
    let mut out = String::with_capacity(t.len() + 16);
    let mut chars = t.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, chars.peek().and_then(|d| d.to_digit(10))) {
            ('%', Some(n)) if n >= 1 && (n as usize) <= args.len() => {
                chars.next();
                out.push_str(&args[n as usize - 1].to_string());
            }
            _ => out.push(c),
        }
    }
    out
}

// Number with d decimals and the separator of the language (key "@decimal" of the catalog)
pub fn num(v: f64, d: usize) -> String {
    let s = format!("{v:.d$}");
    match catalog().get("@decimal") {
        Some(sep) => s.replacen('.', sep, 1),
        None => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flat_object() {
        let c = parse(r#"{"Offline": "Hors ligne", "a \"b\"": "é\n", "@decimal": ","}"#);
        assert_eq!(c["Offline"], "Hors ligne");
        assert_eq!(c["a \"b\""], "é\n");
        assert_eq!(c["@decimal"], ",");
    }

    #[test]
    fn english_without_catalog() {
        assert_eq!(trf("Battery %1 %, %2", &[&66, &"x"]), "Battery 66 %, x");
        assert_eq!(trf("100 % %3", &[&1]), "100 % %3");
    }

    #[test]
    fn the_embedded_french_catalog_parses_and_keeps_its_placeholders() {
        let fr = parse(EMBEDDED[0].1);
        assert!(fr.len() > 200, "{} entries", fr.len());
        assert_eq!(fr["@decimal"], ",");
        for (k, v) in &fr {
            let count = |s: &str| (1..=9).filter(|n| s.contains(&format!("%{n}"))).count();
            assert_eq!(count(k), count(v), "placeholders of {k:?}");
        }
    }
}
