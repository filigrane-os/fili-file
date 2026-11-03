#!/usr/bin/env python3
"""Scenarios of fili-file (see run.sh): one instance without a display, driven by `fili-file ctl`, state read through
`ctl state`, effects checked on the file system. TAP output: "ok N - name" or
"not ok N - name" followed by the reason, then "# done: X failure(s) of N". Exit code 0 if everything passes."""
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

T = Path(os.environ["FILI_TEST_ROOT"])
BIN = os.environ["FILI_FILE_BIN"]
VERBOSE = bool(os.environ.get("FILI_VERBOSE"))
DATA = T / "home" / "data"
SOCK_DIR = T / "run" / "fili" / "context"


class App:
    def __init__(self, env=None, name="app"):
        """`env` overrides variables of the environment (a second instance with another PATH, for example)."""
        self.log = open(T / f"{name}.log", "w")
        self.proc = subprocess.Popen([BIN, str(DATA)], stdout=self.log, stderr=self.log, start_new_session=True, env={**os.environ, **(env or {})})
        self.pid = self.proc.pid
        if name == "app":
            (T / "pid").write_text(str(self.pid))
        self.wait(lambda: (SOCK_DIR / f"{self.pid}.sock").exists(), 20, "socket de l'application")

    def ctl(self, *args, stdin=None):
        """Runs `fili-file ctl --pid PID ARGS`; returns (code, JSON of the answer or None, raw text)."""
        r = subprocess.run([BIN, "ctl", "--pid", str(self.pid), *args], input=stdin, capture_output=True, text=True, timeout=20)
        out = r.stdout.strip()
        try:
            data = json.loads(out) if out else None
        except ValueError:
            data = None
        if VERBOSE:
            print(f"#   ctl {' '.join(args)} -> rc={r.returncode} {out or r.stderr.strip()}")
        return r.returncode, data, out or r.stderr.strip()

    def ok(self, *args, stdin=None):
        rc, data, raw = self.ctl(*args, stdin=stdin)
        if rc != 0:
            raise AssertionError(f"ctl {' '.join(args)}: code {rc}: {raw}")
        return data

    def fails(self, *args, stdin=None, containing=""):
        """The command must fail (code 1) with this text in the error; returns the message."""
        rc, _, raw = self.ctl(*args, stdin=stdin)
        if rc != 1:
            raise AssertionError(f"ctl {' '.join(args)}: expected a failure (code 1), got code {rc}: {raw}")
        if containing not in raw:
            raise AssertionError(f"ctl {' '.join(args)}: \"{containing}\" not found in \"{raw}\"")
        return raw

    def state(self):
        return self.ok("state")["state"]

    def wait(self, pred, seconds=8, what="condition"):
        end = time.time() + seconds
        while time.time() < end:
            if self.proc.poll() is not None and what != "socket de l'application":
                raise AssertionError(f"fili-file stopped (code {self.proc.returncode}), see app.log")
            try:
                if pred():
                    return
            except (AssertionError, KeyError):
                pass
            time.sleep(0.1)
        raise AssertionError(f"timeout: {what}")


def names(d):
    return sorted(p.name for p in Path(d).iterdir())


def make(name, files):
    """Fresh folder DATA/name with these files ({name: content}; a name ending with / = folder); the application goes there."""
    d = DATA / name
    d.mkdir()
    for f, content in files.items():
        if f.endswith("/"):
            (d / f).mkdir()
        else:
            (d / f).write_text(content)
    return d


def eq(got, want, what=""):
    if got != want:
        raise AssertionError(f"{what}: got {got!r}, expected {want!r}".strip(": "))


def have(tool):
    """Whether the application can find `tool`: PATH only (HOME is a test directory, ~/.local/bin does not count)."""
    return shutil.which(tool, path=os.environ.get("PATH", "")) is not None


class Skip(Exception):
    """The scenario cannot run here (reported as "ok # SKIP" with the reason)."""


app = None
results = []


def test(name):
    def deco(fn):
        results.append((name, fn))
        return fn
    return deco


# ---------------------------------------------------------------- startup and the ctl contract

