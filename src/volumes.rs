// Removable volumes (USB stick, SD card, external disk): lsblk to list them, udisksctl (udisks2, started on demand
// by D-Bus) to mount and unmount. Network drives: those of gvfs (`gio mount`, folder $XDG_RUNTIME_DIR/gvfs; SMB,
// SFTP, WebDAV, FTP, NFS...) and the network-type system mounts (NFS, CIFS, sshfs) read from /proc/self/mounts.
// Without Qt: tested by cargo test (lsblk, names of network mounts, authentication dialog of gio).
use crate::i18n::{tr, trf};
use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

// Prefixes of `dev` for the rows that are not disks: gvfs mount or network system mount
pub const GVFS: &str = "gvfs:";
pub const NET: &str = "net:";
// MTP device (phone, camera) plugged in but not mounted yet: "mtp:/dev/bus/usb/BUS/DEV" (the address to mount,
// mtp://VENDOR_MODEL_SERIAL/, is asked from gio at the time of the click)
pub const MTP: &str = "mtp:";
// Server already used but not connected at the moment: "srv:smb://nas/docs"; a click reconnects it
pub const SRV: &str = "srv:";

#[derive(Debug, PartialEq)]
pub struct Volume {
    pub dev: String,
    pub name: String,
    pub mount: Option<String>,
}

// "KEY="value" KEY2="..."" (lsblk -P); special characters of a value are written \xNN
fn pairs(line: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = line.trim();
    while let Some((key, after)) = rest.split_once("=\"") {
        let mut value = String::new();
        let mut chars = after.char_indices();
        let mut end = after.len();
        while let Some((i, c)) = chars.next() {
            match c {
                '"' => {
                    end = i + 1;
                    break;
                }
                '\\' if after[i + 1..].starts_with('x') && after.len() >= i + 4 => {
                    if let Ok(b) = u8::from_str_radix(&after[i + 2..i + 4], 16) {
                        value.push(b as char);
                        chars.nth(2);
                    }
                }
                c => value.push(c),
            }
        }
        out.push((key.trim().to_string(), value));
        rest = after[end..].trim_start();
    }
    out
}

pub fn parse(text: &str) -> Vec<Volume> {
    let mut out = Vec::new();
    for line in text.lines() {
        let f: std::collections::HashMap<String, String> = pairs(line).into_iter().collect();
        let get = |k: &str| f.get(k).map(String::as_str).unwrap_or("");
        let fstype = get("FSTYPE");
        // Removable: USB (HOTPLUG), card reader (RM), SD card of the built-in reader (mmcblk), never the system
        let mount_here = get("MOUNTPOINT");
        let removable = get("HOTPLUG") == "1" || get("RM") == "1" || get("PATH").starts_with("/dev/mmcblk");
        let system = mount_here == "/" || mount_here.starts_with("/boot") || mount_here.starts_with("/efi") || mount_here == "[SWAP]";
        if !removable
            || system
            || fstype.is_empty()
            || ["swap", "crypto_LUKS", "LVM2_member", "linux_raid_member"].contains(&fstype)
            || !["part", "disk", "rom"].contains(&get("TYPE"))
        {
            continue;
        }
        let name = if get("LABEL").is_empty() { trf("%1 volume", &[&get("SIZE")]) } else { get("LABEL").to_string() };
        let mount = Some(get("MOUNTPOINT")).filter(|m| !m.is_empty()).map(String::from);
        out.push(Volume { dev: get("PATH").to_string(), name, mount });
    }
    out
}

pub fn list() -> Vec<Volume> {
    match Command::new("lsblk").args(["-P", "-o", "PATH,LABEL,SIZE,HOTPLUG,RM,MOUNTPOINT,TYPE,FSTYPE"]).output() {
        Ok(o) => parse(&String::from_utf8_lossy(&o.stdout)),
        Err(_) => Vec::new(),
    }
}

fn udisks(action: &str, dev: &str) -> Result<String, String> {
    let o = Command::new("udisksctl")
        .args([action, "-b", dev, "--no-user-interaction"])
        .output()
        .map_err(|e| e.to_string())?;
    if o.status.success() {
        Ok(String::from_utf8_lossy(&o.stdout).into_owned())
    } else {
        // "Error mounting /dev/sdb1: GDBus.Error:...: message"
        let err = String::from_utf8_lossy(&o.stderr);
        Err(err.trim().rsplit(": ").next().unwrap_or("").to_string())
    }
}

