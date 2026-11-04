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

// First numbers of the array that follows `"key":[` in niri's JSON (compact), for example `"window_size":[1431,567]`
fn numbers(json: &str, key: &str, count: usize) -> Option<Vec<f64>> {
    let at = json.find(&format!("\"{key}\":["))? + key.len() + 4;
    let list: Vec<f64> = json[at..].split(']').next()?.split(',').filter_map(|n| n.trim().parse().ok()).collect();
    (list.len() >= count).then_some(list)
}

fn number(json: &str, key: &str) -> Option<f64> {
    let at = json.find(&format!("\"{key}\":"))? + key.len() + 3;
    json[at..].split([',', '}']).next()?.trim().parse().ok()
}

// Which panel actions describe the focused window as it is now (their names, comma separated): floating, filling the
// width of the screen (maximize column, fill the width), covering the whole screen (maximize to edges). niri does not
// report the rest (centered, tabbed, presets), so the panel cannot show it.
pub fn state() -> String {
    if !available() {
        return String::new();
    }
    let run = |args: &[&str]| Command::new("niri").args(args).stdin(Stdio::null()).stderr(Stdio::null()).output().map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default();
    state_of(&run(&["msg", "-j", "focused-window"]), &run(&["msg", "-j", "focused-output"])).join(",")
}

fn state_of(window: &str, output: &str) -> Vec<&'static str> {
    let mut on = Vec::new();
    let floating = window.contains("\"is_floating\":true");
    if floating {
        on.push("toggle-window-floating");
    }
    // The size that counts is the logical one (the "modes" of the output also have a width and a height)
    let logical = output.find("\"logical\"").map_or("", |i| &output[i..]);
    let (Some(size), Some(out_w), Some(out_h)) = (numbers(window, "window_size", 2), number(logical, "width"), number(logical, "height")) else { return on };
    if !floating {
        if size[0] >= out_w * 0.95 {
            on.push("maximize-column");
            on.push("expand-column-to-available-width");
        }
        if size[0] >= out_w - 1.0 && size[1] >= out_h - 1.0 {
            on.push("maximize-window-to-edges");
        }
    }
    on
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

// When a file is opened with an application that already has a window, the application just opens the file in that
// window, and Wayland does not let it take the focus. So the existing window (an app-id from `names`, among the
// windows listed before the launch) is brought to the front at once. If the application opens a new window instead,
// niri gives that one the focus a moment later, so nothing is lost.
pub fn raise_existing(before: &[Win], names: &[String]) {
    let Some(existing) = before.iter().rev().find(|w| names.contains(&w.app)).map(|w| w.id) else { return };
    std::thread::spawn(move || {
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
    fn state_of_a_window() {
        let out = r#"{"name":"eDP-1","modes":[{"width":2560,"height":1600}],"logical":{"x":0,"y":0,"width":1462,"height":914,"scale":1.75}}"#;
        let tiled = r#"{"id":1,"is_floating":false,"layout":{"window_size":[1431,567],"tile_size":[1430.8,566.8]}}"#;
        let full = r#"{"id":1,"is_floating":false,"layout":{"window_size":[1462,914]}}"#;
        let small = r#"{"id":1,"is_floating":true,"layout":{"window_size":[700,500]}}"#;
        assert_eq!(state_of(tiled, out), vec!["maximize-column", "expand-column-to-available-width"]);
        assert_eq!(state_of(full, out), vec!["maximize-column", "expand-column-to-available-width", "maximize-window-to-edges"]);
        assert_eq!(state_of(small, out), vec!["toggle-window-floating"]);
        assert!(state_of("", "").is_empty());
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