@test("startup: state of the window")
def _():
    s = app.state()
    eq(s["location"], str(DATA), "folder")
    eq(s["tabs"], [str(DATA)], "tabs")
    eq((s["view"], s["hidden_shown"], s["preview"], s["dialog"], s["in_trash"], s["results"]), ("list", False, False, "", False, ""))
    assert s["can_act"] and s["pid"] == app.pid


@test("actions: full list with labels and shortcuts")
def _():
    acts = {a["name"]: a for a in app.ok("actions")["actions"]}
    assert len(acts) >= 41, len(acts)
    for n in ("copy", "cut", "paste", "rename", "trash", "undo", "new-folder", "duplicate", "compress", "extract", "select-all", "new-tab", "reload"):
        assert n in acts, n
    eq((acts["copy"]["label"], acts["copy"]["keys"]), ("Copy", "Ctrl+C"))
    assert all(isinstance(a["enabled"], bool) and a["label"] for a in acts.values())


@test("errors: unknown action, disabled one, useless argument, unknown command")
def _():
    app.fails("action", "nope", containing="unknown action")
    app.fails("action", "empty-trash", containing="disabled")
    app.fails("action", "undo", "foo", containing="takes no argument")
    app.fails("goto", str(DATA / "missing"), containing="not found")
    app.fails("dialog", "cancel", containing="no dialog")


# ---------------------------------------------------------------- navigation

@test("navigation: goto, back, forward, parent")
def _():
    d = make("nav", {"a/": "", "b/": ""})
    app.ok("goto", str(d / "a"))
    eq(app.state()["location"], str(d / "a"))
    app.ok("goto", str(d / "b"))
    app.ok("action", "back")
    eq(app.state()["location"], str(d / "a"), "back")
    app.ok("action", "forward")
    eq(app.state()["location"], str(d / "b"), "before")
    app.ok("action", "up")
    eq(app.state()["location"], str(d), "parent")


@test("navigation: a file is selected in its folder; reveal too (folder included)")
def _():
    d = make("goto-file", {"f.txt": "x", "sub/": ""})
    app.ok("goto", str(d / "f.txt"))
    s = app.state()
    eq((s["location"], s["selection"]), (str(d), [str(d / "f.txt")]))
    app.ok("goto", str(DATA))
    app.ok("reveal", str(d / "sub"))
    s = app.state()
    eq((s["location"], s["selection"]), (str(d), [str(d / "sub")]))
    app.fails("reveal", str(d / "absent"), containing="not found in")
    eq(app.state()["selection"], [str(d / "sub")], "selection unchanged")


@test("navigation: relative path resolved from the caller's folder")
def _():
    d = make("rel", {"in/": ""})
    r = subprocess.run([BIN, "ctl", "--pid", str(app.pid), "goto", "in"], cwd=d, capture_output=True, text=True)
    eq(r.returncode, 0, r.stdout + r.stderr)
    eq(app.state()["location"], str(d / "in"))


# ---------------------------------------------------------------- selection

@test("selection: replace, add, clear, pattern")
def _():
    d = make("sel", {"a.txt": "", "b.txt": "", "c.pdf": ""})
    app.ok("goto", str(d))
    eq(app.ok("select", stdin=f"{d}/a.txt\n{d}/c.pdf\n")["selected"], 2)
    eq(sorted(app.state()["selection"]), [str(d / "a.txt"), str(d / "c.pdf")])
    eq(app.ok("select", "--add", stdin=f"{d}/b.txt\n")["selected"], 3)
    eq(app.ok("select", "--pattern", "*.pdf")["selected"], 1)
    eq(app.state()["selection"], [str(d / "c.pdf")])
    eq(app.ok("select", "--none")["selected"], 0)
    eq(app.state()["selection"], [])


@test("selection: a path outside the folder is an error and is not selected")
def _():
    d = make("sel-bad", {"a.txt": ""})
    app.ok("goto", str(d))
    app.fails("select", stdin=f"{d}/a.txt\n{d}/absent\n", containing="absent")
    eq(app.state()["selection"], [str(d / "a.txt")], "the present one stays selected")


