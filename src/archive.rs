// Compress and extract with tar, zip and 7z (the tools every distribution has): Archiveur, the display name of the
// `fili-archive` command. It is the same program as Casier: `fili-archive` and `archiveur` are other names of the
// `fili-file` binary (like `casier`), and `fili-file archive ...` does the same. No window, no Qt start-up.
//
//   fili-archive pack [-f FORMAT] [-l fast|normal|best] [--no-hidden] [--password-stdin] [-q] [-o OUT] FILE...
//                                                  creates an archive next to the files (zip by default)
//   fili-archive unpack [--password-stdin] [-o FOLDER] ARCHIVE...   extracts next to the archive (or into FOLDER)
//   fili-archive open ARCHIVE...                   same as unpack (file association, fili-archive.desktop)
//   fili-archive list ARCHIVE                      contents of the archive
//   fili-archive about                             version, commands and what it is made of
//
// Write formats: zip (default, or $FILI_ARCHIVE_FORMAT), 7z, tar.gz, tar.xz, tar.zst; read: whatever tar and 7z open
// (zip, 7z, rar, iso...). Rules: never overwrite ("Archive 2.zip", "folder 2"); the archive or folder only appears once
// finished (written under a hidden name, then renamed: a failure or an interruption leaves nothing half done); an archive
// is extracted into a folder of its own (a single top-level item is put as it is, otherwise a folder named after the
// archive). A password goes through the standard input (--password-stdin), never as an argument. Output: one created path
// per line; errors on stderr. Outside a terminal (menu, file association) the result is announced by a notification
// (notify-send); with the Filigrane `fili-tx` each call is a transaction (`fili-tx undo` trashes the result); -q skips both
// (Casier has its own panel, cancel and undo).
use crate::i18n::tr;
use std::{
    env,
    ffi::OsString,
    fs,
    io::{IsTerminal, Read},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

// Recognized archive extensions (compound ones first). tar: read by tar; anything else by 7z.
const EXTS: [&str; 19] = [
    "tar.gz", "tar.xz", "tar.bz2", "tar.zst", "tgz", "txz", "tbz2", "tzst", "tar", "zip", "7z", "rar", "iso", "gz", "xz", "bz2",
    "zst", "cab", "jar",
];

// (name without the archive extension, lower-case extension)
fn split_ext(name: &str) -> Option<(String, &'static str)> {
    let lower = name.to_lowercase();
    EXTS.iter()
        .find(|e| lower.ends_with(&format!(".{e}")) && name.len() > e.len() + 1)
        .map(|e| (name[..name.len() - e.len() - 1].to_string(), *e))
}

fn is_tar(ext: &str) -> bool {
    ext.starts_with("tar") || matches!(ext, "tgz" | "txz" | "tbz2" | "tzst")
}

fn file_name(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "Archive".into())
}

fn exists(p: &Path) -> bool {
    fs::symlink_metadata(p).is_ok()
}

// First free name in `dir`: "name.ext", "name 2.ext", "name 3.ext"...
fn unique(dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let make = |n: u32| {
        let s = if n == 1 { stem.to_string() } else { format!("{stem} {n}") };
        dir.join(if ext.is_empty() { s } else { format!("{s}.{ext}") })
    };
    (1..).map(make).find(|p| !exists(p)).unwrap()
}

