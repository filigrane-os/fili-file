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
    sync::OnceLock,
    time::Duration,
};

// A spare that has not been claimed yet (nothing it does counts as use: no visit is recorded)
pub fn waiting() -> bool {
    is_spare() && CLAIM.get().is_none()
}

pub fn is_spare() -> bool {
    env::args().skip(1).any(|a| a == "--spare")
}

fn socket() -> Option<PathBuf> {
    let dir = PathBuf::from(env::var_os("XDG_RUNTIME_DIR")?).join("fili");
    fs::create_dir_all(&dir).ok()?;
    Some(dir.join("fili-file-spare.sock"))
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

// Message "claim" NUL cwd NUL arg NUL arg ...
fn encode(verb: &str, cwd: &str, args: &[String]) -> Vec<u8> {
    let mut parts = vec![verb.to_string(), cwd.to_string()];
    parts.extend(args.iter().cloned());
    parts.join("\0").into_bytes()
}

fn decode(raw: &[u8]) -> (String, String, Vec<String>) {
    let text = String::from_utf8_lossy(raw);
    let mut it = text.split('\0').map(String::from);
    (it.next().unwrap_or_default(), it.next().unwrap_or_default(), it.collect())
}

fn ask(msg: &[u8]) -> bool {
    let Some(sock) = socket() else { return false };
    let Ok(mut c) = UnixStream::connect(sock) else { return false };
    c.set_read_timeout(Some(Duration::from_secs(2))).ok();
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
    ask(&encode("claim", &cwd, args))
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
            ask(&encode("quit", "", &[]));
            0
        }
        "status" => {
            println!("{}", if enabled() { "on" } else { "off" });
            0
        }
        "stop" => {
            ask(&encode("quit", "", &[]));
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
    if let Ok(exe) = env::current_exe() {
        let _ = Command::new(exe).arg("--spare").current_dir("/").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).process_group(0).spawn();
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
    Some(l)
}

// Thread of the spare, blocked on accept: a claim calls `on_claim` (the window shows), then the socket is freed and
// the next spare starts; "quit" ends the process.
pub fn serve(listener: UnixListener, on_claim: impl Fn() + Send + 'static) {
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            let Ok(mut c) = conn else { continue };
            c.set_read_timeout(Some(Duration::from_secs(2))).ok();
            let mut raw = Vec::new();
            let _ = (&mut c).take(1 << 16).read_to_end(&mut raw);
            let (verb, cwd, args) = decode(&raw);
            match verb.as_str() {
                "claim" => {
                    let _ = CLAIM.set((args, cwd));
                    if let Some(sock) = socket() {
                        let _ = fs::remove_file(sock);
                    }
                    on_claim();
                    let _ = c.write_all(b"ok\n");
                    if enabled() {
                        start();
                    }
                    return;
                }
                "quit" => {
                    if let Some(sock) = socket() {
                        let _ = fs::remove_file(sock);
                    }
                    let _ = c.write_all(b"ok\n");
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_round_trip() {
        let (verb, cwd, args) = decode(&encode("claim", "/home/a", &["--info".into(), "/tmp/x y".into()]));
        assert_eq!((verb.as_str(), cwd.as_str(), args), ("claim", "/home/a", vec!["--info".to_string(), "/tmp/x y".to_string()]));
        let (verb, _, args) = decode(&encode("quit", "", &[]));
        assert_eq!(verb, "quit");
        assert!(args.is_empty());
    }

    #[test]
    fn what_a_spare_can_take() {
        let a = |l: &[&str]| l.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(claimable(&a(&[])));
        assert!(claimable(&a(&["/tmp"])));
        assert!(claimable(&a(&["--info", "/tmp/f"])));
        assert!(!claimable(&a(&["--pick", "open"])));
        assert!(!claimable(&a(&["/a", "/b"])));
        assert!(!claimable(&a(&["--portal"])));
    }
}