@test("selection: all, invert, same type, pattern by action")
def _():
    d = make("sel2", {"a.txt": "", "b.txt": "", "c.pdf": ""})
    app.ok("goto", str(d))
    app.ok("action", "select-all")
    eq(len(app.state()["selection"]), 3)
    app.ok("select", stdin=f"{d}/a.txt\n")
    app.ok("action", "invert-selection")
    eq(sorted(app.state()["selection"]), [str(d / "b.txt"), str(d / "c.pdf")])
    app.ok("select", stdin=f"{d}/a.txt\n")
    app.ok("action", "select-same-kind")
    eq(sorted(app.state()["selection"]), [str(d / "a.txt"), str(d / "b.txt")])
    eq(app.ok("action", "select-matching", "c*")["selected"], 1)


@test("selection: hidden files only exist once shown")
def _():
    d = make("hid", {"a": "", ".h": ""})
    app.ok("goto", str(d))
    app.fails("select", stdin=f"{d}/.h\n", containing=".h")
    app.ok("hidden", "on")
    eq(app.state()["hidden_shown"], True)
    app.ok("select", stdin=f"{d}/.h\n")
    app.ok("hidden", "off")
    eq(app.state()["hidden_shown"], False)


# ---------------------------------------------------------------- tabs and display

@test("hidden files: the choice is kept between two launches")
def _():
    global app
    app.ok("hidden", "on")
    ui = T / "home" / ".local" / "state" / "filigrane" / "fili-file-ui"
    assert "hidden=1" in ui.read_text().split(), ui.read_text()
    app.proc.terminate()
    app.proc.wait(10)
    app = App()
    eq(app.state()["hidden_shown"], True, "reopening")
    app.ok("hidden", "off")
    assert "hidden=0" in ui.read_text().split(), ui.read_text()
    app.proc.terminate()
    app.proc.wait(10)
    app = App()
    eq(app.state()["hidden_shown"], False, "reopening after unchecking")


@test("tabs: new, switch, close, last tab")
def _():
    d = make("tabs", {"x/": ""})
    app.ok("goto", str(d))
    app.ok("tab", "new", str(d / "x"))
    s = app.state()
    eq((s["tab"], s["tabs"], s["location"]), (1, [str(d), str(d / "x")], str(d / "x")))
    app.ok("tab", "switch", "0")
    s = app.state()
    eq((s["tab"], s["location"]), (0, str(d)))
    app.ok("tab", "close", "1")
    eq(len(app.state()["tabs"]), 1)
    app.fails("tab", "close", containing="last tab")
    app.fails("tab", "switch", "5", containing="no such tab")


@test("tabs: reordering keeps the tab shown")
def _():
    d = make("tabs2", {"a/": "", "b/": "", "c/": ""})
    app.ok("goto", str(d / "a"))
    app.ok("tab", "new", str(d / "b"))
    app.ok("tab", "new", str(d / "c"))
    s = app.state()
    eq((s["tabs"], s["tab"]), ([str(d / "a"), str(d / "b"), str(d / "c")], 2))
    app.ok("tab", "move", "0", "2")
    s = app.state()
    eq((s["tabs"], s["tab"], s["location"]), ([str(d / "b"), str(d / "c"), str(d / "a")], 1, str(d / "c")), "the tab shown (c) follows")
    app.ok("tab", "move", "2", "0")
    s = app.state()
    eq((s["tabs"], s["tab"]), ([str(d / "a"), str(d / "b"), str(d / "c")], 2))
    app.fails("tab", "move", "0", "9", containing="no such tab")
    app.ok("tab", "close", "2")
    app.ok("tab", "close", "1")


@test("tabs: detaching a tab opens it in a new window, on its folder")
def _():
    import signal
    d = make("detach", {"a/": "", "b/": ""})
    app.ok("goto", str(d / "a"))
    app.ok("tab", "new", str(d / "b"))
    before = set(SOCK_DIR.glob("*.sock"))
    app.fails("tab", "detach", "9", containing="no such tab")
    app.ok("action", "detach-tab")
    s = app.state()
    eq((s["tabs"], s["location"]), ([str(d / "a")], str(d / "a")), "the tab left the window")
    app.wait(lambda: len(set(SOCK_DIR.glob("*.sock")) - before) == 1, 20, "new window")
    pid = int((set(SOCK_DIR.glob("*.sock")) - before).pop().stem)
    try:
        r = subprocess.run([BIN, "ctl", "--pid", str(pid), "state"], capture_output=True, text=True, timeout=20)
        eq(json.loads(r.stdout)["state"]["location"], str(d / "b"), "folder of the new window")
    finally:
        os.kill(pid, signal.SIGTERM)
    app.fails("tab", "detach", containing="last tab")
    app.fails("action", "detach-tab", containing="disabled")