// Runs a program. stdin: closed, or the password followed by a line break (7z reads it so; never as an argument, which
// `ps` would show). The error carries the end of the output; an encrypted archive gives "protected by a password" (no
// password given) or "wrong password".
fn run(program: &str, args: &[OsString], cwd: &Path, envs: &[(&str, String)], password: Option<&str>) -> Result<(), String> {
    let mut child = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .envs(envs.iter().map(|(k, v)| (*k, v.as_str())))
        .env("ZSTD_NBTHREADS", "0")
        .stdin(if password.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(
            |e| if e.kind() == std::io::ErrorKind::NotFound { format!("{program} is not installed") } else { e.to_string() },
        )?;
    if let (Some(pw), Some(mut stdin)) = (password, child.stdin.take()) {
        use std::io::Write;
        // 7z asks twice when creating (entry and confirmation) if it reads a terminal, once otherwise: we send one line
        let _ = writeln!(stdin, "{pw}");
    }
    // The standard output is read apart (7z asks its questions there) so that we never block on a full pipe
    let out_reader = child.stdout.take().map(|mut o| {
        std::thread::spawn(move || {
            let mut t = String::new();
            let _ = o.read_to_string(&mut t);
            t
        })
    });
    let mut err = String::new();
    if let Some(mut s) = child.stderr.take() {
        let _ = s.read_to_string(&mut err);
    }
    let out = out_reader.and_then(|h| h.join().ok()).unwrap_or_default();
    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() {
        return Ok(());
    }
    let all = format!("{out}\n{err}").to_lowercase();
    if all.contains("wrong password") || all.contains("cannot open encrypted") {
        return Err(if password.is_some() { "wrong password" } else { "the archive is protected by a password" }.into());
    }
    if password.is_none() && (all.contains("enter password") || all.contains("break signaled") || all.contains("encrypted")) {
        return Err("the archive is protected by a password".into());
    }
    let tail: Vec<&str> = err.lines().filter(|l| !l.trim().is_empty()).collect();
    let tail = tail[tail.len().saturating_sub(2)..].join(" ");
    Err(if tail.is_empty() { format!("{program} failed") } else { tail })
}

// First line of the standard input (option --password-stdin)
fn read_password() -> Result<String, String> {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).map_err(|e| e.to_string())?;
    let pw = line.trim_end_matches(['\n', '\r']).to_string();
    if pw.is_empty() { Err("empty password".into()) } else { Ok(pw) }
}

fn os(s: &str) -> OsString {
    OsString::from(s)
}

const USAGE_PACK: &str = "usage: fili-archive pack [-f zip|7z|tar.gz|tar.xz|tar.zst] [-l fast|normal|best] [--no-hidden] [--password-stdin] [-q] [-o OUT] FILE...";

// ---- pack ----------------------------------------------------------------------------------------------

