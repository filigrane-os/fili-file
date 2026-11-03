// Passwords of network servers, kept in the user's keyring (the freedesktop Secret Service: gnome-keyring, KeePassXC...)
// through `secret-tool`, so that a favorite or a server in the sidebar connects without typing the password again. The
// password never touches a file of ours and never goes in an argument (it is sent on the standard input).
// Without secret-tool or without a running keyring nothing is remembered and Casier simply asks again.
use std::{
    io::Write,
    process::{Command, Stdio},
};

const PROGRAM: &str = "secret-tool";

// Whether passwords can be kept: secret-tool is installed
pub fn available() -> bool {
    crate::tools::available(PROGRAM)
}

fn attributes(host: &str, user: &str) -> Vec<String> {
    ["service", "casier", "host", host, "user", user].map(String::from).to_vec()
}

fn store_with(program: &str, host: &str, user: &str, password: &str) -> Result<(), String> {
    let mut child = Command::new(program)
        .arg("store")
        .arg(format!("--label=Casier: {user}@{host}"))
        .args(attributes(host, user))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(password.as_bytes());
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if out.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).trim().to_string()) }
}

fn lookup_with(program: &str, host: &str, user: &str) -> Option<String> {
    let out = Command::new(program).arg("lookup").args(attributes(host, user)).stdin(Stdio::null()).stderr(Stdio::null()).output().ok()?;
    let password = String::from_utf8_lossy(&out.stdout).trim_end_matches(['\n', '\r']).to_string();
    (out.status.success() && !password.is_empty()).then_some(password)
}

pub fn store(host: &str, user: &str, password: &str) -> Result<(), String> {
    store_with(PROGRAM, host, user, password)
}

// The remembered password of `user` on `host`, if there is one
pub fn lookup(host: &str, user: &str) -> Option<String> {
    if host.is_empty() || user.is_empty() {
        return None;
    }
    lookup_with(PROGRAM, host, user)
}

pub fn forget(host: &str, user: &str) {
    let _ = Command::new(PROGRAM).arg("clear").args(attributes(host, user)).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt};

    // A fake secret-tool that keeps what it is given in a file next to itself
    fn fake(tag: &str) -> (std::path::PathBuf, String) {
        let dir = std::env::temp_dir().join(format!("fili-creds-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let script = format!(
            "#!/bin/sh\ncase \"$1\" in\n store) cat > '{d}/secret'; echo \"$@\" > '{d}/args';;\n lookup) [ -s '{d}/secret' ] && cat '{d}/secret' || exit 1;;\nesac\n",
            d = dir.display()
        );
        let program = dir.join("secret-tool");
        fs::write(&program, script).unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
        (dir.clone(), program.display().to_string())
    }

    #[test]
    fn a_password_goes_through_stdin_never_through_the_arguments() {
        let (dir, program) = fake("store");
        store_with(&program, "nas", "ana", "s3cret pass").unwrap();
        assert_eq!(fs::read_to_string(dir.join("secret")).unwrap(), "s3cret pass");
        let args = fs::read_to_string(dir.join("args")).unwrap();
        assert!(args.contains("host nas") && args.contains("user ana") && !args.contains("s3cret"), "{args}");
        assert_eq!(lookup_with(&program, "nas", "ana").as_deref(), Some("s3cret pass"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn nothing_remembered_gives_none() {
        let (dir, program) = fake("none");
        assert_eq!(lookup_with(&program, "nas", "ana"), None);
        assert_eq!(lookup("", "ana"), None);
        assert_eq!(lookup("nas", ""), None);
        let _ = fs::remove_dir_all(dir);
    }
}
