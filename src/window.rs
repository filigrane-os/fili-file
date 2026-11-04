// Window management of the compositor, driven from the window panel of the toolbar. Only niri is supported: its
// actions are run on the focused window (the one the user just clicked in) through `niri msg action`. The names are
// checked against a fixed list, so nothing but a window action can be run from the interface.
use std::{
    process::{Command, Stdio},
    time::Duration,
};

// Every action the window panel offers (qml/WindowPanel.qml), in niri's own spelling
pub const ACTIONS: &[&str] = &[
    "fullscreen-window",
    "toggle-windowed-fullscreen",
    "maximize-column",
    "maximize-window-to-edges",
    "expand-column-to-available-width",
    "switch-preset-column-width",
    "switch-preset-window-height",
    "reset-window-height",
    "center-column",
    "center-window",
    "toggle-window-floating",
    "move-column-left",
    "move-column-right",
    "move-window-up",
    "move-window-down",
    "consume-or-expel-window-left",
    "consume-or-expel-window-right",
    "toggle-column-tabbed-display",
    "move-window-to-workspace-up",
    "move-window-to-workspace-down",
    "move-window-to-monitor-left",
    "move-window-to-monitor-right",
    "screenshot-window",
    "toggle-overview",
];

// Whether the session runs niri (its socket is announced in the environment) and the `niri` program is there
pub fn available() -> bool {
    std::env::var_os("NIRI_SOCKET").is_some_and(|s| !s.is_empty()) && crate::tools::available("niri")
}

// Runs one action of the list without waiting for it; false when the name is not on the list or niri is absent
pub fn run(name: &str) -> bool {
    if !ACTIONS.contains(&name) || !available() {
        return false;
    }
    Command::new("niri").args(["msg", "action", name]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().is_ok()
}

// A window of the session as niri lists it
#[derive(Clone, Debug, PartialEq)]
pub struct Win {
    pub id: u64,
    pub app: String, // app-id, lowercase
    pub focused: bool,
}

fn windows() -> Vec<Win> {
    let Ok(out) = Command::new("niri").args(["msg", "windows"]).stdin(Stdio::null()).stderr(Stdio::null()).output() else { return Vec::new() };
    parse_windows(&String::from_utf8_lossy(&out.stdout))
}

fn parse_windows(text: &str) -> Vec<Win> {
    let mut list: Vec<Win> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Window ID ") {
            let id = rest.split(':').next().and_then(|n| n.trim().parse().ok()).unwrap_or(0);
            list.push(Win { id, app: String::new(), focused: rest.contains("(focused)") });
        } else if let Some(rest) = line.strip_prefix("App ID: ")
            && let Some(last) = list.last_mut()
        {
            last.app = rest.trim().trim_matches('"').to_lowercase();
        }
    }
    list
}

// The windows that exist before an application is launched, so that a new one can be told from an old one
pub fn snapshot() -> Vec<Win> {
    if available() { windows() } else { Vec::new() }
}

// After launching an application (Open With): a new window gets the focus from niri by itself, but an application that
// was already running just opens the file in its existing window, and Wayland does not let it take the focus. So when
// the application already had a window (an app-id from `names`) and no new window shows up right away, that window is
// brought to the front, unless the user has already moved to another window in the meantime.
pub fn raise_existing(before: Vec<Win>, names: Vec<String>) {
    let Some(existing) = before.iter().rev().find(|w| names.contains(&w.app)).map(|w| w.id) else { return };
    let from = before.iter().find(|w| w.focused).map(|w| w.id);
    std::thread::spawn(move || {
        for _ in 0..4 {
            std::thread::sleep(Duration::from_millis(300));
            let now = windows();
            if now.iter().any(|w| !before.iter().any(|b| b.id == w.id)) || now.iter().find(|w| w.focused).map(|w| w.id) != from {
                return;
            }
        }
        let _ = Command::new("niri").args(["msg", "action", "focus-window", "--id", &existing.to_string()]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_window_list() {
        let text = "Window ID 7: (focused)\n  Title: \"a\"\n  App ID: \"Foot\"\n  PID: 1\n\nWindow ID 9:\n  Title: \"b\"\n  App ID: \"org.gnome.TextEditor\"\n";
        assert_eq!(
            parse_windows(text),
            vec![Win { id: 7, app: "foot".into(), focused: true }, Win { id: 9, app: "org.gnome.texteditor".into(), focused: false }]
        );
        assert!(parse_windows("").is_empty());
    }

    #[test]
    fn only_listed_actions_run() {
        assert!(!run("quit"));
        assert!(!run("spawn-sh"));
        assert!(!run(""));
    }

    // Every action the panel shows is on the list, and every listed action is in the panel
    #[test]
    fn the_panel_and_the_list_agree() {
        let qml = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/qml/WindowPanel.qml")).unwrap();
        let used: Vec<&str> = qml.split("action: \"").skip(1).filter_map(|t| t.split('"').next()).collect();
        for u in &used {
            assert!(ACTIONS.contains(u), "{u} is shown but not allowed");
        }
        for a in ACTIONS {
            assert!(used.contains(a), "{a} is allowed but not shown");
        }
    }
}