@test("display: list/icons, preview, sort")
def _():
    app.ok("view", "icons")
    eq(app.state()["view"], "icons")
    app.ok("view", "list")
    eq(app.state()["view"], "list")
    app.ok("preview", "on")
    eq(app.state()["preview"], True)
    app.ok("preview", "off")
    eq(app.state()["preview"], False)
    app.ok("sort", "size", "desc")
    eq(app.state()["sort"], "size desc")
    app.ok("sort", "size", "asc")
    eq(app.state()["sort"], "size asc")
    app.ok("sort", "name")
    eq(app.state()["sort"], "name asc")
    app.fails("sort", "nope", containing="usage")


# ---------------------------------------------------------------- file operations

@test("copy/paste in another folder (the original stays)")
def _():
    d = make("cp", {"a.txt": "alpha", "dst/": ""})
    app.ok("goto", str(d))
    app.ok("select", stdin=f"{d}/a.txt\n")
    app.ok("action", "copy")
    app.ok("goto", str(d / "dst"))
    app.ok("action", "paste")
    app.wait(lambda: (d / "dst" / "a.txt").exists(), what="pasted file")
    eq((d / "dst" / "a.txt").read_text(), "alpha")
    assert (d / "a.txt").exists()


@test("cut/paste moves")
def _():
    d = make("mv", {"a.txt": "alpha", "dst/": ""})
    app.ok("goto", str(d))
    app.ok("select", stdin=f"{d}/a.txt\n")
    app.ok("action", "cut")
    app.ok("goto", str(d / "dst"))
    app.ok("action", "paste")
    app.wait(lambda: (d / "dst" / "a.txt").exists() and not (d / "a.txt").exists(), what="moved file")


@test("clipboard: copy the path, the name, the base name, the folder")
def _():
    d = make("clip", {"rapport.final.txt": ""})
    app.ok("goto", str(d))
    app.ok("select", stdin=f"{d}/rapport.final.txt\n")
    clip = T / "clip" / "text_plain"
    for action, want in (("copy-path", str(d / "rapport.final.txt")), ("copy-name", "rapport.final.txt"), ("copy-stem", "rapport.final"), ("copy-parent", str(d))):
        app.ok("action", action)
        app.wait(lambda: clip.exists() and clip.read_text() == want, 3, f"{action} in the clipboard")


@test("duplicate")
def _():
    d = make("dup", {"a.txt": "alpha"})
    app.ok("goto", str(d))
    app.ok("select", stdin=f"{d}/a.txt\n")
    app.ok("action", "duplicate")
    app.wait(lambda: (d / "a copy.txt").exists(), what="copy")
    eq((d / "a copy.txt").read_text(), "alpha")


def conflict(name, answer, check):
    d = make(name, {"a.txt": "new", "dst/": ""})
    (d / "dst" / "a.txt").write_text("old")
    app.ok("goto", str(d))
    app.ok("select", stdin=f"{d}/a.txt\n")
    app.ok("action", "copy")
    app.ok("goto", str(d / "dst"))
    app.ok("action", "paste")
    app.wait(lambda: app.state()["dialog"] == "conflict", what="conflict dialog")
    app.fails("action", "copy", containing="disabled")
    app.ok("dialog", answer)
    app.wait(lambda: app.state()["dialog"] == "" and not app.state()["tasks"], what="end of the operation")
    check(d / "dst")


@test("name conflict: replace (the old one goes to the trash)")
def _():
    def check(dst):
        eq((dst / "a.txt").read_text(), "new")
        eq(names(dst), ["a.txt"])
    conflict("cf-replace", "replace", check)


@test("name conflict: keep both")
def _():
    def check(dst):
        eq((dst / "a.txt").read_text(), "old")
        eq(names(dst), ["a copy.txt", "a.txt"])
        eq((dst / "a copy.txt").read_text(), "new")
    conflict("cf-both", "both", check)


