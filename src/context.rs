// Contexte partagé avec l'assistant (fili-ai): l'application écrit son état utile (dossier, sélection) dans
// $XDG_RUNTIME_DIR/fili/context/<pid>.json. Rien ne tourne en plus: le fichier est lu à la demande par
// `fili-context`, qui retrouve celui de la fenêtre active par son pid (niri). Contrat commun: voir le skill filigrane-ui.
use std::{
    fs,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
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

// Ce que l'assistant doit savoir de l'application (contrat commun des fili-*): le rôle, le sens des champs, la voie
// d'exécution conseillée. Court: il part dans le prompt.
const ABOUT: &str = "File manager. location = folder shown (results = title if a result list from ctl show replaces its content), selection = paths the user selected (absolute). \
Act on files with normal shell tools (mv, cp, rg, sha256sum, tar...) using these paths; prefer the shell for batch work. \
fili-file shows changes live (inotify), so no refresh call is needed. Trash with gio trash, never rm, unless asked. \
To take the user somewhere: fili-file ctl goto PATH (a file is selected in its folder). To show search results in this window: \
find them with fd/rg/find, then fili-file ctl show 'TITLE' < paths (one absolute path per line); both bring the window forward.";

// results: titre de la liste de résultats affichée (vide: le dossier normal)
pub fn publish(app: &str, location: &str, selection: &[String], results: &str) {
    let Some(f) = file() else { return };
    let sel: Vec<String> = selection.iter().take(200).map(|p| json(p)).collect();
    let body = format!(
        "{{\"app\":{},\"about\":{},\"pid\":{},\"location\":{},\"results\":{},\"selection\":[{}],\"selection_count\":{}}}\n",
        json(app),
        json(ABOUT),
        std::process::id(),
        json(location),
        json(results),
        sel.join(","),
        selection.len()
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

// Commandes de l'assistant à cette instance: un socket <pid>.sock à côté du fichier de contexte. Une connexion = une
// commande: première ligne "verbe<TAB>argument", le reste (jusqu'à la fin du flux) est le corps (liste de chemins).
// Le fil n'existe que dans l'application et dort sur accept.
pub fn serve(on_command: impl Fn(String, String, String) + Send + 'static) {
    let Some(f) = file() else { return };
    let sock = f.with_extension("sock");
    let _ = fs::remove_file(&sock);
    let Ok(listener) = UnixListener::bind(&sock) else { return };
    // Le socket n'est lisible que par l'utilisateur
    let _ = fs::set_permissions(&sock, std::os::unix::fs::PermissionsExt::from_mode(0o600));
    std::thread::spawn(move || {
        for mut c in listener.incoming().flatten() {
            let _ = c.set_read_timeout(Some(Duration::from_secs(2)));
            let mut text = String::new();
            let _ = (&mut c).take(4 << 20).read_to_string(&mut text);
            let (head, body) = text.split_once('\n').unwrap_or((&text, ""));
            let (verb, arg) = head.split_once('\t').unwrap_or((head, ""));
            if !verb.is_empty() {
                on_command(verb.to_string(), arg.to_string(), body.to_string());
            }
            let _ = c.write_all(b"ok\n");
        }
    });
}

fn alive(pid: &str) -> bool {
    Path::new(&format!("/proc/{pid}")).is_dir()
}

// Instance visée: celle notée par Mod+Y ($FILI_CONTEXT_FILE), sinon la dernière lancée
fn target() -> Option<PathBuf> {
    let dir = PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR")?).join("fili/context");
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

// Met la fenêtre de l'instance au premier plan (niri): on cherche l'id de la fenêtre qui porte ce pid
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

// fili-file ctl <goto CHEMIN | show TITRE (chemins sur l'entrée standard)>
pub fn ctl(args: &[String]) -> Result<(), String> {
    let (verb, arg) = (args.first().ok_or("usage: fili-file ctl goto PATH | show TITLE < paths")?, args.get(1).cloned().unwrap_or_default());
    let mut body = String::new();
    match verb.as_str() {
        "goto" if !arg.is_empty() => {}
        "show" => {
            let _ = std::io::stdin().take(4 << 20).read_to_string(&mut body);
        }
        _ => return Err("usage: fili-file ctl goto PATH | show TITLE < paths".into()),
    }
    // Chemin relatif: relatif au dossier de l'appelant, pas à celui de l'application
    let arg = if verb == "goto" { std::path::absolute(&arg).map_or(arg, |p| p.to_string_lossy().into_owned()) } else { arg };
    let mut sock = target();
    if sock.is_none() {
        // Aucune instance: on en lance une, puis on attend son socket
        std::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
            .current_dir(if verb == "goto" { "/" } else { "." })
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        for _ in 0..50 {
            std::thread::sleep(Duration::from_millis(100));
            sock = target();
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
    raise(&sock.file_stem().unwrap_or_default().to_string_lossy());
    Ok(())
}
