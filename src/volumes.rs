// Volumes amovibles (clé USB, carte SD, disque externe): lsblk pour les lister, udisksctl (udisks2, lancé à la demande
// par D-Bus) pour monter et démonter. Lecteurs réseau: ceux de gvfs (`gio mount`, dossier $XDG_RUNTIME_DIR/gvfs; SMB,
// SFTP, WebDAV, FTP, NFS...) et les montages système de type réseau (NFS, CIFS, sshfs) lus dans /proc/self/mounts.
// Sans Qt: testé par cargo test (lsblk, noms des montages réseau, dialogue d'authentification de gio).
use crate::i18n::{tr, trf};
use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

// Préfixes de `dev` pour les lignes qui ne sont pas des disques: montage gvfs ou montage système réseau
pub const GVFS: &str = "gvfs:";
pub const NET: &str = "net:";
// Appareil MTP (téléphone, appareil photo) branché mais pas encore monté: « mtp:/dev/bus/usb/BUS/DEV » (l'adresse à monter,
// mtp://FABRICANT_MODELE_SERIE/, est demandée à gio au moment du clic)
pub const MTP: &str = "mtp:";
// Serveur déjà utilisé mais pas connecté en ce moment: « srv:smb://nas/docs »; un clic le reconnecte
pub const SRV: &str = "srv:";

#[derive(Debug, PartialEq)]
pub struct Volume {
    pub dev: String,
    pub name: String,
    pub mount: Option<String>,
}

// « CLE="valeur" CLE2="..." » (lsblk -P); les caractères spéciaux d'une valeur sont écrits \xNN
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
        // Amovible: USB (HOTPLUG), lecteur de cartes (RM), carte SD du lecteur intégré (mmcblk), jamais le système
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
        // « Error mounting /dev/sdb1: GDBus.Error:...: message »
        let err = String::from_utf8_lossy(&o.stderr);
        Err(err.trim().rsplit(": ").next().unwrap_or("").to_string())
    }
}

// Monte dev et rend son point de montage
pub fn mount(dev: &str) -> Result<String, String> {
    if let Some(uri) = dev.strip_prefix(MTP) {
        return mount_mtp(uri);
    }
    if let Some(uri) = dev.strip_prefix(SRV) {
        // Sans identifiant: gvfs reprend le mot de passe gardé dans le trousseau, s'il y en a un
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

// Appareils MTP branchés (marqués ID_MTP_DEVICE=1 par la règle udev de libmtp) et pas encore montés. Lus dans la base udev
// (/run/udev/data/c189:N, N = 128 × (bus − 1) + (appareil − 1)): ni processus ni gvfs lancé tant qu'on ne clique pas.
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

// Appareils MTP branchés d'après la base udev, avec leur numéro de série
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

// Appareils éjectés par l'utilisateur: absents de la barre latérale jusqu'à ce qu'on les débranche et les rebranche.
// Une entrée disparaît dès que l'appareil n'est plus branché.
// Gardés dans $XDG_RUNTIME_DIR (effacé à la déconnexion), donc communs à toutes les fenêtres de Casier.
fn ejected_file() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into())).join("fili-file-ejected")
}

fn read_ejected() -> Vec<String> {
    std::fs::read_to_string(ejected_file()).unwrap_or_default().lines().filter(|l| !l.is_empty()).map(String::from).collect()
}

fn write_ejected(list: &[String]) {
    let _ = std::fs::write(ejected_file(), list.join("\n"));
}

// L'éjection d'un montage gvfs « mtp:host=FABRICANT_MODELE_SERIE »: cache l'appareil qui porte ce numéro de série
// (à défaut de le reconnaître, tous ceux qui sont branchés)
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
    // Déjà monté (entrée « mtp:host=… » de gvfs): la barre latérale montre celui-là
    if std::fs::read_dir(gvfs_dir()).is_ok_and(|d| d.flatten().any(|e| e.file_name().to_string_lossy().starts_with("mtp:"))) {
        return Vec::new();
    }
    all.into_iter().map(|(v, _)| v).filter(|v| !ejected.contains(&v.dev)).collect()
}

// Adresse à monter d'un appareil: « Volume(0): … unix-device: '/dev/bus/usb/004/011' … activation_root=mtp://X/ » (gio mount -li)
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