@test("name conflict: skip")
def _():
    def check(dst):
        eq((dst / "a.txt").read_text(), "old")
        eq(names(dst), ["a.txt"])
    conflict("cf-skip", "skip", check)


@test("name conflict: cancel")
def _():
    def check(dst):
        eq((dst / "a.txt").read_text(), "old")
        eq(names(dst), ["a.txt"])
    conflict("cf-cancel", "cancel", check)


@test("rename by argument; refused if the name exists")
def _():
    d = make("ren", {"a.txt": "", "b.txt": ""})
    app.ok("goto", str(d))
    app.ok("select", stdin=f"{d}/a.txt\n")
    app.ok("action", "rename", "z.txt")
    eq(names(d), ["b.txt", "z.txt"])
    eq(app.state()["selection"], [str(d / "z.txt")])
    app.fails("action", "rename", "b.txt", containing="")
    eq(names(d), ["b.txt", "z.txt"], "nothing changed")
    app.ok("select", stdin=f"{d}/b.txt\n{d}/z.txt\n")
    app.fails("action", "rename", "q.txt", containing="exactly one")


@test("new folder by argument")
def _():
    d = make("nf", {})
    app.ok("goto", str(d))
    app.ok("action", "new-folder", "projects")
    assert (d / "projects").is_dir(), names(d)
    eq(app.state()["selection"], [str(d / "projects")])


@test("trash then undo; emptying the trash asks for confirmation")
def _():
    d = make("trash", {"a.txt": "alpha", "b.txt": ""})
    app.ok("goto", str(d))
    app.ok("select", stdin=f"{d}/a.txt\n")
    app.ok("action", "trash")
    app.wait(lambda: not (d / "a.txt").exists(), what="file in the trash")
    trash = T / "home" / ".local" / "share" / "Trash" / "files"
    assert (trash / "a.txt").exists(), names(trash) if trash.exists() else "no trash"
    app.ok("action", "undo")
    app.wait(lambda: (d / "a.txt").exists(), what="restauration")
    eq((d / "a.txt").read_text(), "alpha")
    # In the trash: emptying asks for confirmation; cancelling deletes nothing
    app.ok("select", stdin=f"{d}/b.txt\n")
    app.ok("action", "trash")
    app.wait(lambda: (trash / "b.txt").exists())
    app.ok("goto", str(trash))
    s = app.state()
    assert s["in_trash"], s
    app.ok("action", "empty-trash")
    eq(app.state()["dialog"], "confirm")
    app.ok("dialog", "cancel")
    assert (trash / "b.txt").exists()
    app.ok("action", "empty-trash")
    app.ok("dialog", "accept")
    app.wait(lambda: not (trash / "b.txt").exists(), what="trash emptied")


@test("trash of another volume: gathered in the Trash, put back, emptied with the others")
def _():
    import shutil
    import tempfile
    shm = Path("/dev/shm")
    uid = os.getuid()
    leftover = shm / f".Trash-{uid}"
    if not os.access(shm, os.W_OK) or os.stat(shm).st_dev == os.stat(T).st_dev or leftover.exists():
        raise Skip("/dev/shm absent, on the same volume as the test, or already a trash")
    d = Path(tempfile.mkdtemp(dir=shm, prefix="fili-file-test."))
    try:
        (d / "v.txt").write_text("volume")
        app.ok("goto", str(d))
        app.ok("select", stdin=f"{d}/v.txt\n")
        app.ok("action", "trash")
        app.wait(lambda: not (d / "v.txt").exists(), what="file in the trash of the volume")
        vtrash = leftover / "files" / "v.txt"
        assert vtrash.exists(), "not in .Trash-UID of the volume"
        app.ok("goto", str(T / "home" / ".local" / "share" / "Trash" / "files"))
        s = app.state()
        assert s["in_trash"], s
        # Casier's Trash shows the item of the other volume (selectable = present in the view)
        app.ok("select", stdin=f"{vtrash}\n")
        app.ok("action", "put-back")
        app.wait(lambda: (d / "v.txt").exists() and not vtrash.exists(), what="put back from the trash of the volume")
        eq((d / "v.txt").read_text(), "volume")
        # Emptying the trash also empties the one of the volume
        app.ok("goto", str(d))
        app.ok("select", stdin=f"{d}/v.txt\n")
        app.ok("action", "trash")
        app.wait(lambda: vtrash.exists())
        app.ok("goto", str(T / "home" / ".local" / "share" / "Trash" / "files"))
        app.ok("action", "empty-trash")
        app.ok("dialog", "accept")
        app.wait(lambda: not vtrash.exists(), what="trash of the volume emptied")
    finally:
        shutil.rmtree(d, ignore_errors=True)
        shutil.rmtree(leftover, ignore_errors=True)


