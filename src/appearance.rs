// Light or dark mode from the desktop when Filigrane's own state file (~/.local/state/filigrane/theme) is absent:
// the freedesktop appearance setting, read through the settings portal (org.freedesktop.portal.Settings), which GNOME,
// KDE, sway-based desktops with xdg-desktop-portal-gtk/gnome/kde and Filigrane all provide. Dark when nothing answers.
use zbus::{
    blocking::{Connection, Proxy},
    zvariant::{OwnedValue, Value},
};

const NAMESPACE: &str = "org.freedesktop.appearance";
const KEY: &str = "color-scheme";

fn proxy(conn: &Connection) -> Option<Proxy<'_>> {
    Proxy::new(conn, "org.freedesktop.portal.Desktop", "/org/freedesktop/portal/desktop", "org.freedesktop.portal.Settings").ok()
}

// color-scheme: 0 no preference, 1 prefers dark, 2 prefers light (some portals wrap the value in a second variant)
fn is_light(v: &Value<'_>) -> Option<bool> {
    match v {
        Value::U32(n) => Some(*n == 2),
        Value::Value(inner) => is_light(inner),
        _ => None,
    }
}

// Whether the desktop prefers a light appearance; None when no portal answers
pub fn prefers_light() -> Option<bool> {
    let conn = Connection::session().ok()?;
    let v: OwnedValue = proxy(&conn)?.call("ReadOne", &(NAMESPACE, KEY)).ok()?;
    is_light(&v)
}

// Calls `changed(light)` each time the desktop changes its color scheme; returns when the portal goes away.
// Blocks: run it in a thread.
pub fn watch(mut changed: impl FnMut(bool)) {
    let Ok(conn) = Connection::session() else { return };
    let Some(proxy) = proxy(&conn) else { return };
    let Ok(signals) = proxy.receive_signal("SettingChanged") else { return };
    for msg in signals {
        let Ok((ns, key, value)) = msg.body().deserialize::<(String, String, OwnedValue)>() else { continue };
        if ns == NAMESPACE && key == KEY {
            if let Some(light) = is_light(&value) {
                changed(light);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_scheme_values() {
        assert_eq!(is_light(&Value::U32(0)), Some(false));
        assert_eq!(is_light(&Value::U32(1)), Some(false));
        assert_eq!(is_light(&Value::U32(2)), Some(true));
        assert_eq!(is_light(&Value::Value(Box::new(Value::U32(2)))), Some(true));
        assert_eq!(is_light(&Value::Bool(true)), None);
    }
}