fn pack(args: &[String]) -> Result<Vec<PathBuf>, String> {
    let (mut format, mut out, mut items) = (None::<String>, None::<PathBuf>, Vec::new());
    let (mut level, mut skip_hidden, mut ask_password) = ("normal".to_string(), false, false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-f" | "--format" => format = Some(it.next().ok_or("-f needs a format")?.to_lowercase()),
            "-o" | "--output" => out = Some(PathBuf::from(it.next().ok_or("-o needs a file name")?)),
            "-l" | "--level" => {
                level = it.next().ok_or("-l needs a level")?.to_lowercase();
                if !matches!(level.as_str(), "fast" | "normal" | "best") {
                    return Err(format!("unknown level “{level}” (fast, normal, best)"));
                }
            }
            "--no-hidden" => skip_hidden = true,
            "--password-stdin" => ask_password = true,
            "-q" | "--quiet" => {} // (quiet mode: nothing to do here)
            "--" => items.extend(it.by_ref().map(PathBuf::from)),
            s if s.starts_with('-') && s.len() > 1 => return Err(format!("unknown option {s}")),
            s => items.push(PathBuf::from(s)),
        }
    }
    if items.is_empty() {
        return Err(USAGE_PACK.into());
    }
    let items: Vec<PathBuf> =
        items.iter().map(|p| std::path::absolute(p).map_err(|e| e.to_string())).collect::<Result<_, _>>()?;
    for p in &items {
        if !exists(p) {
            return Err(format!("“{}” does not exist", file_name(p)));
        }
    }
    // Common folder: the names in the archive are relative to it
    let base = {
        let mut c: Vec<Component> = items[0].parent().unwrap_or(Path::new("/")).components().collect();
        for p in &items[1..] {
            let q: Vec<Component> = p.parent().unwrap_or(Path::new("/")).components().collect();
            let n = c.iter().zip(&q).take_while(|(a, b)| a == b).count();
            c.truncate(n);
        }
        c.iter().collect::<PathBuf>()
    };
    let rels: Vec<String> = items.iter().map(|p| p.strip_prefix(&base).unwrap().to_string_lossy().into_owned()).collect();
    if rels.iter().any(|r| r.is_empty()) {
        return Err("cannot compress the root folder".into());
    }
    // Output: -o (exact file, format deduced from its extension), otherwise next to the items, named after the item (or "Archive")
    let default = env::var("FILI_ARCHIVE_FORMAT").unwrap_or_else(|_| "zip".into());
    let fmt_of = |f: &str| -> Result<String, String> {
        Ok(match f {
            "zip" | "7z" | "tar.gz" | "tar.xz" | "tar.zst" => f.into(),
            "tgz" => "tar.gz".into(),
            "txz" => "tar.xz".into(),
            "tzst" | "zst" => "tar.zst".into(),
            other => return Err(format!("unknown format “{other}” (zip, 7z, tar.gz, tar.xz, tar.zst)")),
        })
    };
    let (dest, fmt) = match &out {
        Some(o) => {
            let o = std::path::absolute(o).map_err(|e| e.to_string())?;
            let (o, fmt) = match (format.as_deref(), split_ext(&file_name(&o))) {
                (None, Some((_, e))) => (o, fmt_of(e)?),
                (f, _) => {
                    let fmt = fmt_of(f.unwrap_or(&default))?;
                    (o.with_file_name(format!("{}.{fmt}", file_name(&o))), fmt)
                }
            };
            if exists(&o) {
                return Err(format!("“{}” already exists", file_name(&o)));
            }
            (o, fmt)
        }
        None => {
            let stem = if items.len() == 1 {
                let n = file_name(&items[0]);
                // a file loses its extension (photo.jpg -> photo.zip), a folder keeps its name
                match (fs::symlink_metadata(&items[0]).map(|m| m.is_dir()), n.rsplit_once('.')) {
                    (Ok(false), Some((s, _))) if !s.is_empty() => s.to_string(),
                    _ => n,
                }
            } else {
                "Archive".into()
            };
            let fmt = fmt_of(format.as_deref().unwrap_or(&default))?;
            (unique(&base, &stem, &fmt), fmt)
        }
    };
    let dir = dest.parent().unwrap().to_path_buf();
    if !dir.is_dir() {
        return Err(format!("“{}” is not a folder", dir.display()));
    }
    let password = if ask_password {
        if !matches!(fmt.as_str(), "zip" | "7z") {
            return Err("a password needs the zip or 7z format".into());
        }
        Some(read_password()?)
    } else {
        None
    };
    // Written under a hidden name that keeps the extension (zip would add its own otherwise), then renamed
    let tmp = dir.join(format!(".{}.{}.part.{fmt}", file_name(&dest), std::process::id()));
    let result = (|| {
        let mut a: Vec<OsString> = Vec::new();
        let mut envs: Vec<(&str, String)> = Vec::new();
        let lv = |fast: &str, normal: &str, best: &str| match level.as_str() {
            "fast" => fast.to_string(),
            "best" => best.to_string(),
            _ => normal.to_string(),
        };
        let program = match fmt.as_str() {
            // With a password, classic zip (weak encryption, password as an argument) is avoided: 7z writes an AES-256 zip
            "zip" if password.is_some() => {
                a.extend([os("a"), os("-tzip"), os("-mem=AES256"), os("-p"), os("-y"), os("-bd"), os("-snl")]);
                a.push(os(&lv("-mx=1", "-mx=5", "-mx=9")));
                if skip_hidden {
                    a.push(os("-xr!.*"));
                }
                a.push(tmp.clone().into_os_string());
                a.extend(rels.iter().map(|r| os(&format!("./{r}"))));
                "7z"
            }
            "zip" => {
                a.extend([os("-q"), os("-r"), os("-y"), os(&lv("-1", "-6", "-9")), tmp.clone().into_os_string()]);
                a.extend(rels.iter().map(|r| os(&format!("./{r}"))));
                if skip_hidden {
                    a.extend([os("-x"), os("*/.*")]);
                }
                "zip"
            }
            "7z" => {
                a.extend([os("a"), os("-y"), os("-bd"), os("-snl"), os(&lv("-mx=1", "-mx=5", "-mx=9"))]);
                if password.is_some() {
                    // file names encrypted too
                    a.extend([os("-mhe=on"), os("-p")]);
                }
                if skip_hidden {
                    a.push(os("-xr!.*"));
                }
                a.push(tmp.clone().into_os_string());
                a.extend(rels.iter().map(|r| os(&format!("./{r}"))));
                "7z"
            }
            _ => {
                let flag = match fmt.as_str() {
                    "tar.gz" => "-czf",
                    "tar.xz" => "-cJf",
                    _ => "--zstd",
                };
                if skip_hidden {
                    a.push(os("--exclude=.*"));
                }
                a.push(os(flag));
                if flag == "--zstd" {
                    a.push(os("-cf"));
                }
                match fmt.as_str() {
                    "tar.gz" => envs.push(("GZIP", lv("-1", "-6", "-9"))),
                    "tar.xz" => envs.push(("XZ_OPT", lv("-1", "-6", "-9"))),
                    _ => envs.push(("ZSTD_CLEVEL", lv("1", "3", "19"))),
                }
                a.extend([tmp.clone().into_os_string(), os("--")]);
                a.extend(rels.iter().map(|r| os(r)));
                "tar"
            }
        };
        run(program, &a, &base, &envs, password.as_deref())?;
        fs::rename(&tmp, &dest).map_err(|e| e.to_string())
    })();
    if let Err(e) = result {
        let _ = fs::remove_file(&tmp);
        return Err(format!("cannot compress: {e}"));
    }
    Ok(vec![dest])
}

