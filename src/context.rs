// Contexte partagé avec l'assistant (fili-ai): l'application écrit son état utile (dossier, sélection) dans
// $XDG_RUNTIME_DIR/fili/context/<pid>.json. Rien ne tourne en plus: le fichier est lu à la demande par
// `fili-context`, qui retrouve celui de la fenêtre active par son pid (niri). Contrat commun: voir le skill filigrane-ui.
use std::{fs, path::PathBuf};

fn file() -> Option<PathBuf> {
    let dir = PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR")?).join("fili/context");
    fs::create_dir_all(&dir).ok()?;
    Some(dir.join(format!("{}.json", std::process::id())))
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

// Ce que l'assistant doit savoir de l'application (contrat commun des fili-*): le rôle, le sens des champs, la voie
// d'exécution conseillée. Court: il part dans le prompt.
const ABOUT: &str = "File manager. location = folder shown, selection = paths the user selected (absolute). \
Act on files with normal shell tools (mv, cp, rg, sha256sum, tar...) using these paths; prefer the shell for batch work. \
fili-file shows changes live (inotify), so no refresh call is needed. Trash with gio trash, never rm, unless asked.";

pub fn publish(app: &str, location: &str, selection: &[String]) {
    let Some(f) = file() else { return };
    let sel: Vec<String> = selection.iter().take(200).map(|p| json(p)).collect();
    let body = format!(
        "{{\"app\":{},\"about\":{},\"pid\":{},\"location\":{},\"selection\":[{}],\"selection_count\":{}}}\n",
        json(app),
        json(ABOUT),
        std::process::id(),
        json(location),
        sel.join(","),
        selection.len()
    );
    let tmp = f.with_extension("tmp");
    if fs::write(&tmp, body).is_ok() {
        let _ = fs::rename(&tmp, &f);
    }
}

pub fn clear() {
    if let Some(f) = file() {
        let _ = fs::remove_file(f);
    }
}
