// Volumes amovibles: lsblk pour les lister, udisksctl (udisks2, lancé à la demande par D-Bus) pour monter et
// démonter. Sans Qt: testé par cargo test (analyse de la sortie de lsblk).
use crate::i18n::trf;
use std::process::Command;

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
        if get("HOTPLUG") != "1"
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
    match Command::new("lsblk").args(["-P", "-o", "PATH,LABEL,SIZE,HOTPLUG,MOUNTPOINT,TYPE,FSTYPE"]).output() {
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
    let out = udisks("mount", dev)?;
    out.split_once(" at ")
        .map(|(_, p)| p.trim().trim_end_matches('.').to_string())
        .ok_or_else(|| out.trim().to_string())
}

pub fn unmount(dev: &str) -> Result<(), String> {
    udisks("unmount", dev).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