// Mounts dev and returns its mount point
pub fn mount(dev: &str) -> Result<String, String> {
    if let Some(uri) = dev.strip_prefix(MTP) {
        return mount_mtp(uri);
    }
    if let Some(uri) = dev.strip_prefix(SRV) {
        // Without credentials: gvfs reuses the password kept in the keyring, if there is one
        return connect(uri, "", "");
    }
    let out = udisks("mount", dev)?;
    out.split_once(" at ")
        .map(|(_, p)| p.trim().trim_end_matches('.').to_string())
        .ok_or_else(|| out.trim().to_string())
}

pub fn unmount(dev: &str) -> Result<(), String> {
    if let Some(uri) = dev.strip_prefix(SRV) {
        forget_server(uri);
        return Ok(());
    }
    if let Some(path) = dev.strip_prefix(GVFS) {
        let done = run_plain("gio", &["mount", "-u", path]);
        if done.is_ok() {
            mtp_ejected(path);
        }
        return done;
    }
    if let Some(path) = dev.strip_prefix(NET) {
        return run_plain("umount", &[path]);
    }
    udisks("unmount", dev).map(|_| ())
}

// MTP devices plugged in (marked ID_MTP_DEVICE=1 by the udev rule of libmtp) and not mounted yet. Read from the udev database
// (/run/udev/data/c189:N, N = 128 × (bus − 1) + (device − 1)): no process and no gvfs started until we click.
pub fn mtp_from_udev(file: &str, text: &str) -> Option<Volume> {
    let minor: u32 = file.strip_prefix("c189:")?.parse().ok()?;
    let get = |k: &str| text.lines().find_map(|l| l.strip_prefix(&format!("E:{k}="))).map(str::to_string);
    if get("ID_MTP_DEVICE").as_deref() != Some("1") {
        return None;
    }
    let unescape = |s: &str| s.replace("\\x20", " ").replace('_', " ");
    let name = get("ID_MODEL_ENC").map(|m| unescape(&m)).filter(|m| !m.trim().is_empty()).unwrap_or_else(|| tr("Portable device").to_string());
    Some(Volume { dev: format!("{MTP}/dev/bus/usb/{:03}/{:03}", minor / 128 + 1, minor % 128 + 1), name, mount: None })
}

// MTP devices plugged in according to the udev database, with their serial number
fn mtp_detected() -> Vec<(Volume, String)> {
    let Ok(dir) = std::fs::read_dir("/run/udev/data") else { return Vec::new() };
    let mut out: Vec<(Volume, String)> = dir
        .flatten()
        .filter_map(|e| {
            let file = e.file_name().to_string_lossy().into_owned();
            let text = std::fs::read_to_string(e.path()).ok()?;
            let serial = text.lines().find_map(|l| l.strip_prefix("E:ID_SERIAL_SHORT=")).unwrap_or("").to_string();
            Some((mtp_from_udev(&file, &text)?, serial))
        })
        .collect();
    out.sort_by(|a, b| a.0.dev.cmp(&b.0.dev));
    out
}

// Devices ejected by the user: absent from the sidebar until they are unplugged and plugged in again.
// An entry disappears as soon as the device is no longer plugged in.
// Kept in $XDG_RUNTIME_DIR (erased at logout), so shared by all Casier windows.
fn ejected_file() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into())).join("fili-file-ejected")
}

fn read_ejected() -> Vec<String> {
    std::fs::read_to_string(ejected_file()).unwrap_or_default().lines().filter(|l| !l.is_empty()).map(String::from).collect()
}

fn write_ejected(list: &[String]) {
    let _ = std::fs::write(ejected_file(), list.join("\n"));
}

// Ejecting a gvfs mount "mtp:host=VENDOR_MODEL_SERIAL": hides the device that carries this serial number
// (failing to recognize it, all the ones that are plugged in)
fn mtp_ejected(mount_path: &str) {
    let Some(host) = mount_path.rsplit('/').next().filter(|n| n.starts_with("mtp:")) else { return };
    let all = mtp_detected();
    let mut hit: Vec<&Volume> = all.iter().filter(|(_, s)| !s.is_empty() && host.ends_with(s.as_str())).map(|(v, _)| v).collect();
    if hit.is_empty() {
        hit = all.iter().map(|(v, _)| v).collect();
    }
    let mut ejected = read_ejected();
    for v in hit {
        if !ejected.contains(&v.dev) {
            ejected.push(v.dev.clone());
        }
    }
    write_ejected(&ejected);
}

pub fn mtp_devices() -> Vec<Volume> {
    let all = mtp_detected();
    let mut ejected = read_ejected();
    let before = ejected.len();
    ejected.retain(|d| all.iter().any(|(v, _)| v.dev == *d));
    if ejected.len() != before {
        write_ejected(&ejected);
    }
    // Already mounted (gvfs "mtp:host=…" entry): the sidebar shows that one
    if std::fs::read_dir(gvfs_dir()).is_ok_and(|d| d.flatten().any(|e| e.file_name().to_string_lossy().starts_with("mtp:"))) {
        return Vec::new();
    }
    all.into_iter().map(|(v, _)| v).filter(|v| !ejected.contains(&v.dev)).collect()
}

