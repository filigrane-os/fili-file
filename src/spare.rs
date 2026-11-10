// Resident mode ("spare"): with `resident = true` in fili-file.toml, a spare Casier is kept loaded in the background
// (`fili-file --spare`: interface built, window hidden, no folder read). Opening Casier then only claims the spare,
// which shows its window at once, and a new spare loads for the next time. It costs memory (see the setting's help in
// Settings); without the setting nothing runs. No Qt here: tested by cargo test.
use std::{
    env, fs,
    io::{Read, Write},
    os::unix::{net::{UnixListener, UnixStream}, process::CommandExt},
    path::PathBuf,
    process::{Command, Stdio},
    sync::{OnceLock, atomic::{AtomicBool, Ordering}},
    time::Duration,
};

// Variables of the launch that the claimed window takes over (focus token for the compositor, search path, terminal)
const LAUNCH_ENV: [&str; 4] = ["XDG_ACTIVATION_TOKEN", "DESKTOP_STARTUP_ID", "PATH", "TERMINAL"];
// Variable that makes a new spare wait this many seconds before it loads (a launch just finished: not during its start)
const DELAY_ENV: &str = "FILI_SPARE_DELAY";
// Identity of the binary the spare runs, taken when it binds: a claim from another build (upgrade) is refused
static EXE_ID: OnceLock<String> = OnceLock::new();
// The first folder of a waiting spare was loaded without recording the visit: the claim records it
static WARM: AtomicBool = AtomicBool::new(false);

// A spare that has not been claimed yet (nothing it does counts as use: no visit is recorded)
pub fn waiting() -> bool {
    is_spare() && CLAIM.get().is_none()
}

// Only as the first argument, like the other subcommands (a folder called "--spare" is not a spare)
pub fn is_spare() -> bool {
    is_spare_args(&env::args().collect::<Vec<_>>())
}

fn is_spare_args(args: &[String]) -> bool {
    args.get(1).is_some_and(|a| a == "--spare")
}

// The waiting spare loaded its first folder without recording the visit
pub fn note_warm() {
    WARM.store(true, Ordering::SeqCst);
}

// True once, after the claim, if the visit of the warm folder is still to record
pub fn take_warm() -> bool {
    !waiting() && WARM.swap(false, Ordering::SeqCst)
}

// The directory is private (0700) from its creation: no other user can reach the socket
fn socket() -> Option<PathBuf> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    let dir = PathBuf::from(env::var_os("XDG_RUNTIME_DIR")?).join("fili");
    fs::DirBuilder::new().recursive(true).mode(0o700).create(&dir).ok()?;
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).ok()?;
    Some(dir.join("fili-file-spare.sock"))
}

// The running binary, even if it was replaced since (Linux appends " (deleted)")
fn exe_path() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    let text = exe.to_string_lossy();
    Some(text.strip_suffix(" (deleted)").map_or(exe.clone(), PathBuf::from))
}

// Device, inode and modification time of the installed binary: changes when an update replaces it
fn exe_id() -> String {
    use std::os::unix::fs::MetadataExt;
    exe_path().and_then(|p| fs::metadata(p).ok()).map_or(String::new(), |m| format!("{}:{}:{}", m.dev(), m.ino(), m.mtime()))
}

// What a claim carries: the arguments and the working folder of the launch that the spare takes over
static CLAIM: OnceLock<(Vec<String>, String)> = OnceLock::new();

// Arguments of this launch (the claim's if the spare was claimed)
pub fn args() -> Vec<String> {
    CLAIM.get().map_or_else(|| env::args().skip(1).collect(), |(a, _)| a.clone())
}

pub fn cwd() -> Option<String> {
    CLAIM.get().map(|(_, c)| c.clone()).or_else(|| env::current_dir().ok().map(|d| d.to_string_lossy().into_owned()))
}

// A launch that a spare can take over: no file chooser, no option other than --info
pub fn claimable(args: &[String]) -> bool {
    args.iter().all(|a| a == "--info" || !a.starts_with("--")) && args.iter().filter(|a| !a.starts_with("--")).count() <= 1
}

// Message: verb NUL binary id NUL cwd NUL number of variables NUL KEY=VALUE... NUL arg NUL arg ...
fn encode(verb: &str, id: &str, cwd: &str, env: &[(String, String)], args: &[String]) -> Vec<u8> {
    let mut parts = vec![verb.to_string(), id.to_string(), cwd.to_string(), env.len().to_string()];
    parts.extend(env.iter().map(|(k, v)| format!("{k}={v}")));
    parts.extend(args.iter().cloned());
    parts.join("\0").into_bytes()
}

