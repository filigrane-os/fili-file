// Usage state of Casier (what the user did, not what they configured): one folder, ~/.local/state/filigrane/fili-file/, one
// file per piece of state (ui, view, columns, visits, servers, favorites, preview), instead of a growing pile of
// `fili-file-<name>` next to the files of every other application.
use std::{
    env, fs,
    path::{Path, PathBuf},
};

// The file `name` of the state folder under `state` (XDG_STATE_HOME). A file of the versions up to 1.16, which sat next to
// the others as `fili-file-<name>`, is moved into the folder the first time it is asked for.
fn file_in(state: &Path, name: &str) -> PathBuf {
    let dir = state.join("filigrane");
    let new = dir.join("fili-file").join(name);
    let old = dir.join(format!("fili-file-{name}"));
    if !new.exists() && old.exists() {
        let _ = fs::create_dir_all(dir.join("fili-file"));
        let _ = fs::rename(&old, &new);
    }
    new
}

pub fn file(name: &str) -> PathBuf {
    let state = env::var("XDG_STATE_HOME").unwrap_or_else(|_| format!("{}/.local/state", env::var("HOME").unwrap_or_default()));
    file_in(Path::new(&state), name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_files_move_into_the_folder_once() {
        let t = env::temp_dir().join(format!("fili-state-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&t);
        fs::create_dir_all(t.join("filigrane")).unwrap();
        fs::write(t.join("filigrane/fili-file-view"), "grid").unwrap();
        let f = file_in(&t, "view");
        assert_eq!(f, t.join("filigrane/fili-file/view"));
        assert_eq!(fs::read_to_string(&f).unwrap(), "grid");
        assert!(!t.join("filigrane/fili-file-view").exists());
        // Nothing old: the path is just returned, and an existing new file is not replaced by a stale old one
        fs::write(t.join("filigrane/fili-file-view"), "list").unwrap();
        assert_eq!(fs::read_to_string(file_in(&t, "view")).unwrap(), "grid");
        let _ = fs::remove_dir_all(&t);
    }
}
