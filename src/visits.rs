// Dossiers visités, classés par « fréquence récente » (comme zoxide): chaque visite compte, les anciennes pèsent moins.
// Fichier ~/.local/state/filigrane/fili-file-visits: « nombre<TAB>dernière visite (s)<TAB>chemin », 200 dossiers au plus.
// Sans Qt: testé par cargo test.
use std::{
    env,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Visit {
    pub count: u32,
    pub last: i64,
    pub path: String,
}

const KEEP: usize = 200;

fn file() -> PathBuf {
    let home = env::var("HOME").unwrap_or_default();
    let state = env::var("XDG_STATE_HOME").unwrap_or_else(|_| format!("{home}/.local/state"));
    PathBuf::from(state).join("filigrane/fili-file-visits")
}

pub fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

pub fn parse(text: &str) -> Vec<Visit> {
    text.lines()
        .filter_map(|l| {
            let mut f = l.splitn(3, '\t');
            Some(Visit { count: f.next()?.parse().ok()?, last: f.next()?.parse().ok()?, path: f.next()?.to_string() })
        })
        .collect()
}

pub fn format(list: &[Visit]) -> String {
    list.iter().map(|v| format!("{}\t{}\t{}\n", v.count, v.last, v.path)).collect()
}

// Poids: le nombre de visites, multiplié selon l'ancienneté de la dernière (heure ×4, jour ×2, semaine ×0,5, plus ×0,25)
pub fn score(v: &Visit, now: i64) -> f64 {
    let age = (now - v.last).max(0);
    let w = if age < 3600 {
        4.0
    } else if age < 86_400 {
        2.0
    } else if age < 7 * 86_400 {
        0.5
    } else {
        0.25
    };
    v.count as f64 * w
}

pub fn bump(list: &mut Vec<Visit>, path: &str, now: i64) {
    match list.iter_mut().find(|v| v.path == path) {
        Some(v) => {
            v.count += 1;
            v.last = now;
        }
        None => list.push(Visit { count: 1, last: now, path: path.to_string() }),
    }
    if list.len() > KEEP {
        list.sort_by(|a, b| score(b, now).total_cmp(&score(a, now)).then_with(|| b.last.cmp(&a.last)));
        list.truncate(KEEP);
    }
}

pub fn load() -> Vec<Visit> {
    std::fs::read_to_string(file()).map(|t| parse(&t)).unwrap_or_default()
}

pub fn record(path: &str) {
    let mut list = load();
    bump(&mut list, path, now());
    let f = file();
    if let Some(dir) = f.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(f, format(&list));
}

// Dossiers qui existent encore, les mieux classés d'abord
pub fn ranked() -> Vec<Visit> {
    let t = now();
    let mut list: Vec<Visit> = load().into_iter().filter(|v| std::path::Path::new(&v.path).is_dir()).collect();
    list.sort_by(|a, b| score(b, t).total_cmp(&score(a, t)).then_with(|| b.last.cmp(&a.last)));
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_and_ranks_recent_visits_higher() {
        let mut l = Vec::new();
        bump(&mut l, "/a", 1_000_000);
        bump(&mut l, "/a", 1_000_010);
        bump(&mut l, "/b", 1_000_020);
        assert_eq!(l.iter().find(|v| v.path == "/a").map(|v| v.count), Some(2));
        let now = 1_000_000 + 30 * 86_400;
        // Une visite d'il y a une minute passe devant deux visites vieilles d'un mois
        let fresh = Visit { count: 1, last: now - 60, path: "/c".into() };
        assert!(score(&fresh, now) > score(&l[0], now));
        assert_eq!(parse(&format(&l)), l);
    }

    #[test]
    fn keeps_the_best_200() {
        let mut l = Vec::new();
        for i in 0..250 {
            bump(&mut l, &format!("/d{i}"), 1000 + i);
        }
        assert_eq!(l.len(), 200);
        assert!(l.iter().any(|v| v.path == "/d249"));
    }
}
