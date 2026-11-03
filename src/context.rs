// Context shared with the assistant (fili-ai): the application writes its useful state (folder, selection) to
// $XDG_RUNTIME_DIR/fili/context/<pid>.json. Nothing else runs: the file is read on demand by
// `fili-context`, which finds the one of the active window by its pid (niri). Common contract: see the filigrane-ui skill.
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicI32, Ordering},
        mpsc::{Sender, channel},
    },
    time::Duration,
};

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

// What the assistant must know about the application (common contract of the fili-* apps): the role, the meaning of the fields, the
// recommended way to act. Short: it goes into the prompt.
const ABOUT: &str = "File manager. location = folder shown (results = title if a result list from ctl show replaces its content), selection = paths the user selected (absolute). \
Act on files with normal shell tools (mv, cp, rg, sha256sum, tar...) using these paths; prefer the shell for batch work. \
fili-file shows changes live (inotify), so no refresh call is needed. To change files use fili-tx (mv, mv - for batches, trash, mkdir, cp), never bare mv/rm: fili-tx undo reverts the whole request. \
To take the user somewhere: fili-file ctl goto PATH (a file is selected in its folder). Menu commands (the fili-bar menu): fili-file ctl action new-folder|copy|paste|rename|trash|undo|reload|... acts on the user's selection. Every ctl command prints a JSON answer and exits 1 with the reason if it did not happen (unknown or disabled action, bad path). fili-file ctl state prints the window state (tabs, folder, selection, view, tasks, open dialog); fili-file ctl actions lists the commands with label, shortcut and whether they are enabled now. Commands with an argument skip their dialog: ctl action rename NAME | new-folder NAME | select-matching GLOB | compress NAME[.zip|.7z|.tar.gz|.tar.xz|.tar.zst] | extract PASSWORD | search TEXT | path PATH. Selection: ctl select < paths (replaces), select --add < paths, select --none, select --pattern GLOB (paths must be in the folder shown). ctl reveal PATH opens the parent and selects the item; ctl tab new [PATH]|close [N]|switch N|move FROM TO|detach [N] (the tab becomes its own window); ctl view list|icons; hidden on|off; preview on|off; sort name|size|modified|kind|created|permissions|owner [asc|desc]. ctl dialog accept|cancel (a name conflict also replace|skip|both) answers the open dialog; ctl width column NAME PX|reset and width preview PX|reset set the column and preview panel widths; action quick-look (Space) opens the quick look window. To show search results in this window: \
find them with fd/rg/find, then fili-file ctl show 'TITLE' < paths (one absolute path per line); both bring the window forward.";

// results: title of the list of results shown (empty: the normal folder)
// disabled: names of the menu commands that are impossible now (the fili-bar greys them out); the bar re-reads this file at
// the menu opening
pub fn publish(app: &str, location: &str, selection: &[String], results: &str, disabled: &[String]) {
    let Some(f) = file() else { return };
    let sel: Vec<String> = selection.iter().take(200).map(|p| json(p)).collect();
    let off: Vec<String> = disabled.iter().map(|d| json(d)).collect();
    let body = format!(
        "{{\"app\":{},\"about\":{},\"pid\":{},\"location\":{},\"results\":{},\"selection\":[{}],\"selection_count\":{},\"disabled\":[{}]}}\n",
        json(app),
        json(ABOUT),
        std::process::id(),
        json(location),
        json(results),
        sel.join(","),
        selection.len(),
        off.join(",")
    );
    let tmp = f.with_extension("tmp");
    if fs::write(&tmp, body).is_ok() {
        let _ = fs::rename(&tmp, &f);
    }
}

pub fn clear() {
    if let Some(f) = file() {
        let _ = fs::remove_file(f.with_extension("sock"));
        let _ = fs::remove_file(f);
    }
}

// Commands of the assistant to this instance: a socket <pid>.sock next to the context file. One connection = one
// command: first line "verb<TAB>argument", the rest (until the end of the stream) is the body (list of paths).
// The answer is a JSON line {"ok":true,...} or {"ok":false,"error":"..."} given by the interface (`reply`), not by
// the socket thread: the acknowledgement says what happened. The thread exists only in the application and sleeps on accept.
const REPLY_TIMEOUT: Duration = Duration::from_secs(5);

