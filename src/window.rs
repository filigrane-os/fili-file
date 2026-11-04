// Window management of the compositor, driven from the window panel of the toolbar. Only niri is supported: its
// actions are run on the focused window (the one the user just clicked in) through `niri msg action`. The names are
// checked against a fixed list, so nothing but a window action can be run from the interface.
use std::process::{Command, Stdio};

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

#[cfg(test)]
mod tests {
    use super::*;

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