@test("close: refused while a copy is in progress, accepted afterwards")
def _():
    d = make("busy", {"dst/": "", "tree/": ""})
    for i in range(40):
        sub = d / "tree" / f"d{i}"
        sub.mkdir()
        for j in range(700):
            (sub / f"f{j}").write_bytes(b"")
    app.ok("goto", str(d))
    app.ok("select", stdin=f"{d}/tree\n")
    app.ok("action", "copy")
    app.ok("goto", str(d / "dst"))
    app.ok("action", "paste")
    caught = False
    for _ in range(60):
        if app.state()["tasks"]:
            rc, _, raw = app.ctl("action", "close")
            eq(rc, 0, "the close command is accepted")
            s = app.state()
            if s["tasks"]:
                assert "operation is in progress" in s["notice"], s["notice"]
                assert app.proc.poll() is None, "the window closed during the copy"
                caught = True
            break
        time.sleep(0.05)
    app.wait(lambda: not app.state()["tasks"] and (d / "dst" / "tree" / "d39" / "f699").exists(), 60, "end of the copy")
    if not caught:
        raise Skip("copy too fast to be interrupted")


@test("compress (zip, 7z, tar.gz) then extract")
def _():
    if not have("fili-archive"):
        raise Skip("fili-archive is not installed")
    d = make("arc", {"a.txt": "alpha"})
    app.ok("goto", str(d))
    app.ok("select", stdin=f"{d}/a.txt\n")
    for n in ("one.zip", "two.7z", "three.tar.gz"):
        app.ok("action", "compress", n)
        app.wait(lambda n=n: (d / n).exists() and not app.state()["tasks"], 15, f"archive {n}")
    eq(sorted(names(d)), ["a.txt", "one.zip", "three.tar.gz", "two.7z"])
    app.ok("select", stdin=f"{d}/one.zip\n")
    app.ok("action", "extract")
    app.wait(lambda: len(names(d)) > 4 and not app.state()["tasks"], 15, "extraction")
    extracted = [p for p in names(d) if p not in ("a.txt", "one.zip", "three.tar.gz", "two.7z")]
    # a.txt already exists next to the archive: the extraction takes the first free name
    eq(extracted, ["a 2.txt"], "extracted file")
    eq((d / "a 2.txt").read_text(), "alpha")


# ---------------------------------------------------------------- dialogs, results, search

@test("dialogs: pattern, compress, info, About open, block the shortcuts and close")
def _():
    d = make("dlg", {"a.txt": ""})
    app.ok("goto", str(d))
    app.ok("select", stdin=f"{d}/a.txt\n")
    for action, name in (("select-matching", "match"), ("compress", "compress"), ("info", "info"), ("about", "about")):
        if action == "compress" and not have("fili-archive"):
            continue  # the command is switched off without its program (see the "optional tools" scenario)
        app.ok("action", action)
        app.wait(lambda name=name: app.state()["dialog"] == name, 3, f"dialog {name}")
        app.fails("action", "copy", containing="disabled")
        app.ok("dialog", "cancel")
        app.wait(lambda: app.state()["dialog"] == "", 3, "dialog closed")
    app.ok("action", "info")
    app.wait(lambda: app.state()["dialog"] == "info", 3)
    app.fails("dialog", "accept", containing="no default button")
    app.ok("dialog", "cancel")