static PENDING: Mutex<Option<HashMap<i32, Sender<String>>>> = Mutex::new(None);
static NEXT_ID: AtomicI32 = AtomicI32::new(1);

// The interface answers command `id` (only once; no effect if the command expired)
pub fn reply(id: i32, text: &str) {
    let tx = PENDING.lock().ok().and_then(|mut p| p.as_mut()?.remove(&id));
    if let Some(tx) = tx {
        let _ = tx.send(text.to_string());
    }
}

// Hands the command to the interface (on_command receives the id to give back to `reply`) and waits for its answer
fn dispatch(text: &str, timeout: Duration, on_command: &dyn Fn(i32, String, String, String)) -> String {
    let (head, body) = text.split_once('\n').unwrap_or((text, ""));
    let (verb, arg) = head.split_once('\t').unwrap_or((head, ""));
    if verb.is_empty() {
        return "{\"ok\":false,\"error\":\"empty command\"}".into();
    }
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = channel();
    if let Ok(mut p) = PENDING.lock() {
        p.get_or_insert_with(HashMap::new).insert(id, tx);
    }
    on_command(id, verb.to_string(), arg.to_string(), body.to_string());
    let answer = rx.recv_timeout(timeout);
    if let Ok(mut p) = PENDING.lock() {
        if let Some(p) = p.as_mut() {
            p.remove(&id);
        }
    }
    answer.unwrap_or_else(|_| "{\"ok\":false,\"error\":\"no answer from the window (busy?)\"}".into())
}

pub fn serve(on_command: impl Fn(i32, String, String, String) + Send + 'static) {
    let Some(f) = file() else { return };
    let sock = f.with_extension("sock");
    let _ = fs::remove_file(&sock);
    let Ok(listener) = UnixListener::bind(&sock) else { return };
    // The socket is readable by the user only
    let _ = fs::set_permissions(&sock, std::os::unix::fs::PermissionsExt::from_mode(0o600));
    std::thread::spawn(move || {
        for mut c in listener.incoming().flatten() {
            let _ = c.set_read_timeout(Some(Duration::from_secs(2)));
            let mut text = String::new();
            let _ = (&mut c).take(4 << 20).read_to_string(&mut text);
            let answer = dispatch(&text, REPLY_TIMEOUT, &on_command);
            let _ = c.write_all(format!("{}\n", answer.replace('\n', " ")).as_bytes());
        }
    });
}

fn alive(pid: &str) -> bool {
    Path::new(&format!("/proc/{pid}")).is_dir()
}

// Instance targeted: the one of the requested pid (--pid: the fili-bar knows the active window), otherwise the one noted by Mod+Y
// ($FILI_CONTEXT_FILE), otherwise the last one started
fn target(pid: Option<&str>) -> Option<PathBuf> {
    let dir = PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR")?).join("fili/context");
    if let Some(pid) = pid.filter(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit())) {
        let s = dir.join(format!("{pid}.sock"));
        if alive(pid) && s.exists() {
            return Some(s);
        }
    }
    let noted = std::env::var_os("FILI_CONTEXT_FILE").and_then(|f| fs::read_to_string(f).ok()).filter(|t| t.contains("\"app\":\"fili-file\""));
    if let Some(t) = noted {
        let pid: String = t.split("\"pid\":").nth(1).unwrap_or("").chars().take_while(char::is_ascii_digit).collect();
        let s = dir.join(format!("{pid}.sock"));
        if alive(&pid) && s.exists() {
            return Some(s);
        }
    }
    let mut socks: Vec<_> = fs::read_dir(&dir)
        .ok()?
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "sock"))
        .filter(|e| alive(&e.path().file_stem().unwrap_or_default().to_string_lossy()))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    socks.sort();
    socks.pop().map(|(_, p)| p)
}

// Brings the window of the instance to the front (niri): we look for the id of the window that carries this pid
fn raise(pid: &str) {
    let Ok(out) = std::process::Command::new("niri").args(["msg", "-j", "windows"]).output() else { return };
    let t = String::from_utf8_lossy(&out.stdout);
    let Some(at) = t.find(&format!("\"pid\":{pid},")).or_else(|| t.find(&format!("\"pid\":{pid}}}"))) else { return };
    let Some(i) = t[..at].rfind("\"id\":") else { return };
    let id: String = t[i + 5..].chars().take_while(char::is_ascii_digit).collect();
    if !id.is_empty() {
        let _ = std::process::Command::new("niri").args(["msg", "action", "focus-window", "--id", &id]).status();
    }
}