struct Message {
    verb: String,
    id: String,
    cwd: String,
    env: Vec<(String, String)>,
    args: Vec<String>,
}

fn decode(raw: &[u8]) -> Message {
    let text = String::from_utf8_lossy(raw);
    let mut it = text.split('\0').map(String::from);
    let verb = it.next().unwrap_or_default();
    let id = it.next().unwrap_or_default();
    let cwd = it.next().unwrap_or_default();
    let n: usize = it.next().and_then(|n| n.parse().ok()).unwrap_or(0);
    let env = it.by_ref().take(n).filter_map(|kv| kv.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))).collect();
    Message { verb, id, cwd, env, args: it.collect() }
}

fn ask(msg: &[u8]) -> bool {
    let Some(sock) = socket() else { return false };
    let Ok(mut c) = UnixStream::connect(sock) else { return false };
    c.set_read_timeout(Some(Duration::from_secs(5))).ok();
    if c.write_all(msg).is_err() {
        return false;
    }
    c.shutdown(std::net::Shutdown::Write).ok();
    let mut ack = String::new();
    let _ = c.read_to_string(&mut ack);
    ack.trim() == "ok"
}

// Hands the launch to the spare: true if it took it (nothing else to do), false to start normally
pub fn claim(args: &[String]) -> bool {
    let cwd = env::current_dir().map_or(String::new(), |d| d.to_string_lossy().into_owned());
    let env: Vec<(String, String)> = LAUNCH_ENV.iter().filter_map(|k| env::var(k).ok().map(|v| (k.to_string(), v))).collect();
    ask(&encode("claim", &exe_id(), &cwd, &env, args))
}

// Resident mode on? The choice made in Settings (state file, key resident) takes precedence over `resident` of fili-file.toml
pub fn enabled() -> bool {
    match crate::bridge::ui_get(&fs::read_to_string(crate::bridge::ui_file()).unwrap_or_default(), "resident").as_str() {
        "1" => true,
        "0" => false,
        _ => crate::config::load().resident,
    }
}

// `fili-file spare on|off|status|auto|start|stop`: on / off = the setting (and the spare at once), status = "on" or
// "off", auto = start the spare if the setting is on (session start), start / stop = the spare only
pub fn control(verb: &str) -> i32 {
    match verb {
        "on" => {
            crate::bridge::ui_store("resident", "1");
            start();
            0
        }
        "off" => {
            crate::bridge::ui_store("resident", "0");
            ask(&encode("quit", "", "", &[], &[]));
            0
        }
        "status" => {
            println!("{}", if enabled() { "on" } else { "off" });
            0
        }
        "stop" => {
            ask(&encode("quit", "", "", &[], &[]));
            0
        }
        "start" => {
            start();
            0
        }
        "auto" => {
            if enabled() {
                start();
            }
            0
        }
        _ => {
            eprintln!("fili-file spare: on, off, status, auto, start or stop");
            2
        }
    }
}

// A detached spare (it exits at once if another one lives)
pub fn start() {
    spawn(0);
}

// The same, loading after a delay: it lives in its own process, so it does not depend on this one staying alive
pub fn start_later(secs: u64) {
    spawn(secs);
}

fn spawn(delay: u64) {
    if let Some(exe) = exe_path() {
        let mut cmd = Command::new(exe);
        cmd.arg("--spare").current_dir("/").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).process_group(0);
        if delay > 0 {
            cmd.env(DELAY_ENV, delay.to_string());
        }
        let _ = cmd.spawn();
    }
}

// In the spare, first thing: waits the delay asked by start_later
pub fn wait_delay() {
    if let Some(s) = env::var(DELAY_ENV).ok().and_then(|v| v.parse::<u64>().ok()) {
        std::thread::sleep(Duration::from_secs(s));
        unsafe { env::remove_var(DELAY_ENV) };
    }
}

// In the spare, before Qt: takes the socket. None if a spare already lives (this one has nothing to do).
pub fn bind() -> Option<UnixListener> {
    let sock = socket()?;
    if sock.exists() {
        if UnixStream::connect(&sock).is_ok() {
            return None;
        }
        let _ = fs::remove_file(&sock);
    }
    let l = UnixListener::bind(&sock).ok()?;
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(&sock, fs::Permissions::from_mode(0o600));
    let _ = EXE_ID.set(exe_id());
    Some(l)
}