// Address to mount for a device: "Volume(0): … unix-device: '/dev/bus/usb/004/011' … activation_root=mtp://X/" (gio mount -li)
pub fn mtp_root(listing: &str, device: &str) -> Option<String> {
    let mut found = false;
    for line in listing.lines() {
        let l = line.trim();
        if l.starts_with("Volume(") || l.starts_with("Mount(") || l.starts_with("Drive(") {
            found = false;
        }
        if l.strip_prefix("unix-device:").is_some_and(|d| d.trim().trim_matches('\'') == device) {
            found = true;
        }
        if found && let Some(root) = l.strip_prefix("activation_root=") {
            return Some(root.to_string());
        }
    }
    None
}

fn mount_mtp(device: &str) -> Result<String, String> {
    let listing = Command::new("gio").args(["mount", "-li"]).stdin(Stdio::null()).output().map_err(|e| e.to_string())?;
    let uri = mtp_root(&String::from_utf8_lossy(&listing.stdout), device)
        .ok_or_else(|| tr("Unlock the device and choose File Transfer (MTP) mode").to_string())?;
    let o = Command::new("gio").args(["mount", &uri]).stdin(Stdio::null()).output().map_err(|e| e.to_string())?;
    if !o.status.success() {
        let err = String::from_utf8_lossy(&o.stderr);
        return Err(if err.to_lowercase().contains("mtp") || err.contains("busy") || err.contains("Unable") {
            tr("Unlock the device and choose File Transfer (MTP) mode").to_string()
        } else {
            err.trim().rsplit(": ").next().unwrap_or("").to_string()
        });
    }
    std::fs::read_dir(gvfs_dir())
        .ok()
        .and_then(|d| d.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).find(|n| n.starts_with("mtp:")))
        .map(|n| format!("{}/{n}", gvfs_dir()))
        .ok_or_else(|| tr("Mounted, but the folder was not found").to_string())
}

fn run_plain(prog: &str, args: &[&str]) -> Result<(), String> {
    let o = Command::new(prog).args(args).output().map_err(|e| e.to_string())?;
    if o.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&o.stderr).trim().rsplit(": ").next().unwrap_or("").to_string())
    }
}

fn gvfs_dir() -> String {
    format!("{}/gvfs", std::env::var("XDG_RUNTIME_DIR").unwrap_or_default())
}

// Readable name of a gvfs folder: "smb-share:server=nas,share=docs" -> "docs on nas", "sftp:host=nas,user=ana" -> "nas (sftp)"
pub fn gvfs_name(entry: &str) -> String {
    let (scheme, rest) = entry.split_once(':').unwrap_or((entry, ""));
    let get = |k: &str| rest.split(',').find_map(|kv| kv.strip_prefix(&format!("{k}=")).map(str::to_string));
    let host = get("server").or_else(|| get("host")).unwrap_or_default();
    // Phone: "SAMSUNG_Android_RF8N…" -> "SAMSUNG Android RF8N…"
    if scheme == "mtp" && !host.is_empty() {
        return host.replace('_', " ");
    }
    match (get("share"), host.as_str()) {
        (Some(share), h) if !h.is_empty() => trf("%1 on %2", &[&share, &h]),
        (_, "") => entry.to_string(),
        (_, h) => format!("{h} ({})", scheme.trim_end_matches("-share")),
    }
}

// Gvfs mounts in progress (FUSE path) and network system mounts, for the sidebar
pub fn network() -> Vec<Volume> {
    let mut out = Vec::new();
    if let Ok(dir) = std::fs::read_dir(gvfs_dir()) {
        let mut entries: Vec<String> = dir.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        entries.sort();
        for e in entries {
            let path = format!("{}/{e}", gvfs_dir());
            out.push(Volume { dev: format!("{GVFS}{path}"), name: gvfs_name(&e), mount: Some(path) });
        }
    }
    if let Ok(text) = std::fs::read_to_string("/proc/self/mounts") {
        out.extend(system_network(&text));
    }
    out
}

// NFS, CIFS and sshfs mounted by the system (fstab, mount): "source point type options ..."
pub fn system_network(mounts: &str) -> Vec<Volume> {
    let mut out = Vec::new();
    for line in mounts.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 3 || !["nfs", "nfs4", "cifs", "smb3", "fuse.sshfs"].contains(&f[2]) {
            continue;
        }
        let unescape = |s: &str| s.replace("\\040", " ");
        let (source, point) = (unescape(f[0]), unescape(f[1]));
        let name = Path::new(&point).file_name().map_or(source.clone(), |n| n.to_string_lossy().into_owned());
        out.push(Volume { dev: format!("{NET}{point}"), name: format!("{name} ({source})"), mount: Some(point) });
    }
    out
}

