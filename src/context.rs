// Contexte partagé avec l'assistant (fili-ai): l'application écrit son état utile (dossier, sélection) dans
// $XDG_RUNTIME_DIR/fili/context/<pid>.json. Rien ne tourne en plus: le fichier est lu à la demande par
// `fili-context`, qui retrouve celui de la fenêtre active par son pid (niri). Contrat commun: voir le skill filigrane-ui.
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

// Ce que l'assistant doit savoir de l'application (contrat commun des fili-*): le rôle, le sens des champs, la voie
// d'exécution conseillée. Court: il part dans le prompt.
const ABOUT: &str = "File manager. location = folder shown (results = title if a result list from ctl show replaces its content), selection = paths the user selected (absolute). \
Act on files with normal shell tools (mv, cp, rg, sha256sum, tar...) using these paths; prefer the shell for batch work. \
fili-file shows changes live (inotify), so no refresh call is needed. To change files use fili-tx (mv, mv - for batches, trash, mkdir, cp), never bare mv/rm: fili-tx undo reverts the whole request. \
To take the user somewhere: fili-file ctl goto PATH (a file is selected in its folder). Menu commands (the fili-bar menu): fili-file ctl action new-folder|copy|paste|rename|trash|undo|reload|... acts on the user's selection. Every ctl command prints a JSON answer and exits 1 with the reason if it did not happen (unknown or disabled action, bad path). fili-file ctl state prints the window state (tabs, folder, selection, view, tasks, open dialog); fili-file ctl actions lists the commands with label, shortcut and whether they are enabled now. To show search results in this window: \
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
// La réponse est une ligne JSON {"ok":true,...} ou {"ok":false,"error":"..."} donnée par l'interface (`reply`), pas par
// le fil du socket: l'accusé dit ce qui s'est passé. Le fil n'existe que dans l'application et dort sur accept.
const REPLY_TIMEOUT: Duration = Duration::from_secs(5);

static PENDING: Mutex<Option<HashMap<i32, Sender<String>>>> = Mutex::new(None);
static NEXT_ID: AtomicI32 = AtomicI32::new(1);

// L'interface répond à la commande `id` (une seule fois; sans effet si la commande a expiré)
pub fn reply(id: i32, text: &str) {
    let tx = PENDING.lock().ok().and_then(|mut p| p.as_mut()?.remove(&id));
    if let Some(tx) = tx {
        let _ = tx.send(text.to_string());
    }
}

// Passe la commande à l'interface (on_command reçoit l'id à rendre à `reply`) et attend sa réponse
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
    // Le socket n'est lisible que par l'utilisateur
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

// Instance visée: celle du pid demandé (--pid: la fili-bar connaît la fenêtre active), sinon celle notée par Mod+Y
// ($FILI_CONTEXT_FILE), sinon la dernière lancée
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

const USAGE: &str = "usage: fili-file ctl [--pid PID] goto PATH | show TITLE < paths | action NAME | state | actions";

// Sépare l'option --pid du reste des arguments
fn split_pid(args: &[String]) -> (Option<String>, &[String]) {
    match args {
        [flag, pid, rest @ ..] if flag == "--pid" => (Some(pid.clone()), rest),
        _ => (None, args),
    }
}

// fili-file ctl [--pid PID] <goto CHEMIN | show TITRE (chemins sur l'entrée standard) | action NOM | state | actions>
// action: déclenche une commande de la barre de menus (new-folder, copy, paste, rename...), la fili-bar les propose
// state: JSON de l'état (onglets, dossier, sélection, tâches); actions: JSON des commandes (libellé, raccourci, activée)
// Renvoie la réponse JSON de la fenêtre; Err si elle dit {"ok":false,...}
pub fn ctl(args: &[String]) -> Result<String, String> {
    let (pid, args) = split_pid(args);
    let (verb, arg) = (args.first().ok_or(USAGE)?, args.get(1).cloned().unwrap_or_default());
    let mut body = String::new();
    match verb.as_str() {
        "goto" | "action" if !arg.is_empty() => {}
        "state" | "actions" => {}
        "show" => {
            let _ = std::io::stdin().take(4 << 20).read_to_string(&mut body);
        }
        _ => return Err(USAGE.into()),
    }
    // Chemin relatif: relatif au dossier de l'appelant, pas à celui de l'application
    let arg = if verb == "goto" { std::path::absolute(&arg).map_or(arg, |p| p.to_string_lossy().into_owned()) } else { arg };
    let mut sock = target(pid.as_deref());
    // Une commande de menu ou une lecture d'état vise une fenêtre ouverte: pas d'instance à lancer pour elle
    if sock.is_none() && matches!(verb.as_str(), "action" | "state" | "actions") {
        return Err("no fili-file window".into());
    }
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
    if verb != "state" && verb != "actions" {
        raise(&sock.file_stem().unwrap_or_default().to_string_lossy());
    }
    if ack.starts_with("{\"ok\":true") { Ok(ack) } else { Err(ack_error(&ack)) }
}

// Message d'erreur d'une réponse {"ok":false,"error":"..."} (la réponse brute si elle est d'une autre forme)
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
    fn pid_optionnel_avant_le_verbe() {
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
    fn la_reponse_vient_de_l_interface() {
        let r = dispatch("action\tcopy\n", Duration::from_secs(2), &|id, verb, arg, body| {
            assert_eq!((verb.as_str(), arg.as_str(), body.as_str()), ("action", "copy", ""));
            std::thread::spawn(move || reply(id, "{\"ok\":true}"));
        });
        assert_eq!(r, "{\"ok\":true}");
    }

    #[test]
    fn sans_reponse_l_echec_est_dit() {
        let r = dispatch("goto\t/tmp\n", Duration::from_millis(50), &|_, _, _, _| {});
        assert!(r.starts_with("{\"ok\":false"), "{r}");
        // Une réponse tardive est sans effet
        reply(1_000_000, "{\"ok\":true}");
    }

    #[test]
    fn commande_vide_refusee_et_corps_transmis() {
        assert!(dispatch("", Duration::from_millis(50), &|_, _, _, _| panic!("not called")).contains("empty"));
        let r = dispatch("show\tT\n/a\n/b\n", Duration::from_secs(2), &|id, _, arg, body| {
            assert_eq!((arg.as_str(), body.as_str()), ("T", "/a\n/b\n"));
            reply(id, "{\"ok\":true}");
        });
        assert_eq!(r, "{\"ok\":true}");
    }

    #[test]
    fn message_d_erreur_extrait() {
        assert_eq!(ack_error("{\"ok\":false,\"error\":\"disabled: no selection\"}"), "disabled: no selection");
        assert_eq!(ack_error("{\"ok\":false,\"error\":\"a \\\"b\\\"\"}"), "a \"b\"");
        assert_eq!(ack_error(""), "no answer");
    }

    #[test]
    fn commande_sans_nom_refusee() {
        assert!(ctl(&v(&["action"])).is_err());
        assert!(ctl(&v(&["--pid", "1", "inconnue", "x"])).is_err());
    }
}
