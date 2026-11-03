#!/bin/sh
# Tests of fili-file (Casier): an isolated instance, without a display (QT_QPA_PLATFORM=offscreen), driven by
# `fili-file ctl` (the channel of the assistant) and checked through `ctl state` and the file system. Nothing touches
# the session: HOME, runtime folder and data are temporary, no Wayland or niri, fake wl-copy/wl-paste (the real
# clipboard of the user stays intact). Texts in English. Exit code 0 if everything passes.
# Usage: tests/run.sh [-v]   Binary: $FILI_FILE_BIN, else the build of `make` (target/, or $CARGO_TARGET_DIR).
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${FILI_FILE_BIN:-${CARGO_TARGET_DIR:-$ROOT/target}/release/fili-file}"
[ -x "$BIN" ] || { echo "not ok - fili-file not found ($BIN): run make first" >&2; exit 1; }
# A binary older than the sources would test something other than the repository
stale="$(find "$ROOT/src" "$ROOT/qml" "$ROOT/i18n" "$ROOT/fonts" -newer "$BIN" -type f 2>/dev/null | head -3)"
if [ -n "$stale" ]; then
        echo "not ok - stale binary, rebuild it with make; newer than $BIN:"
    printf '%s\n' "$stale" | sed 's/^/#   /'
    exit 1
fi
# The translation catalog matches the texts of the code
python3 -I "$ROOT/scripts/check-i18n.py" >/dev/null || { python3 -I "$ROOT/scripts/check-i18n.py"; exit 1; }
# Short path: the Unix socket of the context is limited to 108 characters
T="$(mktemp -d /tmp/fili-file-test.XXXXXX)"
trap 'kill $(cat "$T/pid" 2>/dev/null) 2>/dev/null; rm -rf "$T"' EXIT INT TERM
mkdir -m 700 "$T/run" "$T/bin" "$T/clip"
mkdir -p "$T/home/data"

# Fake wl-copy / wl-paste: the "clipboard" is a folder (one file per MIME type)
cat > "$T/bin/wl-copy" <<'SH'
#!/bin/sh
d="${FAKE_CLIP:?}"; type=text/plain
while [ $# -gt 0 ]; do
    case "$1" in
        --clear) rm -f "$d"/*; exit 0 ;;
        --type) type="$2"; shift 2 ;;
        --) shift; rm -f "$d"/*; printf %s "$*" > "$d/$(echo "$type" | tr / _)"; exit 0 ;;
        *) shift ;;
    esac
done
rm -f "$d"/*; cat > "$d/$(echo "$type" | tr / _)"
SH
cat > "$T/bin/wl-paste" <<'SH'
#!/bin/sh
d="${FAKE_CLIP:?}"; type=
while [ $# -gt 0 ]; do
    case "$1" in
        --list-types) for f in "$d"/*; do [ -e "$f" ] && basename "$f" | tr _ /; done; exit 0 ;;
        --type) type="$2"; shift 2 ;;
        *) shift ;;
    esac
done
f="$d/$(echo "$type" | tr / _)"
[ -e "$f" ] && cat "$f" || exit 1
SH
# Fake quicklook: the real one opens a window (foot, imv...) on the user's screen; this one notes its argument
cat > "$T/bin/quicklook" <<'SH'
#!/bin/sh
printf '%s\n' "$1" >> "${FAKE_CLIP:?}/../quicklook.log"
SH
# Fake default text editor (text/plain): an empty file opens with it, it notes its argument
cat > "$T/bin/fake-editor" <<'SH'
#!/bin/sh
printf '%s\n' "$1" >> "${FAKE_CLIP:?}/../editor.log"
SH
mkdir -p "$T/home/.local/share/applications" "$T/home/.config"
printf '[Desktop Entry]\nType=Application\nName=Fake Editor\nExec=%s/bin/fake-editor %%f\nMimeType=text/plain;\n' "$T" > "$T/home/.local/share/applications/fili-test-editor.desktop"
printf '[Default Applications]\ntext/plain=fili-test-editor.desktop\n' > "$T/home/.config/mimeapps.list"
chmod +x "$T/bin/wl-copy" "$T/bin/wl-paste" "$T/bin/quicklook" "$T/bin/fake-editor"

# Fake mounts: the Trash view gathers the trashes of the mounted volumes, and "Empty" empties them all. The bench only
# knows /dev/shm (trash of a volume other than the test HOME), never the real volumes of the machine.
printf '22 1 0:20 / /dev/shm rw - tmpfs tmpfs rw\n' > "$T/mountinfo"

env -i FILI_MOUNTINFO="$T/mountinfo" PATH="$T/bin:$PATH" HOME="$T/home" XDG_RUNTIME_DIR="$T/run" FAKE_CLIP="$T/clip" QT_QPA_PLATFORM=offscreen \
    FILI_LANG=en LANG=C.UTF-8 FILI_TEST_ROOT="$T" FILI_FILE_BIN="$BIN" FILI_VERBOSE="$([ "${1:-}" = -v ] && echo 1)" \
    timeout 240 python3 -I "$ROOT/tests/scenarios.py"