// Answer to a question of `gio mount` ("User [ana]:", "Domain [WORKGROUP]:", "Password:", "Anonymous? (y/n)");
// nothing if the text is not a pending question
pub fn answer(prompt: &str, user: &str, password: &str) -> Option<String> {
    let line = prompt.rsplit('\n').next().unwrap_or("");
    if !line.ends_with(": ") && !line.ends_with("(y/n) ") && !line.ends_with("(y/n)? ") {
        return None;
    }
    let l = line.to_lowercase();
    Some(if l.contains("anonymous") {
        "n".into()
    } else if l.starts_with("user") {
        user.into()
    } else if l.starts_with("password") {
        password.into()
    } else {
        String::new()
    })
}

// Runs `cmd` (gio mount ...) and answers its questions with the given user and password; error after 60 s.
// Returns the error output on failure.
pub fn run_mount(cmd: &mut Command, user: &str, password: &str) -> Result<(), String> {
    run_gio(cmd, user, password).map(|_| ())
}

// Connection journal: ~/.cache/fili-file/connect.log (the questions gio asked, which kind of answer was sent, how it
// ended; never a password). Kept small. It is what to read when a server will not connect.
fn journal(line: &str) {
    use std::io::Write as _;
    let cache = std::env::var("XDG_CACHE_HOME").ok().filter(|v| !v.is_empty()).or_else(|| std::env::var("HOME").ok().map(|h| format!("{h}/.cache")));
    let Some(cache) = cache else { return };
    let dir = std::path::PathBuf::from(cache).join("fili-file");
    let _ = std::fs::create_dir_all(&dir);
    let file = dir.join("connect.log");
    if std::fs::metadata(&file).map(|m| m.len() > 64 * 1024).unwrap_or(false) {
        let _ = std::fs::remove_file(&file);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(file) {
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let _ = writeln!(f, "[{secs}] {line}");
    }
}

// Like run_mount, and returns what the command printed (questions included). A password asked a second time means the
// first one was refused: that fails at once instead of answering the same thing until the time is out.
fn run_gio(cmd: &mut Command, user: &str, password: &str) -> Result<String, String> {
    journal(&format!("run {:?} (user {:?}, password {})", cmd, user, if password.is_empty() { "empty" } else { "given" }));
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut stdin = child.stdin.take().ok_or("stdin")?;
    let mut stdout = child.stdout.take().ok_or("stdout")?;
    let mut stderr = child.stderr.take().ok_or("stderr")?;
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut buf = [0u8; 512];
        while let Ok(n) = stdout.read(&mut buf) {
            if n == 0 || tx.send(buf[..n].to_vec()).is_err() {
                return;
            }
        }
    });
    let err = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });
    let (mut seen, start) = (String::new(), Instant::now());
    let (mut output, mut password_answers) = (String::new(), 0);
    let status = loop {
        if let Some(st) = child.try_wait().map_err(|e| e.to_string())? {
            break st;
        }
        if start.elapsed() > Duration::from_secs(60) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(tr("Timed out").to_string());
        }
        if let Ok(chunk) = rx.recv_timeout(Duration::from_millis(100)) {
            seen.push_str(&String::from_utf8_lossy(&chunk));
            output.push_str(&String::from_utf8_lossy(&chunk));
            journal(&format!("gio said {:?}", chunk_text(&chunk)));
            if let Some(reply) = answer(&seen, user, password) {
                journal(&format!("answering the question {:?} with {}", seen.rsplit('\n').next().unwrap_or(""), reply_kind(&seen, &reply)));
                if seen.rsplit('\n').next().unwrap_or("").to_lowercase().starts_with("password") {
                    password_answers += 1;
                    if password_answers > 1 {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(tr("Wrong user name or password").to_string());
                    }
                }
                let _ = writeln!(stdin, "{reply}");
                let _ = stdin.flush();
                seen.clear();
            }
        }
    };
    let msg = err.join().unwrap_or_default();
    journal(&format!("ended: {status}; stderr {:?}", msg.trim()));
    if status.success() {
        return Ok(output);
    }
    // "Error mounting location: Location is already mounted" counts as a success
    if msg.contains("already mounted") {
        return Ok(output);
    }
    Err(msg.trim().rsplit(": ").next().unwrap_or("").to_string())
}

fn chunk_text(chunk: &[u8]) -> String {
    String::from_utf8_lossy(chunk).into_owned()
}