const USAGE: &str = "usage: fili-file ctl [--pid PID] goto PATH | reveal PATH | show TITLE < paths | action NAME [ARG] | state | actions | \
select [--add | --none | --pattern GLOB] < paths | tab new [PATH] | tab close [N] | tab switch N | tab move FROM TO | tab detach [N] | view list|icons | hidden on|off | \
preview on|off | sort COLUMN [asc|desc] | dialog accept|cancel|replace|skip|both | width column NAME PX|reset | width preview PX|reset";

// Splits the --pid option from the rest of the arguments
fn split_pid(args: &[String]) -> (Option<String>, &[String]) {
    match args {
        [flag, pid, rest @ ..] if flag == "--pid" => (Some(pid.clone()), rest),
        _ => (None, args),
    }
}

// What the client sends: verb, argument (fields separated by a tab), body (list of paths on the standard
// input), and whether the absence of an instance allows starting one (going somewhere allows it, acting on a window does not)
struct Request {
    verb: String,
    arg: String,
    stdin: bool,
    launch: bool,
}

fn absolute(p: &str) -> String {
    std::path::absolute(p).map_or_else(|_| p.to_string(), |p| p.to_string_lossy().into_owned())
}

// Parses the command line of `ctl` (without the --pid); relative paths become absolute (caller's folder)
fn parse(args: &[String]) -> Result<Request, String> {
    let verb = args.first().ok_or(USAGE)?.as_str();
    let rest = &args[1..];
    let req = |arg: String, stdin: bool, launch: bool| Ok(Request { verb: verb.into(), arg, stdin, launch });
    let enumerated = |ok: &[&str]| -> Result<String, String> {
        match rest {
            [v] if ok.contains(&v.as_str()) => Ok(v.clone()),
            _ => Err(USAGE.into()),
        }
    };
    match verb {
        "goto" | "reveal" => match rest {
            // "network:" and "nearby:" are places of the sidebar, not paths
            [place] if verb == "goto" && (place == "network:" || place == "nearby:") => req(place.clone(), false, true),
            [path] if !path.is_empty() => req(absolute(path), false, true),
            _ => Err(USAGE.into()),
        },
        "show" => req(rest.first().cloned().unwrap_or_default(), true, true),
        // action NAME [ARGUMENT]: the argument skips the dialog of the command (rename, select-matching, compress...)
        "action" => match rest {
            [name] if !name.is_empty() => req(name.clone(), false, false),
            [name, arg @ ..] if !name.is_empty() => req(format!("{name}\t{}", arg.join(" ")), false, false),
            _ => Err(USAGE.into()),
        },
        "state" | "actions" if rest.is_empty() => req(String::new(), false, false),
        "select" => match rest {
            [] => req("set".into(), true, false),
            [f] if f == "--add" => req("add".into(), true, false),
            [f] if f == "--none" => req("none".into(), false, false),
            [f, glob] if f == "--pattern" && !glob.is_empty() => req(format!("pattern\t{glob}"), false, false),
            _ => Err(USAGE.into()),
        },
        "tab" => match rest {
            [c] if c == "new" || c == "close" => req(c.clone(), false, false),
            [c, path] if c == "new" => req(format!("new\t{}", absolute(path)), false, false),
            [c, n] if (c == "close" || c == "switch") && n.parse::<u32>().is_ok() => req(format!("{c}\t{n}"), false, false),
            [c] if c == "detach" => req("detach".into(), false, false),
            [c, n] if c == "detach" && n.parse::<u32>().is_ok() => req(format!("detach\t{n}"), false, false),
            [c, from, to] if c == "move" && from.parse::<u32>().is_ok() && to.parse::<u32>().is_ok() => req(format!("move\t{from}\t{to}"), false, false),
            _ => Err(USAGE.into()),
        },
        // width column NAME PX|reset, width preview PX|reset (widths of the columns and of the preview panel)
        "width" => match rest {
            [w, px] if w == "preview" && (px == "reset" || px.parse::<u32>().is_ok()) => req(format!("preview\t{px}"), false, false),
            [w, name, px] if w == "column" && !name.is_empty() && (px == "reset" || px.parse::<u32>().is_ok()) => req(format!("column\t{name}\t{px}"), false, false),
            _ => Err(USAGE.into()),
        },
        "dialog" => req(enumerated(&["accept", "cancel", "replace", "skip", "both"])?, false, false),
        "view" => req(enumerated(&["list", "icons"])?, false, false),
        "hidden" | "preview" => req(enumerated(&["on", "off"])?, false, false),
        "sort" => match rest {
            [col] => req(col.clone(), false, false),
            [col, dir] if dir == "asc" || dir == "desc" => req(format!("{col}\t{dir}"), false, false),
            _ => Err(USAGE.into()),
        },
        _ => Err(USAGE.into()),
    }
}