// ---- unpack --------------------------------------------------------------------------------------------

fn unpack_one(archive: &Path, into: Option<&Path>, password: Option<&str>) -> Result<PathBuf, String> {
    let archive = std::path::absolute(archive).map_err(|e| e.to_string())?;
    let name = file_name(&archive);
    if !archive.is_file() {
        return Err(format!("“{name}” is not a file"));
    }
    let (stem, ext) = split_ext(&name).map(|(s, e)| (s, e)).unwrap_or_else(|| (name.clone(), ""));
    let dir = into.map(Path::to_path_buf).unwrap_or_else(|| archive.parent().unwrap().to_path_buf());
    if !dir.is_dir() {
        return Err(format!("“{}” is not a folder", dir.display()));
    }
    // Extraction into a hidden folder next to the result; the result only appears at the end
    let tmp = dir.join(format!(".{stem}.{}.part", std::process::id()));
    fs::create_dir(&tmp).map_err(|e| format!("cannot write in “{}”: {e}", dir.display()))?;
    let args: Vec<OsString> = if is_tar(ext) {
        vec![os("-xf"), archive.clone().into_os_string(), os("-C"), tmp.clone().into_os_string()]
    } else {
        vec![
            os("x"),
            os("-y"),
            os("-bd"),
            os("-snl"),
            os(&format!("-o{}", tmp.display())),
            os("--"),
            archive.clone().into_os_string(),
        ]
    };
    let program = if is_tar(ext) { "tar" } else { "7z" };
    let done = run(program, &args, &dir, &[], password).and_then(|()| {
        let mut top: Vec<PathBuf> =
            fs::read_dir(&tmp).map_err(|e| e.to_string())?.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        if top.is_empty() {
            return Err("the archive is empty".into());
        }
        let (src, dest) = if top.len() == 1 {
            let only = top.remove(0);
            let n = file_name(&only);
            let (s, e) = match n.rsplit_once('.') {
                Some((s, e)) if !s.is_empty() && !only.is_dir() => (s.to_string(), e.to_string()),
                _ => (n.clone(), String::new()),
            };
            let dest = unique(&dir, &s, &e);
            (only, dest)
        } else {
            (tmp.clone(), unique(&dir, &stem, ""))
        };
        fs::rename(&src, &dest).map_err(|e| e.to_string())?;
        Ok(dest)
    });
    let _ = fs::remove_dir_all(&tmp);
    done.map_err(|e| format!("cannot extract “{name}”: {e}"))
}

fn unpack(args: &[String]) -> (Vec<PathBuf>, Vec<String>) {
    let (mut into, mut archives, mut ask_password) = (None::<PathBuf>, Vec::new(), false);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-o" | "--output" => match it.next() {
                Some(d) => into = Some(PathBuf::from(d)),
                None => return (vec![], vec!["-o needs a folder".into()]),
            },
            "--password-stdin" => ask_password = true,
            "-q" | "--quiet" => {}
            "--" => archives.extend(it.by_ref().map(PathBuf::from)),
            s if s.starts_with('-') && s.len() > 1 => return (vec![], vec![format!("unknown option {s}")]),
            s => archives.push(PathBuf::from(s)),
        }
    }
    if archives.is_empty() {
        return (vec![], vec!["no archive given (usage: fili-archive unpack [--password-stdin] [-o FOLDER] ARCHIVE...)".into()]);
    }
    let password = if ask_password {
        match read_password() {
            Ok(p) => Some(p),
            Err(e) => return (vec![], vec![e]),
        }
    } else {
        None
    };
    let (mut done, mut errors) = (Vec::new(), Vec::new());
    for a in &archives {
        match unpack_one(a, into.as_deref(), password.as_deref()) {
            Ok(p) => done.push(p),
            Err(e) => errors.push(e),
        }
    }
    (done, errors)
}