// What kind of thing an answer is, for the journal (a password is never written)
fn reply_kind(question: &str, reply: &str) -> String {
    let q = question.rsplit('\n').next().unwrap_or("").to_lowercase();
    if q.starts_with("password") {
        "a password".into()
    } else if reply.is_empty() {
        "an empty line".into()
    } else {
        format!("{reply:?}")
    }
}

// After a successful connection: the server is remembered (one line in the sidebar) and, if asked, so is the password
fn finish(full: &str, host: &str, user: &str, password: &str, remember_password: bool) {
    remember_server(&server_uri(full));
    if remember_password && !user.is_empty() && !password.is_empty() {
        if let Err(e) = crate::creds::store(host, user, password) {
            journal(&format!("the password could not be kept in the keyring: {e}"));
        }
    }
}

// Names of the shares printed by `gio list smb://host/` (the questions of the authentication are not names)
fn shares_from(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.contains(':') && !l.starts_with("Authentication") && !l.ends_with('$'))
        .map(String::from)
        .collect()
}

// Connects the shares of an SMB server given without one: at most MAX_SHARES, the first connected one is returned
const MAX_SHARES: usize = 8;

// The shares of an SMB server, `root` being "smb://user@host/"
pub fn list_shares(root: &str, user: &str, password: &str) -> Result<Vec<String>, String> {
    let mut cmd = if Command::new("stdbuf").arg("--version").output().is_ok() {
        let mut c = Command::new("stdbuf");
        c.args(["-o0", "gio", "list", root]);
        c
    } else {
        let mut c = Command::new("gio");
        c.args(["list", root]);
        c
    };
    Ok(shares_from(&run_gio(&mut cmd, user, password)?))
}

// The shares of a server with the password kept in the keyring (anonymous listing otherwise)
pub fn list_shares_of(uri: &str) -> Result<Vec<String>, String> {
    let (user, host, _) = address_parts(uri);
    let password = crate::creds::lookup(&host, &user).unwrap_or_default();
    list_shares(&server_uri(uri), &user, &password)
}

// "smb://user@host/share" for a path inside a gvfs mount that is not mounted any more (a favorite or a recent folder of a
// share that was ejected, or from before the last login); None for any other path
pub fn unmounted_share_uri(path: &str) -> Option<String> {
    let rest = path.strip_prefix(&gvfs_dir())?.trim_start_matches('/');
    let name = rest.split('/').next()?;
    let fields = name.strip_prefix("smb-share:")?;
    if Path::new(&format!("{}/{name}", gvfs_dir())).exists() {
        return None;
    }
    let field = |k: &str| fields.split(',').find_map(|p| p.strip_prefix(&format!("{k}="))).unwrap_or("").to_string();
    let (host, share, user) = (field("server"), field("share"), field("user"));
    (!host.is_empty() && !share.is_empty()).then(|| if user.is_empty() { format!("smb://{host}/{share}") } else { format!("smb://{user}@{host}/{share}") })
}

fn connect_shares(root: &str, user: &str, password: &str) -> Result<String, String> {
    let shares = list_shares(root, user, password)?;
    if shares.is_empty() {
        return Err(tr("No shares found on this server").to_string());
    }
    let base = root.trim_end_matches('/');
    let (mut first, mut last_error) = (None, String::new());
    for share in shares.iter().take(MAX_SHARES) {
        match connect(&format!("{base}/{share}"), user, password) {
            Ok(folder) => {
                first.get_or_insert(folder);
            }
            Err(e) => last_error = e,
        }
    }
    first.ok_or(last_error)
}

// ---- remembered servers ---------------------------------------------------------------------------------
// ~/.local/state/filigrane/fili-file-servers: one address per line, the most recent first, 10 at most. Never a password
// (gvfs keeps its own in the session keyring).
fn servers_file() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    let state = std::env::var("XDG_STATE_HOME").unwrap_or_else(|_| format!("{home}/.local/state"));
    std::path::PathBuf::from(state).join("filigrane/fili-file-servers")
}

fn read_servers() -> Vec<String> {
    std::fs::read_to_string(servers_file()).unwrap_or_default().lines().map(str::trim).filter(|l| l.contains("://")).map(String::from).collect()
}

fn write_servers(list: &[String]) {
    let f = servers_file();
    if let Some(dir) = f.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(f, list.join("\n") + "\n");
}