// fili-file ctl [--pid PID] <command> (see USAGE). Action commands (action, select, tab, view...) target an
// open window and never start one; goto, reveal and show start one if there is none.
// state: JSON of the state (tabs, folder, selection, tasks); actions: JSON of the commands (label, shortcut, enabled)
// Returns the JSON answer of the window; Err if it says {"ok":false,...}
pub fn ctl(args: &[String]) -> Result<String, String> {
    let (pid, args) = split_pid(args);
    let Request { verb, arg, stdin, launch } = parse(args)?;
    let mut body = String::new();
    if stdin {
        let _ = std::io::stdin().take(4 << 20).read_to_string(&mut body);
        // Relative paths: relative to the caller's folder, except for show (list already absolute, free title)
        if verb == "select" {
            body = body.lines().filter(|l| !l.is_empty()).map(absolute).collect::<Vec<_>>().join("\n");
        }
    }
    let mut sock = target(pid.as_deref());
    // A command that acts on the window or reads its state targets an open window: no instance to start for it
    if sock.is_none() && !launch {
        return Err("no fili-file window".into());
    }
    if sock.is_none() {
        // No instance: one is started, then we wait for its socket
        std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
            .current_dir(if verb == "goto" || verb == "reveal" { "/" } else { "." })
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        for _ in 0..50 {
            std::thread::sleep(Duration::from_millis(100));
            sock = target(pid.as_deref());
            if sock.is_some() {
                break;
            }
        }
    }
    let sock = sock.ok_or("fili-file does not answer")?;
    let mut c = UnixStream::connect(&sock).map_err(|e| e.to_string())?;
    c.set_read_timeout(Some(Duration::from_secs(5))).ok();
    c.write_all(format!("{verb}\t{arg}\n{body}").as_bytes()).map_err(|e| e.to_string())?;
    c.shutdown(std::net::Shutdown::Write).ok();
    let mut ack = String::new();
    let _ = c.read_to_string(&mut ack);
    let ack = ack.trim().to_string();
    if launch || matches!(verb.as_str(), "action" | "select" | "tab" | "dialog") {
        raise(&sock.file_stem().unwrap_or_default().to_string_lossy());
    }
    if ack.starts_with("{\"ok\":true") { Ok(ack) } else { Err(ack_error(&ack)) }
}