fn list(args: &[String]) -> Result<(), String> {
    let [a] = args else { return Err("usage: fili-archive list ARCHIVE".into()) };
    let name = file_name(Path::new(a));
    let tar = split_ext(&name).map(|(_, e)| is_tar(e)).unwrap_or(false);
    let st = if tar {
        Command::new("tar").args(["-tvf", a]).status()
    } else {
        Command::new("7z").args(["l", "-bd", "--", a]).status()
    }
    .map_err(|e| e.to_string())?;
    if st.success() { Ok(()) } else { Err(format!("cannot read “{name}”")) }
}

// ---- transaction and announcement ----------------------------------------------------------------------------

fn tx_made(paths: &[PathBuf], label: &str) {
    let tx = format!("{}/.local/bin/fili-tx", env::var("HOME").unwrap_or_default());
    let tx = if Path::new(&tx).exists() { tx } else { "fili-tx".into() };
    let quiet = |args: &[OsString]| {
        Command::new(&tx).args(args).stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok_and(|s| s.success())
    };
    if quiet(&[os("begin"), os(label)]) {
        let mut a = vec![os("made")];
        a.extend(paths.iter().map(|p| p.clone().into_os_string()));
        quiet(&a);
    }
}

fn size_of(p: &Path) -> String {
    let n = fs::metadata(p).map(|m| m.len()).unwrap_or(0) as f64;
    match n {
        n if n >= 1e9 => format!("{:.1} GB", n / 1e9),
        n if n >= 1e6 => format!("{:.1} MB", n / 1e6),
        n if n >= 1e3 => format!("{:.0} kB", n / 1e3),
        n => format!("{n:.0} B"),
    }
}