@test("quick look (Space): starts quicklook on the current item, disabled without an item")
def _():
    log = T / "quicklook.log"
    d = make("ql", {"a.txt": "", "b.txt": ""})
    app.ok("goto", str(d))
    app.ok("select", stdin=f"{d}/b.txt\n")
    app.ok("action", "quick-look")
    app.wait(lambda: log.exists() and log.read_text().strip().splitlines()[-1] == str(d / "b.txt"), 3, "quicklook started on b.txt")
    acts = {a["name"]: a for a in app.ok("actions")["actions"]}
    eq((acts["quick-look"]["label"], acts["quick-look"]["keys"], acts["quick-look"]["enabled"]), ("Quick Look", "Space", True))
    e = make("ql-empty", {})
    app.ok("goto", str(e))
    app.fails("action", "quick-look", containing="disabled")


@test("open an empty file (0 bytes): application of its type, not xdg-open")
def _():
    log = T / "editor.log"
    d = make("zero", {"empty.txt": "", "no-extension": ""})
    app.ok("goto", str(d))
    for name in ("empty.txt", "no-extension"):
        app.ok("select", stdin=f"{d}/{name}\n")
        app.ok("action", "open")
        app.wait(lambda name=name: log.exists() and log.read_text().strip().splitlines()[-1] == str(d / name), 4, f"{name} opened by the editor of its type")


@test("optional tools missing: compress, extract, share and nearby switch off, the rest keeps working")
def _():
    # The helpers are looked up in PATH and ~/.local/bin (HOME is a test directory): a PATH of the system directories only
    # shows the application without them. Skipped when they are installed system-wide.
    if any(Path(d, tool).exists() for d in ("/usr/bin", "/bin", "/usr/local/bin") for tool in ("fili-archive", "fili-share")):
        raise Skip("fili-archive or fili-share installed in a system directory")
    bare = App(env={"PATH": f"{T}/bin:/usr/bin:/bin"}, name="bare")
    try:
        d = make("bare", {"a.txt": "x"})
        bare.ok("goto", str(d))
        bare.ok("select", stdin=f"{d}/a.txt\n")
        acts = {a["name"]: a for a in bare.ok("actions")["actions"]}
        for n in ("compress", "extract", "share", "nearby"):
            eq(acts[n]["enabled"], False, f"{n} enabled without its program")
        for n in ("copy", "rename", "trash", "new-folder"):
            eq(acts[n]["enabled"], True, f"{n} enabled")
        bare.fails("action", "compress", containing="disabled")
    finally:
        bare.proc.terminate()
        bare.proc.wait(timeout=10)


@test("menu bar: file commands greyed out without a selection, enabled again with one (published context)")
def _():
    d = make("grey", {"a.txt": ""})
    e = make("grey-empty", {})
    ctx = T / "run" / "fili" / "context" / f"{app.pid}.json"
    disabled = lambda: set(json.loads(ctx.read_text())["disabled"])
    # What targets selected items
    needs = {"copy", "cut", "trash", "rename", "duplicate", "open", "quick-look", "copy-path", "copy-name", "select-same-kind"}
    if have("fili-archive"):
        needs.add("compress")
    app.ok("goto", str(e))
    app.wait(lambda: needs <= disabled(), 4, "selection commands greyed out in an empty folder")
    assert not ({"paste", "new-folder", "select-all", "reload"} & disabled()), disabled()
    assert {"empty-trash", "put-back"} <= disabled(), "the trash is not open"
    app.ok("goto", str(d))
    app.ok("select", stdin=f"{d}/a.txt\n")
    app.wait(lambda: not (needs & disabled()), 4, "with a selection, these commands are enabled")
    # An emptied selection (click on the empty space) greys them out again, even if a row stays "current"
    app.ok("select", "--none")
    app.wait(lambda: needs <= disabled(), 4, "selection emptied: greyed out")
    app.fails("action", "copy", containing="disabled")
    # An open dialog also greys out paste and new folder
    app.ok("action", "select-matching")
    app.wait(lambda: {"paste", "new-folder"} <= disabled(), 4, "dialog open: paste greyed out")
    app.ok("dialog", "cancel")
    app.wait(lambda: "paste" not in disabled(), 4, "dialog closed: paste enabled")


