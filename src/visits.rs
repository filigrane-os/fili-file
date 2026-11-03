// Visited folders, ranked by "recent frequency" (like zoxide): each visit counts, old ones weigh less.
// File ~/.local/state/filigrane/fili-file-visits: "count<TAB>last visit (s)<TAB>path", 200 folders at most.
// Without Qt: tested by cargo test.
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

// Weight: the number of visits, multiplied according to the age of the last one (hour ×4, day ×2, week ×0.5, older ×0.25)
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

// Removes a folder from the list ("Remove from recents"): it comes back if it is visited again
pub fn forget_in(list: &mut Vec<Visit>, path: &str) -> bool {
    let before = list.len();
    list.retain(|v| v.path != path);
    list.len() != before
}

pub fn forget(path: &str) {
    let mut list = load();
    if forget_in(&mut list, path) {
        let _ = std::fs::write(file(), format(&list));
    }
}

// Folders that still exist, best ranked first
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
        // A visit from a minute ago comes before two visits a month old
        let fresh = Visit { count: 1, last: now - 60, path: "/c".into() };
        assert!(score(&fresh, now) > score(&l[0], now));
        assert_eq!(parse(&format(&l)), l);
    }

    #[test]
    fn forgets_a_folder_and_it_comes_back_when_visited_again() {
        let mut l = Vec::new();
        bump(&mut l, "/a", 1000);
        bump(&mut l, "/b", 1001);
        assert!(forget_in(&mut l, "/a"));
        assert!(!forget_in(&mut l, "/a"));
        assert_eq!(l.iter().map(|v| v.path.as_str()).collect::<Vec<_>>(), vec!["/b"]);
        bump(&mut l, "/a", 2000);
        assert_eq!(l.iter().find(|v| v.path == "/a").map(|v| v.count), Some(1));
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