// Thread of the spare, blocked on accept: a claim calls `on_claim` (the window shows), then the socket is freed and
// the next spare starts; "quit" ends the process. A claim from a launcher that gave up waiting (it started its own
// window) is dropped: the acknowledgement is written first, and fails if nobody listens. A claim from another build
// of the binary (an update replaced it) is refused: the spare leaves, the launch starts normally.
pub fn serve(listener: UnixListener, on_claim: impl Fn() + Send + 'static) {
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            let Ok(mut c) = conn else { continue };
            c.set_read_timeout(Some(Duration::from_secs(2))).ok();
            let mut raw = Vec::new();
            let _ = (&mut c).take(1 << 16).read_to_end(&mut raw);
            let msg = decode(&raw);
            match msg.verb.as_str() {
                "claim" => {
                    if EXE_ID.get().is_some_and(|id| *id != msg.id) {
                        free_socket();
                        let _ = c.write_all(b"stale\n");
                        std::process::exit(0);
                    }
                    if c.write_all(b"ok\n").is_err() {
                        continue;
                    }
                    for k in LAUNCH_ENV {
                        match msg.env.iter().find(|(n, _)| n == k) {
                            Some((_, v)) => unsafe { env::set_var(k, v) },
                            None if k.starts_with("XDG_ACT") || k.starts_with("DESKTOP_") => unsafe { env::remove_var(k) },
                            None => {}
                        }
                    }
                    let _ = CLAIM.set((msg.args, msg.cwd));
                    free_socket();
                    on_claim();
                    if enabled() {
                        start();
                    }
                    return;
                }
                "quit" => {
                    free_socket();
                    let _ = c.write_all(b"ok\n");
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    });
}

fn free_socket() {
    if let Some(sock) = socket() {
        let _ = fs::remove_file(sock);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(l: &[&str]) -> Vec<String> {
        l.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn message_round_trip() {
        let env = vec![("XDG_ACTIVATION_TOKEN".to_string(), "t=k".to_string())];
        let m = decode(&encode("claim", "1:2:3", "/home/a", &env, &a(&["--info", "/tmp/x y"])));
        assert_eq!((m.verb.as_str(), m.id.as_str(), m.cwd.as_str()), ("claim", "1:2:3", "/home/a"));
        assert_eq!(m.env, env);
        assert_eq!(m.args, a(&["--info", "/tmp/x y"]));
        let m = decode(&encode("quit", "", "", &[], &[]));
        assert_eq!(m.verb, "quit");
        assert!(m.env.is_empty() && m.args.is_empty());
    }

    #[test]
    fn what_a_spare_can_take() {
        assert!(claimable(&a(&[])));
        assert!(claimable(&a(&["/tmp"])));
        assert!(claimable(&a(&["--info", "/tmp/f"])));
        assert!(!claimable(&a(&["--pick", "open"])));
        assert!(!claimable(&a(&["/a", "/b"])));
        assert!(!claimable(&a(&["--portal"])));
        assert!(!claimable(&a(&["--spare"])));
    }

    #[test]
    fn spare_flag_only_first() {
        assert!(is_spare_args(&a(&["fili-file", "--spare"])));
        assert!(!is_spare_args(&a(&["fili-file", "/dir", "--spare"])));
        assert!(!is_spare_args(&a(&["fili-file", "/dir"])));
        assert!(!is_spare_args(&a(&["fili-file"])));
    }

    #[test]
    fn warm_visit_is_recorded_once_after_the_claim() {
        // Not claimed: the spare still waits, nothing to record yet
        note_warm();
        assert!(!is_spare() || !take_warm());
        // A normal launch is never "waiting": the flag is taken once
        if !is_spare() {
            assert!(take_warm());
            assert!(!take_warm());
        }
    }

    #[test]
    fn socket_is_private_and_single() {
        let dir = std::env::temp_dir().join(format!("fili-spare-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        // The only test touching the environment
        unsafe { env::set_var("XDG_RUNTIME_DIR", &dir) };
        use std::os::unix::fs::PermissionsExt;
        let first = bind().expect("first spare takes the socket");
        let mode = |p: PathBuf| fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(dir.join("fili")), 0o700);
        assert_eq!(mode(dir.join("fili/fili-file-spare.sock")), 0o600);
        // The listener lives: a second spare has nothing to do
        let l = first;
        std::thread::spawn(move || {
            let _ = l.accept();
        });
        assert!(bind().is_none());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn binary_id_is_stable() {
        assert!(!exe_id().is_empty());
        assert_eq!(exe_id(), exe_id());
    }
}
