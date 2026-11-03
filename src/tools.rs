// Programs the application can use when they are installed. Nothing here is required to browse files: a feature whose
// program is missing is switched off (compress and extract need fili-archive, Nearby and Share need fili-share) or
// replaced (the terminal is the first one found).
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

// Terminals tried after $TERMINAL and xdg-terminal-exec: the process starts in the folder, no option needed
const TERMINALS: [&str; 10] = ["foot", "kitty", "alacritty", "wezterm", "ghostty", "gnome-terminal", "kgx", "konsole", "xfce4-terminal", "xterm"];

// `name` in the directories of `path` (a PATH-like string), then in the extra ones
fn search(name: &str, path: &str, extra: &[PathBuf]) -> Option<PathBuf> {
    if name.contains('/') {
        return Some(PathBuf::from(name)).filter(|p| is_executable(p));
    }
    path.split(':').filter(|d| !d.is_empty()).map(PathBuf::from).chain(extra.iter().cloned()).map(|d| d.join(name)).find(|p| is_executable(p))
}

fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.metadata().is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

// Full path of a program: PATH, then ~/.local/bin (where bootstrap links the tools it builds)
pub fn find(name: &str) -> Option<PathBuf> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<PathBuf>>>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(Default::default).lock().unwrap_or_else(|e| e.into_inner());
    cache
        .entry(name.to_string())
        .or_insert_with(|| {
            let home = std::env::var("HOME").unwrap_or_default();
            search(name, &std::env::var("PATH").unwrap_or_default(), &[PathBuf::from(format!("{home}/.local/bin"))])
        })
        .clone()
}

pub fn available(name: &str) -> bool {
    find(name).is_some()
}

// Program to run for a terminal window: $TERMINAL, xdg-terminal-exec, then the first known terminal that is installed
pub fn terminal() -> Option<PathBuf> {
    let from_env = std::env::var("TERMINAL").ok().filter(|t| !t.is_empty()).and_then(|t| find(&t));
    from_env.or_else(|| find("xdg-terminal-exec")).or_else(|| TERMINALS.iter().find_map(|t| find(t)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt};

    fn tool_dir(tag: &str, names: &[&str]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("fili-tools-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        for n in names {
            fs::write(dir.join(n), "#!/bin/sh\n").unwrap();
            fs::set_permissions(dir.join(n), fs::Permissions::from_mode(0o755)).unwrap();
        }
        dir
    }

    #[test]
    fn finds_executables_in_path_and_extra_directories() {
        let a = tool_dir("a", &["alpha"]);
        let b = tool_dir("b", &["beta"]);
        let path = format!("/nonexistent:{}", a.display());
        assert_eq!(search("alpha", &path, &[]), Some(a.join("alpha")));
        assert_eq!(search("beta", &path, &[]), None);
        assert_eq!(search("beta", &path, std::slice::from_ref(&b)), Some(b.join("beta")));
        assert_eq!(search("gamma", &path, &[b.clone()]), None);
        let _ = (fs::remove_dir_all(a), fs::remove_dir_all(b));
    }

    #[test]
    fn a_file_without_the_execute_bit_is_not_a_program() {
        let d = tool_dir("x", &[]);
        fs::write(d.join("plain"), "x").unwrap();
        assert_eq!(search("plain", &d.display().to_string(), &[]), None);
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn explicit_paths_are_used_as_they_are() {
        let d = tool_dir("p", &["term"]);
        assert_eq!(search(&d.join("term").display().to_string(), "", &[]), Some(d.join("term")));
        assert_eq!(search("/nonexistent/term", "", &[]), None);
        let _ = fs::remove_dir_all(d);
    }
}