// "smb://ana:secret@nas/docs" -> "smb://ana@nas/docs"
pub fn without_password(uri: &str) -> String {
    let Some((scheme, rest)) = uri.split_once("://") else { return uri.to_string() };
    let (auth, tail) = rest.split_once('/').map_or((rest, None), |(a, t)| (a, Some(t)));
    let auth = match auth.rsplit_once('@') {
        Some((user, host)) => format!("{}@{host}", user.split(':').next().unwrap_or("")),
        None => auth.to_string(),
    };
    match tail {
        Some(t) => format!("{scheme}://{auth}/{t}"),
        None => format!("{scheme}://{auth}"),
    }
}

pub fn remember_server(uri: &str) {
    let uri = without_password(uri.trim());
    let mut list = read_servers();
    list.retain(|u| *u != uri);
    list.insert(0, uri);
    list.truncate(10);
    write_servers(&list);
}

pub fn forget_server(uri: &str) {
    let (user, host, _) = address_parts(uri);
    crate::creds::forget(&host, &user);
    let mut list = read_servers();
    list.retain(|u| u != uri);
    write_servers(&list);
}

// Remembered servers that are not connected (a gvfs mount talks about the same host, and the same share for SMB)
// Name shown for a server: the name it announces on the network (discover.rs) when the address matches, else the host
fn server_name(host: &str) -> String {
    crate::discover::cached()
        .into_iter()
        .find(|s| s.uri.contains(&format!("//{host}/")))
        .map(|s| s.name.trim_end_matches("(SMB)").trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| host.to_string())
}

// One row per server in the sidebar: the servers already used and the ones with a folder mounted right now (a share is
// reached through its server). A click opens the server (its shares), the cross forgets it.
pub fn saved_servers() -> Vec<Volume> {
    let mut uris: Vec<String> = read_servers().iter().map(|u| server_uri(u)).collect();
    for v in network() {
        // smb-share:server=nas,share=docs,user=ana
        let Some(name) = v.mount.as_deref().and_then(|m| m.rsplit('/').next().filter(|n| n.starts_with("smb-share:"))) else { continue };
        let field = |k: &str| name.split(',').find_map(|p| p.trim_start_matches("smb-share:").strip_prefix(&format!("{k}="))).unwrap_or("").to_string();
        let (host, user) = (field("server"), field("user"));
        if !host.is_empty() {
            uris.push(if user.is_empty() { format!("smb://{host}/") } else { format!("smb://{user}@{host}/") });
        }
    }
    let mut seen: Vec<String> = Vec::new();
    uris.into_iter()
        .filter(|u| {
            let (user, host, _) = address_parts(u);
            let key = format!("{}@{}", user.to_lowercase(), host.to_lowercase());
            !seen.contains(&key) && {
                seen.push(key);
                true
            }
        })
        .map(|u| {
            let (_, host, _) = address_parts(&u);
            Volume { dev: format!("{SRV}{u}"), name: server_name(&host), mount: None }
        })
        .collect()
}

// Connects to a server (smb://host/share, sftp://host, davs://..., ftp://..., nfs://...) with gvfs and returns the
// FUSE folder of the mount. A typed user name is put in the address; without a full address: error.
pub fn connect(uri: &str, user: &str, password: &str) -> Result<String, String> {
    connect_with(uri, user, password, false)
}

// (user, host, path) of an address such as smb://ana@nas/docs/x; the user may be empty
pub fn address_parts(uri: &str) -> (String, String, String) {
    let rest = uri.split_once("://").map_or(uri, |(_, r)| r);
    let (auth, path) = rest.split_once('/').unwrap_or((rest, ""));
    match auth.rsplit_once('@') {
        Some((user, host)) => (user.split(':').next().unwrap_or("").to_string(), host.to_string(), path.to_string()),
        None => (String::new(), auth.to_string(), path.to_string()),
    }
}

// The server alone: "smb://ana@nas/docs/x" -> "smb://ana@nas/"
pub fn server_uri(uri: &str) -> String {
    let scheme = uri.split_once("://").map_or("smb", |(s, _)| s);
    let (user, host, _) = address_parts(uri);
    if user.is_empty() { format!("{scheme}://{host}/") } else { format!("{scheme}://{user}@{host}/") }
}

