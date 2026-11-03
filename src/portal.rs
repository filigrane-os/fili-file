// xdg-desktop-portal backend for org.freedesktop.impl.portal.FileChooser (fili-file --portal): the D-Bus service
// is started on demand (dbus-1/services), answers the Open and Save requests of applications (Chrome...) by
// starting fili-file in chooser mode (pick.rs) and the FileManager1 requests (Show in folder), then exits after two minutes without a request.
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use zbus::{
    blocking::connection::Builder,
    interface,
    zvariant::{OwnedObjectPath, OwnedValue, Value},
};

const NAME: &str = "org.freedesktop.impl.portal.desktop.filigrane";
const IDLE: Duration = Duration::from_secs(120);

static BUSY: AtomicBool = AtomicBool::new(false);
static LAST: AtomicU64 = AtomicU64::new(0);
static COUNT: AtomicU64 = AtomicU64::new(0);

type Options = HashMap<String, OwnedValue>;
type Reply = (u32, HashMap<String, OwnedValue>);

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

fn value<T: TryFrom<Value<'static>>>(o: &Options, key: &str) -> Option<T> {
    let v = o.get(key)?.try_clone().ok()?;
    T::try_from(Value::from(v)).ok()
}

// Path sent as a zero-terminated byte array (ay)
fn path(o: &Options, key: &str) -> Option<String> {
    let mut b: Vec<u8> = value(o, key)?;
    while b.last() == Some(&0) {
        b.pop();
    }
    String::from_utf8(b).ok().filter(|s| !s.is_empty())
}