// Error message of an answer {"ok":false,"error":"..."} (the raw answer if it has another shape)
fn ack_error(ack: &str) -> String {
    ack.split_once("\"error\":\"")
        .and_then(|(_, e)| e.rsplit_once("\"}"))
        .map_or_else(|| if ack.is_empty() { "no answer".into() } else { ack.to_string() }, |(e, _)| e.replace("\\\"", "\"").replace("\\\\", "\\"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn pid_is_optional_before_the_verb() {
        let a = v(&["--pid", "123", "action", "copy"]);
        let (pid, rest) = split_pid(&a);
        assert_eq!(pid.as_deref(), Some("123"));
        assert_eq!(rest, &v(&["action", "copy"])[..]);
        let b = v(&["goto", "/tmp"]);
        let (pid, rest) = split_pid(&b);
        assert_eq!(pid, None);
        assert_eq!(rest, &b[..]);
    }

    #[test]
    fn the_answer_comes_from_the_interface() {
        let r = dispatch("action\tcopy\n", Duration::from_secs(2), &|id, verb, arg, body| {
            assert_eq!((verb.as_str(), arg.as_str(), body.as_str()), ("action", "copy", ""));
            std::thread::spawn(move || reply(id, "{\"ok\":true}"));
        });
        assert_eq!(r, "{\"ok\":true}");
    }

    #[test]
    fn failure_is_reported_without_an_answer() {
        let r = dispatch("goto\t/tmp\n", Duration::from_millis(50), &|_, _, _, _| {});
        assert!(r.starts_with("{\"ok\":false"), "{r}");
        // A late answer has no effect
        reply(1_000_000, "{\"ok\":true}");
    }

    #[test]
    fn empty_command_refused_and_body_passed_on() {
        assert!(dispatch("", Duration::from_millis(50), &|_, _, _, _| panic!("not called")).contains("empty"));
        let r = dispatch("show\tT\n/a\n/b\n", Duration::from_secs(2), &|id, _, arg, body| {
            assert_eq!((arg.as_str(), body.as_str()), ("T", "/a\n/b\n"));
            reply(id, "{\"ok\":true}");
        });
        assert_eq!(r, "{\"ok\":true}");
    }

    #[test]
    fn error_message_is_extracted() {
        assert_eq!(ack_error("{\"ok\":false,\"error\":\"disabled: no selection\"}"), "disabled: no selection");
        assert_eq!(ack_error("{\"ok\":false,\"error\":\"a \\\"b\\\"\"}"), "a \"b\"");
        assert_eq!(ack_error(""), "no answer");
    }

    #[test]
    fn command_parsing() {
        let ok = |a: &[&str]| parse(&v(a)).map(|r| (r.verb, r.arg, r.stdin, r.launch));
        let t = |verb: &str, arg: &str, stdin: bool, launch: bool| Ok((verb.to_string(), arg.to_string(), stdin, launch));
        assert_eq!(ok(&["goto", "/tmp"]), t("goto", "/tmp", false, true));
        assert_eq!(ok(&["goto", "network:"]), t("goto", "network:", false, true));
        assert_eq!(ok(&["reveal", "/tmp/a"]), t("reveal", "/tmp/a", false, true));
        assert_eq!(ok(&["action", "copy"]), t("action", "copy", false, false));
        assert_eq!(ok(&["action", "rename", "a b.txt"]), t("action", "rename\ta b.txt", false, false));
        assert_eq!(ok(&["state"]), t("state", "", false, false));
        assert_eq!(ok(&["select"]), t("select", "set", true, false));
        assert_eq!(ok(&["select", "--add"]), t("select", "add", true, false));
        assert_eq!(ok(&["select", "--none"]), t("select", "none", false, false));
        assert_eq!(ok(&["select", "--pattern", "*.pdf"]), t("select", "pattern\t*.pdf", false, false));
        assert_eq!(ok(&["tab", "new", "/tmp"]), t("tab", "new\t/tmp", false, false));
        assert_eq!(ok(&["tab", "switch", "2"]), t("tab", "switch\t2", false, false));
        assert_eq!(ok(&["tab", "move", "0", "2"]), t("tab", "move\t0\t2", false, false));
        assert_eq!(ok(&["tab", "detach"]), t("tab", "detach", false, false));
        assert_eq!(ok(&["tab", "detach", "1"]), t("tab", "detach\t1", false, false));
        assert_eq!(ok(&["tab", "close"]), t("tab", "close", false, false));
        assert_eq!(ok(&["view", "icons"]), t("view", "icons", false, false));
        assert_eq!(ok(&["dialog", "replace"]), t("dialog", "replace", false, false));
        assert_eq!(ok(&["width", "column", "size", "120"]), t("width", "column\tsize\t120", false, false));
        assert_eq!(ok(&["width", "preview", "reset"]), t("width", "preview\treset", false, false));
        assert_eq!(ok(&["hidden", "on"]), t("hidden", "on", false, false));
        assert_eq!(ok(&["sort", "size", "desc"]), t("sort", "size\tdesc", false, false));
        assert_eq!(ok(&["sort", "size"]), t("sort", "size", false, false));
        for bad in [&["tab", "switch", "x"][..], &["tab", "switch"], &["view", "grid"], &["dialog", "ok"], &["width", "size", "1"], &["width", "preview", "-5"], &["width", "column", "size"], &["hidden", "yes"], &["sort", "size", "up"], &["select", "--bad"], &["goto"], &["state", "x"], &["inconnue"]] {
            assert!(parse(&v(bad)).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn command_without_a_name_is_refused() {
        assert!(ctl(&v(&["action"])).is_err());
        assert!(ctl(&v(&["--pid", "1", "inconnue", "x"])).is_err());
    }
}