// Like connect. Without a password, the one kept in the keyring for this user on this host is used; `remember` keeps the
// password that worked there for next time.
pub fn connect_with(uri: &str, user: &str, password: &str, remember: bool) -> Result<String, String> {
    let uri = uri.trim();
    let Some((scheme, rest)) = uri.split_once("://").filter(|(s, r)| !s.is_empty() && !r.is_empty()) else {
        return Err(tr("Enter an address such as smb://server/share or sftp://server").to_string());
    };
    let (uri_user, host, _) = address_parts(uri);
    let user = if user.is_empty() { uri_user } else { user.to_string() };
    let user = user.as_str();
    let kept = if password.is_empty() { crate::creds::lookup(&host, user) } else { None };
    let password = kept.as_deref().unwrap_or(password);
    let full = if !user.is_empty() && !rest.contains('@') { format!("{scheme}://{user}@{rest}") } else { uri.to_string() };
    // An SMB server without a share has no folder to open: connect its shares instead (the first one opens)
    if scheme == "smb" && rest.split_once('/').map_or(true, |(_, path)| path.trim_matches('/').is_empty()) {
        let folder = connect_shares(&full, user, password)?;
        finish(&full, &host, user, password, remember && kept.is_none());
        return Ok(folder);
    }
    let before: Vec<String> = network().into_iter().filter_map(|v| v.mount).collect();
    let mut cmd = if Command::new("stdbuf").arg("--version").output().is_ok() {
        let mut c = Command::new("stdbuf");
        c.args(["-o0", "gio", "mount", &full]);
        c
    } else {
        let mut c = Command::new("gio");
        c.args(["mount", &full]);
        c
    };
    run_mount(&mut cmd, user, password)?;
    finish(&full, &host, user, password, remember && kept.is_none());
    // The new gvfs folder; otherwise the one that talks about the same host (already mounted)
    let now = network();
    let host = rest.split('/').next().unwrap_or("").rsplit('@').next().unwrap_or("").to_string();
    now.iter()
        .filter_map(|v| v.mount.clone())
        .find(|m| !before.contains(m))
        .or_else(|| now.iter().filter_map(|v| v.mount.clone()).find(|m| m.contains(&host)))
        .ok_or_else(|| tr("Mounted, but the folder was not found").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mtp_phone_from_udev() {
        unsafe { std::env::set_var("FILI_LANG", "en") };
        let phone = "I:1\nE:ID_MTP_DEVICE=1\nE:ID_MODEL_ENC=Pixel\\x206\nE:ID_BUS=usb\n";
        // c189:133 = bus 2, appareil 6
        let v = mtp_from_udev("c189:133", phone).unwrap();
        assert_eq!(v, Volume { dev: "mtp:/dev/bus/usb/002/006".into(), name: "Pixel 6".into(), mount: None });
        let listing = "Volume(0): Pixel 6\n  ids:\n   unix-device: '/dev/bus/usb/002/006'\n  activation_root=mtp://G_Pixel_6_ABC/\nVolume(1): autre\n   unix-device: '/dev/bus/usb/001/002'\n  activation_root=mtp://X/\n";
        assert_eq!(mtp_root(listing, "/dev/bus/usb/002/006").as_deref(), Some("mtp://G_Pixel_6_ABC/"));
        assert_eq!(mtp_root(listing, "/dev/bus/usb/001/002").as_deref(), Some("mtp://X/"));
        assert_eq!(mtp_root(listing, "/dev/bus/usb/009/009"), None);
        assert_eq!(mtp_from_udev("c189:0", "E:ID_BUS=usb\n"), None);
        assert_eq!(mtp_from_udev("b8:0", phone), None);
        assert_eq!(gvfs_name("mtp:host=SAMSUNG_Android_RF8N"), "SAMSUNG Android RF8N");
    }

    #[test]
    fn server_addresses() {
        assert_eq!(without_password("smb://ana:secret@nas/docs"), "smb://ana@nas/docs");
        assert_eq!(without_password("sftp://box"), "sftp://box");
        assert_eq!(without_password("ftp://u:p@h"), "ftp://u@h");
    }

    #[test]
    fn parses_lsblk() {
        unsafe { std::env::set_var("FILI_LANG", "en") };
        let text = concat!(
            "PATH=\"/dev/nvme0n1p2\" LABEL=\"\" SIZE=\"232.8G\" HOTPLUG=\"0\" MOUNTPOINT=\"/\" TYPE=\"part\" FSTYPE=\"ext4\"\n",
            "PATH=\"/dev/sda\" LABEL=\"\" SIZE=\"14.5G\" HOTPLUG=\"1\" MOUNTPOINT=\"\" TYPE=\"disk\" FSTYPE=\"\"\n",
            "PATH=\"/dev/sda1\" LABEL=\"CL\\x20USB\" SIZE=\"14.5G\" HOTPLUG=\"1\" MOUNTPOINT=\"/run/media/t/CL USB\" TYPE=\"part\" FSTYPE=\"vfat\"\n",
            "PATH=\"/dev/sdb1\" LABEL=\"\" SIZE=\"1.8T\" HOTPLUG=\"1\" MOUNTPOINT=\"\" TYPE=\"part\" FSTYPE=\"exfat\"\n",
        );
        let v = parse(text);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], Volume { dev: "/dev/sda1".into(), name: "CL USB".into(), mount: Some("/run/media/t/CL USB".into()) });
        assert_eq!(v[1], Volume { dev: "/dev/sdb1".into(), name: "1.8T volume".into(), mount: None });
    }

    #[test]
    fn sd_card_and_system_disks() {
        unsafe { std::env::set_var("FILI_LANG", "en") };
        let text = concat!(
            "PATH=\"/dev/mmcblk0p1\" LABEL=\"CAMERA\" SIZE=\"29.7G\" HOTPLUG=\"0\" RM=\"0\" MOUNTPOINT=\"\" TYPE=\"part\" FSTYPE=\"exfat\"\n",
            "PATH=\"/dev/mmcblk1p2\" LABEL=\"\" SIZE=\"60G\" HOTPLUG=\"0\" RM=\"0\" MOUNTPOINT=\"/\" TYPE=\"part\" FSTYPE=\"ext4\"\n",
            "PATH=\"/dev/sdc1\" LABEL=\"READER\" SIZE=\"8G\" HOTPLUG=\"0\" RM=\"1\" MOUNTPOINT=\"\" TYPE=\"part\" FSTYPE=\"vfat\"\n",
            "PATH=\"/dev/sr0\" LABEL=\"DISC\" SIZE=\"700M\" HOTPLUG=\"1\" RM=\"1\" MOUNTPOINT=\"\" TYPE=\"rom\" FSTYPE=\"iso9660\"\n",
        );
        let names: Vec<String> = parse(text).into_iter().map(|v| v.name).collect();
        assert_eq!(names, ["CAMERA", "READER", "DISC"]);
    }

    #[test]
    fn network_names() {
        unsafe { std::env::set_var("FILI_LANG", "en") };
        assert_eq!(gvfs_name("smb-share:server=nas,share=docs"), "docs on nas");
        assert_eq!(gvfs_name("sftp:host=box,user=ana"), "box (sftp)");
        assert_eq!(gvfs_name("dav:host=cloud.example,ssl=true"), "cloud.example (dav)");
        assert_eq!(gvfs_name("inconnu"), "inconnu");
        let mounts = "/dev/nvme0n1p2 / ext4 rw 0 0\nnas:/export /mnt/nas\\040data nfs4 rw 0 0\n//srv/share /mnt/s cifs rw 0 0\n";
        let v = system_network(mounts);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], Volume { dev: "net:/mnt/nas data".into(), name: "nas data (nas:/export)".into(), mount: Some("/mnt/nas data".into()) });
        assert_eq!(v[1].dev, "net:/mnt/s");
    }

    #[test]
    fn answers_gio_questions() {
        assert_eq!(answer("Authentication Required\nUser [ana]: ", "bob", "pw"), Some("bob".into()));
        assert_eq!(answer("Domain [WORKGROUP]: ", "bob", "pw"), Some(String::new()));
        assert_eq!(answer("x\nPassword: ", "bob", "pw"), Some("pw".into()));
        assert_eq!(answer("Anonymous? (y/n) ", "bob", "pw"), Some("n".into()));
        assert_eq!(answer("Authentication Required\n", "bob", "pw"), None);
    }

    #[test]
    fn shares_are_read_from_a_listing() {
        assert_eq!(shares_from("Authentication Required\nEnter password for “nas”:\nDomain [WORKGROUP]: Password: \npublic\nSauvegarde\nIPC$\n"), vec!["public", "Sauvegarde"]);
        assert!(shares_from("").is_empty());
    }

    #[test]
    fn a_refused_password_fails_at_once() {
        // Asks for the password twice, like gio does when the first one is wrong
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "printf 'Password: '; read p; printf '\nPassword: '; read p; sleep 30"]);
        let t = Instant::now();
        assert_eq!(run_mount(&mut cmd, "bob", "bad"), Err("Wrong user name or password".into()));
        assert!(t.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn mount_dialogue() {
        // Fake `gio mount`: asks the questions in order, refuses a wrong password
        let script = r#"printf 'User [x]: '; read u; printf 'Domain [W]: '; read d; printf 'Password: '; read p
            [ "$u:$p" = "bob:pw" ] || { echo 'gio: Error mounting location: Access denied' >&2; exit 2; }"#;
        let mut ok = Command::new("sh");
        ok.args(["-c", script]);
        assert_eq!(run_mount(&mut ok, "bob", "pw"), Ok(()));
        let mut bad = Command::new("sh");
        bad.args(["-c", script]);
        assert_eq!(run_mount(&mut bad, "bob", "nope"), Err("Access denied".into()));
        let mut again = Command::new("sh");
        again.args(["-c", "echo 'Error mounting location: Location is already mounted' >&2; exit 1"]);
        assert_eq!(run_mount(&mut again, "", ""), Ok(()));
    }
}