fn json(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push(' '),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

// a(sa(us)) -> [{"name":..,"globs":[..],"mimes":[..]}] (0: pattern, 1: MIME type)
fn filters_json(o: &Options) -> (String, Vec<String>) {
    let list: Vec<(String, Vec<(u32, String)>)> = value(o, "filters").unwrap_or_default();
    let names = list.iter().map(|(n, _)| n.clone()).collect();
    let body: Vec<String> = list
        .iter()
        .map(|(name, pats)| {
            let part = |kind: u32| pats.iter().filter(|(k, _)| *k == kind).map(|(_, p)| json(p)).collect::<Vec<_>>().join(",");
            format!("{{\"name\":{},\"globs\":[{}],\"mimes\":[{}]}}", json(name), part(0), part(1))
        })
        .collect();
    (format!("[{}]", body.join(",")), names)
}

fn run(kind: &str, title: &str, o: &Options) -> Reply {
    BUSY.store(true, Ordering::SeqCst);
    let reply = pick(kind, title, o);
    LAST.store(now(), Ordering::SeqCst);
    BUSY.store(false, Ordering::SeqCst);
    reply
}

fn pick(kind: &str, title: &str, o: &Options) -> Reply {
    let cancelled = (1, HashMap::new());
    let Ok(exe) = std::env::current_exe() else { return cancelled };
    let dir = std::env::var("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir());
    let out = dir.join(format!("fili-file-pick-{}-{}.txt", std::process::id(), COUNT.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_file(&out);
    let save_files: Vec<String> = value::<Vec<Vec<u8>>>(o, "files")
        .unwrap_or_default()
        .into_iter()
        .filter_map(|b| String::from_utf8(b).ok())
        .map(|s| s.trim_end_matches('\0').to_string())
        .collect();
    let mut args: Vec<String> = vec!["--out".into(), out.to_string_lossy().into_owned(), "--title".into(), title.into()];
    // Save existing files in a folder: choosing a folder
    let (mode, directory) = match kind {
        "save_files" => ("open", true),
        "save" => ("save", false),
        _ => ("open", value(o, "directory").unwrap_or(false)),
    };
    args.extend(["--pick".into(), mode.into()]);
    if directory {
        args.push("--directory".into());
    }
    if value(o, "multiple").unwrap_or(false) {
        args.push("--multiple".into());
    }
    if let Some(label) = value::<String>(o, "accept_label") {
        args.extend(["--accept".into(), label.replace('_', "")]);
    }
    let mut folder = path(o, "current_folder");
    let mut name = value::<String>(o, "current_name");
    if let Some(file) = path(o, "current_file") {
        let p = Path::new(&file);
        folder = folder.or_else(|| p.parent().map(|d| d.to_string_lossy().into_owned()));
        name = name.or_else(|| p.file_name().map(|n| n.to_string_lossy().into_owned()));
    }
    if let Some(f) = folder {
        args.extend(["--folder".into(), f]);
    }
    if let Some(n) = name {
        args.extend(["--name".into(), n]);
    }
    let (filters, names) = filters_json(o);
    if !names.is_empty() {
        args.extend(["--filters".into(), filters]);
        // current_filter: (name, patterns)
        let current: Option<(String, Vec<(u32, String)>)> = value(o, "current_filter");
        if let Some(i) = current.and_then(|(n, _)| names.iter().position(|x| *x == n)) {
            args.extend(["--filter".into(), i.to_string()]);
        }
    }
    let ran = Command::new(exe).args(&args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
    let text = std::fs::read_to_string(&out).unwrap_or_default();
    let _ = std::fs::remove_file(&out);
    if ran.is_err() {
        return cancelled;
    }
    let mut paths: Vec<String> = text.lines().filter(|l| !l.is_empty()).map(String::from).collect();
    // First line "# filter N": the chosen filter (not used by the portal), to remove from the paths
    if paths.first().is_some_and(|l| l.starts_with("# filter ")) {
        paths.remove(0);
    }
    if paths.is_empty() {
        return cancelled;
    }
    if kind == "save_files" {
        let base = paths[0].clone();
        paths = save_files.iter().map(|f| format!("{}/{f}", base.trim_end_matches('/'))).collect();
    }
    let uris: Vec<String> = paths.iter().map(|p| format!("file://{}", crate::ops::encode(p))).collect();
    let mut results = HashMap::new();
    if let Ok(v) = OwnedValue::try_from(Value::from(uris)) {
        results.insert("uris".to_string(), v);
    }
    (0, results)
}

struct Chooser;

#[interface(name = "org.freedesktop.impl.portal.FileChooser")]
impl Chooser {
    #[zbus(property)]
    fn version(&self) -> u32 {
        3
    }

    fn open_file(&self, _handle: OwnedObjectPath, _app_id: &str, _parent_window: &str, title: &str, options: Options) -> Reply {
        run("open", title, &options)
    }

    fn save_file(&self, _handle: OwnedObjectPath, _app_id: &str, _parent_window: &str, title: &str, options: Options) -> Reply {
        run("save", title, &options)
    }

    fn save_files(&self, _handle: OwnedObjectPath, _app_id: &str, _parent_window: &str, title: &str, options: Options) -> Reply {
        run("save_files", title, &options)
    }
}

// org.freedesktop.FileManager1 (Chrome "Show in folder", etc.): one fili-file window per request
struct FileManager;

impl FileManager {
    fn launch(&self, uris: &[String], extra: &[&str]) {
        let Ok(exe) = std::env::current_exe() else { return };
        for uri in uris.iter().filter(|u| u.starts_with("file://")) {
            let child = Command::new(&exe).args(extra).arg(uri).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
            if let Ok(mut child) = child {
                std::thread::spawn(move || child.wait());
            }
        }
        LAST.store(now(), Ordering::SeqCst);
    }
}

#[interface(name = "org.freedesktop.FileManager1")]
impl FileManager {
    // A file: its folder opens with it selected
    fn show_items(&self, uris: Vec<String>, _startup_id: &str) {
        self.launch(&uris, &[]);
    }

    fn show_folders(&self, uris: Vec<String>, _startup_id: &str) {
        self.launch(&uris, &[]);
    }

    fn show_item_properties(&self, uris: Vec<String>, _startup_id: &str) {
        self.launch(&uris, &["--info"]);
    }
}

pub fn serve() -> zbus::Result<()> {
    let _conn = Builder::session()?
        .name(NAME)?
        .name("org.freedesktop.FileManager1")?
        .serve_at("/org/freedesktop/portal/desktop", Chooser)?
        .serve_at("/org/freedesktop/FileManager1", FileManager)?
        .build()?;
    LAST.store(now(), Ordering::SeqCst);
    // Sleeps until the inactivity deadline (instead of waking up every 5 s); a request pushes the deadline back
    loop {
        let idle = now().saturating_sub(LAST.load(Ordering::SeqCst));
        if !BUSY.load(Ordering::SeqCst) && idle > IDLE.as_secs() {
            return Ok(());
        }
        let wait = if BUSY.load(Ordering::SeqCst) { IDLE.as_secs() } else { (IDLE.as_secs() + 1).saturating_sub(idle).max(1) };
        std::thread::sleep(Duration::from_secs(wait));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_to_json() {
        let list: Vec<(String, Vec<(u32, String)>)> = vec![("Images".into(), vec![(0, "*.png".into()), (1, "image/*".into())])];
        let mut o = Options::new();
        o.insert("filters".into(), OwnedValue::try_from(Value::from(list)).unwrap());
        let (j, names) = filters_json(&o);
        assert_eq!(j, r#"[{"name":"Images","globs":["*.png"],"mimes":["image/*"]}]"#);
        assert_eq!(names, ["Images"]);
    }

    #[test]
    fn byte_paths() {
        let mut o = Options::new();
        o.insert("current_folder".into(), OwnedValue::try_from(Value::from(b"/home/x\0".to_vec())).unwrap());
        assert_eq!(path(&o, "current_folder").as_deref(), Some("/home/x"));
        assert_eq!(path(&o, "absent"), None);
    }
}