// Nom lisible d'un dossier gvfs: « smb-share:server=nas,share=docs » -> « docs sur nas », « sftp:host=nas,user=ana » -> « nas (sftp) »
pub fn gvfs_name(entry: &str) -> String {
    let (scheme, rest) = entry.split_once(':').unwrap_or((entry, ""));
    let get = |k: &str| rest.split(',').find_map(|kv| kv.strip_prefix(&format!("{k}=")).map(str::to_string));
    let host = get("server").or_else(|| get("host")).unwrap_or_default();
    // Téléphone: « SAMSUNG_Android_RF8N… » -> « SAMSUNG Android RF8N… »
    if scheme == "mtp" && !host.is_empty() {
        return host.replace('_', " ");
    }
    match (get("share"), host.as_str()) {
        (Some(share), h) if !h.is_empty() => trf("%1 on %2", &[&share, &h]),
        (_, "") => entry.to_string(),
        (_, h) => format!("{h} ({})", scheme.trim_end_matches("-share")),
    }
}

// Montages gvfs en cours (chemin FUSE) et montages système réseau, pour la barre latérale
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

// NFS, CIFS et sshfs montés par le système (fstab, mount): « source point type options ... »
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

// Réponse à une question de `gio mount` (« User [ana]: », « Domain [WORKGROUP]: », « Password: », « Anonymous? (y/n) »);
// rien si le texte n'est pas une question en attente
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

// Lance `cmd` (gio mount ...) et répond à ses questions avec l'identifiant et le mot de passe donnés; erreur au bout de 60 s.
// Rend la sortie d'erreur en cas d'échec.
pub fn run_mount(cmd: &mut Command, user: &str, password: &str) -> Result<(), String> {
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
            if let Some(reply) = answer(&seen, user, password) {
                let _ = writeln!(stdin, "{reply}");
                let _ = stdin.flush();
                seen.clear();
            }
        }
    };
    if status.success() {
        return Ok(());
    }
    let msg = err.join().unwrap_or_default();
    // « Error mounting location: Location is already mounted » compte comme réussi
    if msg.contains("already mounted") {
        return Ok(());
    }
    Err(msg.trim().rsplit(": ").next().unwrap_or("").to_string())
}

// ---- serveurs mémorisés ---------------------------------------------------------------------------------
// ~/.local/state/filigrane/fili-file-servers: une adresse par ligne, la plus récente en premier, 10 au plus. Jamais de mot de passe
// (gvfs garde le sien dans le trousseau de session).
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

// « smb://ana:secret@nas/docs » -> « smb://ana@nas/docs »
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
    let mut list = read_servers();
    list.retain(|u| u != uri);
    write_servers(&list);
}

// Serveurs mémorisés qui ne sont pas connectés (un montage gvfs parle du même hôte, et du même partage pour SMB)
pub fn saved_servers() -> Vec<Volume> {
    let mounted: Vec<String> = std::fs::read_dir(gvfs_dir())
        .map(|d| d.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect())
        .unwrap_or_default();
    read_servers()
        .into_iter()
        .filter(|u| {
            let rest = u.split_once("://").map_or("", |(_, r)| r);
            let host = rest.split('/').next().unwrap_or("").rsplit('@').next().unwrap_or("");
            let share = rest.split('/').nth(1).unwrap_or("");
            !mounted.iter().any(|m| {
                m.contains(&format!("={host}")) && (share.is_empty() || !u.starts_with("smb") || m.contains(&format!("share={share}")))
            })
        })
        .map(|u| {
            let rest = u.split_once("://").map_or("", |(_, r)| r);
            let host = rest.split('/').next().unwrap_or("").rsplit('@').next().unwrap_or("");
            let share = rest.split('/').nth(1).filter(|s| !s.is_empty());
            let name = match share {
                Some(sh) => trf("%1 on %2", &[&sh, &host]),
                None => host.to_string(),
            };
            Volume { dev: format!("{SRV}{u}"), name, mount: None }
        })
        .collect()
}

// Se connecte à un serveur (smb://hôte/partage, sftp://hôte, davs://..., ftp://..., nfs://...) avec gvfs et rend le
// dossier FUSE du montage. Un identifiant saisi est placé dans l'adresse; sans adresse complète: erreur.
pub fn connect(uri: &str, user: &str, password: &str) -> Result<String, String> {
    let uri = uri.trim();
    let Some((scheme, rest)) = uri.split_once("://").filter(|(s, r)| !s.is_empty() && !r.is_empty()) else {
        return Err(tr("Enter an address such as smb://server/share or sftp://server").to_string());
    };
    let full = if !user.is_empty() && !rest.contains('@') { format!("{scheme}://{user}@{rest}") } else { uri.to_string() };
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
    remember_server(&full);
    // Le nouveau dossier gvfs; sinon celui qui parle du même hôte (déjà monté)
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
    fn mount_dialogue() {
        // Faux `gio mount`: pose les questions dans l'ordre, refuse un mauvais mot de passe
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
