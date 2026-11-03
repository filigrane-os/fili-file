// Watching of the folder shown (inotify). A thread blocked on poll only wakes up if the folder changes; bursts
// of events (copy of a big folder) are grouped: a single on_change call after 120 ms of calm.
use std::{
    ffi::CString,
    path::Path,
    sync::atomic::{AtomicI32, Ordering},
    sync::Arc,
};

pub struct Watcher {
    fd: i32,
    wd: Arc<AtomicI32>,
}

const MASK: u32 = libc::IN_CREATE
    | libc::IN_DELETE
    | libc::IN_MOVED_FROM
    | libc::IN_MOVED_TO
    | libc::IN_CLOSE_WRITE
    | libc::IN_ATTRIB
    | libc::IN_DELETE_SELF
    | libc::IN_MOVE_SELF
    | libc::IN_ONLYDIR;

impl Watcher {
    pub fn new(on_change: impl Fn() + Send + 'static) -> Option<Watcher> {
        let fd = unsafe { libc::inotify_init1(libc::IN_CLOEXEC) };
        if fd < 0 {
            return None;
        }
        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            let mut pfd = libc::pollfd { fd, events: libc::POLLIN, revents: 0 };
            loop {
                if unsafe { libc::read(fd, buf.as_mut_ptr().cast(), buf.len()) } <= 0 {
                    return;
                }
                // Wait for calm: as long as events arrive, read them
                while unsafe { libc::poll(&mut pfd, 1, 120) } > 0 {
                    if unsafe { libc::read(fd, buf.as_mut_ptr().cast(), buf.len()) } <= 0 {
                        return;
                    }
                }
                on_change();
            }
        });
        Some(Watcher { fd, wd: Arc::new(AtomicI32::new(-1)) })
    }

    // Changes the folder being watched
    pub fn watch(&self, dir: &Path) {
        let old = self.wd.swap(-1, Ordering::SeqCst);
        if old >= 0 {
            unsafe { libc::inotify_rm_watch(self.fd, old) };
        }
        let Ok(c) = CString::new(dir.as_os_str().as_encoded_bytes()) else { return };
        let wd = unsafe { libc::inotify_add_watch(self.fd, c.as_ptr(), MASK) };
        self.wd.store(wd, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, sync::mpsc, time::Duration};

    #[test]
    fn reports_changes_once_per_burst() {
        let dir = std::env::temp_dir().join(format!("fili-file-watch-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let (tx, rx) = mpsc::channel();
        let w = Watcher::new(move || {
            let _ = tx.send(());
        })
        .unwrap();
        w.watch(&dir);
        for i in 0..20 {
            fs::write(dir.join(format!("f{i}")), "x").unwrap();
        }
        assert!(rx.recv_timeout(Duration::from_secs(2)).is_ok());
        assert!(rx.recv_timeout(Duration::from_millis(400)).is_err(), "une seule notification par rafale");
        fs::remove_dir_all(&dir).unwrap();
    }
}