fn notify(title: &str, body: &str, urgent: bool) {
    let _ = Command::new("notify-send")
        .args(["-a", crate::about::ARCHIVE_NAME, "-u", if urgent { "critical" } else { "normal" }, title, body])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

// Entry point of `fili-archive ...` and `fili-file archive ...`; returns the exit code
pub fn run_cli(args: &[String]) -> i32 {
    let Some(cmd) = args.first().map(String::as_str) else {
        eprintln!("usage: fili-archive pack|unpack|open|list|about ... (fili-archive --help)");
        return 2;
    };
    let rest = &args[1..];
    let quiet = rest.iter().any(|a| a == "-q" || a == "--quiet");
    let tty = quiet || std::io::stdout().is_terminal();
    let (done, errors, compress, verb) = match cmd {
        "pack" | "compress" => match pack(rest) {
            Ok(p) => (p, vec![], true, tr("Archive created")),
            Err(e) => (vec![], vec![e], true, ""),
        },
        "unpack" | "extract" | "open" => {
            let (d, e) = unpack(rest);
            (d, e, false, tr("Extracted"))
        }
        "list" => match list(rest) {
            Ok(()) => return 0,
            Err(e) => {
                eprintln!("fili-archive: {e}");
                return 1;
            }
        },
        "about" | "--about" => {
            print_about();
            return 0;
        }
        "-V" | "--version" => {
            println!("fili-archive {}", crate::about::version());
            return 0;
        }
        "-h" | "--help" | "help" => {
            println!(
                "fili-archive pack [-f zip|7z|tar.gz|tar.xz|tar.zst] [-l fast|normal|best] [--no-hidden] [--password-stdin] [-q] [-o OUT] FILE...\nfili-archive unpack [--password-stdin] [-o FOLDER] ARCHIVE...\nfili-archive open ARCHIVE...\nfili-archive list ARCHIVE\nfili-archive about"
            );
            return 0;
        }
        _ => {
            eprintln!("fili-archive: unknown command “{cmd}”");
            return 2;
        }
    };
    if !done.is_empty() {
        if !quiet {
            tx_made(&done, if compress { tr("Compress") } else { tr("Extract") });
        }
        for p in &done {
            println!("{}", p.display());
        }
        if !tty {
            let body = if done.len() == 1 {
                format!("{} ({})", file_name(&done[0]), size_of(&done[0]))
            } else {
                format!("{} {}", done.len(), tr("items"))
            };
            notify(verb, &body, false);
        }
    }
    for e in &errors {
        eprintln!("fili-archive: {e}");
        if !tty {
            notify(if compress { tr("Compress failed") } else { tr("Extract failed") }, e, true);
        }
    }
    if errors.is_empty() { 0 } else { 1 }
}

// The About card of Archiveur (no window: printed). Labels are translated, values are shown as they are.
fn print_about() {
    let default = env::var("FILI_ARCHIVE_FORMAT").unwrap_or_else(|_| "zip".into());
    let rows = [
        (tr("Version"), crate::about::version()),
        (tr("Also known as"), "fili-archive".to_string()),
        (tr("Commands"), "archiveur, fili-archive".to_string()),
        (tr("Writes"), "zip, 7z, tar.gz, tar.xz, tar.zst".to_string()),
        (tr("Reads"), "zip, 7z, rar, iso, tar.*, gz, xz, bz2, zst, cab, jar".to_string()),
        (tr("Default format"), default),
        (tr("Part of"), "Casier, Filigrane".to_string()),
        (tr("Built with"), "Rust".to_string()),
        (tr("Uses"), "tar, zip, 7z".to_string()),
    ];
    println!("{}", crate::about::ARCHIVE_NAME);
    let width = rows.iter().map(|(k, _)| k.chars().count()).max().unwrap_or(0);
    for (k, v) in rows {
        println!("  {k:<width$}  {v}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool(name: &str) -> bool {
        Command::new(name).arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok()
    }

    fn work(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("fili-archive-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn splits_archive_extensions() {
        assert_eq!(split_ext("photos.tar.gz"), Some(("photos".into(), "tar.gz")));
        assert_eq!(split_ext("a.zip"), Some(("a".into(), "zip")));
        assert_eq!(split_ext(".zip"), None);
        assert_eq!(split_ext("notes.txt"), None);
    }

    #[test]
    fn unique_names_never_overwrite() {
        let d = work("unique");
        assert_eq!(unique(&d, "x", "zip"), d.join("x.zip"));
        fs::write(d.join("x.zip"), "").unwrap();
        fs::write(d.join("x 2.zip"), "").unwrap();
        assert_eq!(unique(&d, "x", "zip"), d.join("x 3.zip"));
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn tar_round_trip_next_to_the_files() {
        if !tool("tar") {
            return;
        }
        let d = work("tar");
        fs::write(d.join("a.txt"), "alpha").unwrap();
        let out = pack(&["-f".into(), "tar.gz".into(), d.join("a.txt").to_string_lossy().into_owned()]).unwrap();
        assert_eq!(out, vec![d.join("a.tar.gz")]);
        let into = d.join("out");
        fs::create_dir(&into).unwrap();
        let (done, errors) = unpack(&["-o".into(), into.to_string_lossy().into_owned(), out[0].to_string_lossy().into_owned()]);
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(fs::read_to_string(&done[0]).unwrap(), "alpha");
        // a second pack never overwrites
        let again = pack(&["-f".into(), "tar.gz".into(), d.join("a.txt").to_string_lossy().into_owned()]).unwrap();
        assert_eq!(again, vec![d.join("a 2.tar.gz")]);
        let _ = fs::remove_dir_all(d);
    }

    #[test]
    fn zip_with_7z_or_zip_round_trip() {
        if !tool("tar") || !(tool("zip") || tool("7z")) {
            return;
        }
        let d = work("zip");
        fs::create_dir(d.join("folder")).unwrap();
        fs::write(d.join("folder/b.txt"), "beta").unwrap();
        let Ok(out) = pack(&[d.join("folder").to_string_lossy().into_owned()]) else { return }; // zip missing: 7z may still read
        assert_eq!(out, vec![d.join("folder.zip")]);
        fs::remove_dir_all(d.join("folder")).unwrap();
        let (done, errors) = unpack(&[out[0].to_string_lossy().into_owned()]);
        if tool("7z") {
            assert!(errors.is_empty(), "{errors:?}");
            assert_eq!(fs::read_to_string(done[0].join("b.txt")).unwrap(), "beta");
        }
        let _ = fs::remove_dir_all(d);
    }
}