@test("widths: columns bounded, reset to zero, unknown column")
def _():
    eq(app.state()["column_widths"], {}, "start")
    eq(app.ok("width", "column", "size", "200")["width"], 200)
    eq(app.ok("width", "column", "modified", "10")["width"], 56, "minimum")
    eq(app.ok("width", "column", "kind", "9999")["width"], 600, "maximum")
    eq(app.state()["column_widths"], {"size": 200, "modified": 56, "kind": 600})
    eq(app.ok("width", "column", "size", "reset")["width"], 96, "default of the column")
    eq(sorted(app.state()["column_widths"]), ["kind", "modified"])
    app.fails("width", "column", "nope", "100", containing="unknown column")
    app.fails("width", "column", "size", "-3", containing="usage")


@test("widths: preview panel bounded and reset to 300")
def _():
    eq(app.state()["preview_width"], 300, "default")
    eq(app.ok("width", "preview", "420")["width"], 420)
    eq(app.state()["preview_width"], 420)
    eq(app.ok("width", "preview", "50")["width"], 160, "minimum")
    eq(app.ok("width", "preview", "5000")["width"], 900, "maximum")
    eq(app.ok("width", "preview", "reset")["width"], 300)


@test("widths: kept in the state and found again after a restart")
def _():
    global app
    app.ok("width", "column", "size", "180")
    app.ok("width", "preview", "380")
    ui = T / "home" / ".local" / "state" / "filigrane" / "fili-file-ui"
    widths = lambda: sorted(l for l in ui.read_text().split() if l.startswith(("col.", "preview=")))
    eq(widths(), ["col.kind=600", "col.modified=56", "col.size=180", "preview=380"], "state file")
    app.proc.terminate()
    app.proc.wait(10)
    app = App()
    s = app.state()
    eq((s["column_widths"], s["preview_width"]), ({"kind": 600, "modified": 56, "size": 180}, 380))
    app.ok("width", "preview", "reset")
    for c in ("size", "modified", "kind"):
        app.ok("width", "column", c, "reset")
    eq(widths(), [], "widths removed from the state")


@test("network: the \"Network\" place shows the connections, goto a folder closes it")
def _():
    d = make("net", {})
    app.ok("goto", str(d))
    app.ok("goto", "network:")
    s = app.state()
    eq(s["results"], "Network")
    app.ok("goto", str(d))
    eq(app.state()["results"], "", "network view closed")


@test("results: show replaces the folder, goto closes them")
def _():
    d = make("res", {"a.txt": "", "sub/": ""})
    (d / "sub" / "b.txt").write_text("")
    app.ok("goto", str(d))
    app.ok("show", "My results", stdin=f"{d}/a.txt\n{d}/sub/b.txt\n")
    s = app.state()
    eq(s["results"], "My results")
    app.ok("goto", str(d))
    eq(app.state()["results"], "", "results closed")


@test("search: the filter is in the state")
def _():
    d = make("flt", {"alpha.txt": "", "beta.txt": ""})
    app.ok("goto", str(d))
    eq(app.ok("action", "search", "alp")["filter"], "alp")
    eq(app.state()["filter"], "alp")


def main():
    global app
    app = App()
    failed = 0
    for i, (name, fn) in enumerate(results, 1):
        try:
            fn()
            print(f"ok {i} - {name}")
        except Skip as e:
            print(f"ok {i} - {name} # SKIP {e}")
        except Exception as e:  # noqa: BLE001 - any failure is a test result, not an exception of the bench
            failed += 1
            print(f"not ok {i} - {name}")
            print("#   " + str(e).replace("\n", "\n#   "))
        sys.stdout.flush()
        # A window that dies makes the following cases pointless
        if app.proc.poll() is not None:
            print(f"not ok - fili-file stopped (code {app.proc.returncode}); log:")
            print("#   " + (T / "app.log").read_text()[-1500:].replace("\n", "\n#   "))
            failed += 1
            break
        # Put the window back to a neutral state for the next case
        try:
            if app.state()["dialog"]:
                app.ctl("dialog", "cancel")
            app.ctl("hidden", "off")
            app.ctl("goto", str(DATA))
        except Exception:  # noqa: BLE001
            pass
    app.proc.terminate()
    print(f"# done: {failed} failure(s) of {len(results)}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
