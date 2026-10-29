// Mode sélecteur de fichiers (lancé par le portail FileChooser, voir portal.rs): fili-file --pick open|save
// [--multiple] [--directory] [--title T] [--name N] [--folder DOSSIER] [--accept LIBELLÉ] [--filters JSON]
// [--filter N] --out FICHIER. La sélection est écrite dans FICHIER, un chemin par ligne (vide: annulé).
// Sans Qt: testé par cargo test.
use std::{env, sync::OnceLock};

#[derive(Clone, Debug, PartialEq)]
pub enum Pattern {
    Glob(String),
    Mime(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Filter {
    pub name: String,
    pub patterns: Vec<Pattern>,
}

#[derive(Default, Debug)]
pub struct Pick {
    pub kind: String,
    pub multiple: bool,
    pub directory: bool,
    pub title: String,
    pub name: String,
    pub folder: String,
    pub accept: String,
    pub filters: Vec<Filter>,
    pub filter: i32,
    pub out: String,
}

const WITH_VALUE: [&str; 9] = ["--pick", "--title", "--name", "--folder", "--accept", "--filters", "--filter", "--out", "--kind"];

pub fn parse(args: &[String]) -> Option<Pick> {
    let mut p = Pick::default();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = || it.next().cloned().unwrap_or_default();
        match a.as_str() {
            "--pick" => p.kind = value(),
            "--multiple" => p.multiple = true,
            "--directory" => p.directory = true,
            "--title" => p.title = value(),
            "--name" => p.name = value(),
            "--folder" => p.folder = value(),
            "--accept" => p.accept = value(),
            "--filters" => p.filters = parse_filters(&value()),
            "--filter" => p.filter = value().parse().unwrap_or(-1),
            "--out" => p.out = value(),
            _ => {}
        }
    }
    (!p.kind.is_empty()).then_some(p)
}

// Premier argument qui n'est ni une option ni la valeur d'une option (dossier, fichier ou URL à ouvrir)
pub fn positional(args: &[String]) -> Option<String> {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if WITH_VALUE.contains(&a.as_str()) {
            it.next();
        } else if !a.starts_with('-') {
            return Some(a.clone());
        }
    }
    None
}

pub fn get() -> Option<&'static Pick> {
    static P: OnceLock<Option<Pick>> = OnceLock::new();
    P.get_or_init(|| parse(&env::args().skip(1).collect::<Vec<_>>())).as_ref()
}

// [{"name":"Images","globs":["*.png"],"mimes":["image/*"]}]
fn parse_filters(json: &str) -> Vec<Filter> {
    let mut out = Vec::new();
    let mut rest = json;
    while let Some(i) = rest.find("{\"name\":") {
        rest = &rest[i + 8..];
        let Some((name, after)) = read_string(rest) else { break };
        let end = after.find('}').unwrap_or(after.len());
        let body = &after[..end];
        let mut patterns = Vec::new();
        for (key, is_glob) in [("\"globs\":[", true), ("\"mimes\":[", false)] {
            if let Some(j) = body.find(key) {
                let mut s = &body[j + key.len()..];
                while let Some((v, tail)) = read_string(s) {
                    patterns.push(if is_glob { Pattern::Glob(v) } else { Pattern::Mime(v) });
                    s = tail.trim_start_matches(',');
                    if s.starts_with(']') {
                        break;
                    }
                }
            }
        }
        out.push(Filter { name, patterns });
        rest = &after[end..];
    }
    out
}

// Chaîne JSON entre guillemets au début de s (après espaces): (valeur, reste)
fn read_string(s: &str) -> Option<(String, &str)> {
    let s = s.trim_start().strip_prefix('"')?;
    let mut out = String::new();
    let mut chars = s.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return Some((out, &s[i + 1..])),
            '\\' => match chars.next() {
                Some((_, 'n')) => out.push('\n'),
                Some((_, c)) => out.push(c),
                None => return None,
            },
            c => out.push(c),
        }
    }
    None
}

// « * » et « ? » sans casse
fn glob(pattern: &str, name: &str) -> bool {
    fn go(p: &[char], n: &[char]) -> bool {
        match p.split_first() {
            None => n.is_empty(),
            Some(('*', rest)) => (0..=n.len()).any(|i| go(rest, &n[i..])),
            Some(('?', rest)) => !n.is_empty() && go(rest, &n[1..]),
            Some((c, rest)) => n.first().is_some_and(|d| d == c) && go(rest, &n[1..]),
        }
    }
    let p: Vec<char> = pattern.to_lowercase().chars().collect();
    let n: Vec<char> = name.to_lowercase().chars().collect();
    go(&p, &n)
}

impl Filter {
    pub fn matches(&self, name: &str, mime: Option<&str>) -> bool {
        self.patterns.is_empty()
            || self.patterns.iter().any(|p| match p {
                Pattern::Glob(g) => glob(g, name),
                Pattern::Mime(m) => mime.is_some_and(|t| match m.strip_suffix("/*") {
                    Some(prefix) => t.starts_with(&format!("{prefix}/")),
                    None => t == m,
                }),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split(' ').map(String::from).collect()
    }

    #[test]
    fn parses_arguments() {
        let p = parse(&args("--pick save --title Enregistrer --name a.txt --multiple --out /tmp/x --filter 1")).unwrap();
        assert_eq!((p.kind.as_str(), p.title.as_str(), p.name.as_str(), p.multiple, p.filter), ("save", "Enregistrer", "a.txt", true, 1));
        assert!(parse(&args("/tmp")).is_none());
        assert_eq!(positional(&args("--pick open --out /tmp/x /home")).as_deref(), Some("/home"));
        assert_eq!(positional(&args("--pick open --out /tmp/x")), None);
    }

    #[test]
    fn parses_filters_and_matches() {
        let f = parse_filters(r#"[{"name":"Images","globs":["*.PNG","*.jpg"],"mimes":["image/*"]},{"name":"Tous","globs":["*"],"mimes":[]}]"#);
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].name, "Images");
        assert!(f[0].matches("photo.png", None) && f[0].matches("x.JPG", None));
        assert!(f[0].matches("y.webp", Some("image/webp")));
        assert!(!f[0].matches("notes.txt", Some("text/plain")));
        assert!(f[1].matches("anything", None));
    }
}
