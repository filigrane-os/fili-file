#!/bin/bash
# Archiveur, end to end: pack, unpack, list, refusals and the About card, in a throwaway folder. Archiveur is the
# fili-file binary run as `fili-archive` (the symlink made here), exactly like the installed aliases.
#   tests/archive.sh        Binary: $FILI_FILE_BIN, else the build of `make` (target/, or $CARGO_TARGET_DIR).
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REAL="${FILI_FILE_BIN:-${CARGO_TARGET_DIR:-$ROOT/target}/release/fili-file}"
[ -x "$REAL" ] || { echo "not ok - fili-file not found ($REAL): run make first" >&2; exit 1; }
for t in tar zip 7z; do command -v "$t" >/dev/null || { echo "tests/run.sh: $t is required" >&2; exit 2; }; done
W="$(mktemp -d)"
cleanup() { find "$W" -mindepth 1 -delete 2>/dev/null; rmdir "$W" 2>/dev/null; }
trap cleanup EXIT
export HOME="$W/home" LANG=C LC_ALL=C FILI_LANG=
mkdir -p "$W/bin"; ln -s "$(cd "$(dirname "$REAL")" && pwd)/$(basename "$REAL")" "$W/bin/fili-archive"; BIN="$W/bin/fili-archive"
mkdir -p "$HOME" "$W/src/sub"
echo one >"$W/src/a.txt"; echo two >"$W/src/sub/b.txt"; echo hidden >"$W/src/.h"
fail=0
ok() { echo "ok - $1"; }
no() { echo "not ok - $1"; fail=1; }
check() { local name="$1"; shift; if "$@" >/dev/null 2>&1; then ok "$name"; else no "$name"; fi; }

cd "$W" || exit 1
# Every write format round-trips, and the content comes back
for f in zip 7z tar.gz tar.xz tar.zst; do
    # with -f the extension is added to the -o name
    "$BIN" pack -q -f "$f" -o "$W/rt" src >/dev/null 2>&1
    check "pack $f creates the archive" test -f "$W/rt.$f"
    mkdir "$W/out-$f"
    "$BIN" unpack -q -o "$W/out-$f" "$W/rt.$f" >/dev/null 2>&1
    check "unpack $f restores the files" test "$(cat "$W/out-$f/src/sub/b.txt" 2>/dev/null)" = two
    check "list $f shows b.txt" bash -c "'$BIN' list '$W/rt.$f' | grep -q b.txt"
done
# Default name, no overwrite
"$BIN" pack -q src/a.txt >/dev/null 2>&1; check "default name: a.zip next to the file" test -f src/a.zip
"$BIN" pack -q src/a.txt >/dev/null 2>&1; check "no overwrite: a 2.zip" test -f "src/a 2.zip"
# Hidden files
"$BIN" pack -q --no-hidden -f tar.gz -o "$W/nh" src >/dev/null 2>&1
check "--no-hidden leaves .h out" bash -c "! tar -tzf '$W/nh.tar.gz' | grep -q '\.h\$'"
"$BIN" pack -q -f tar.gz -o "$W/h" src >/dev/null 2>&1
check "hidden files are kept by default" bash -c "tar -tzf '$W/h.tar.gz' | grep -q '\.h\$'"
# Password
printf 'secret\n' | "$BIN" pack -q --password-stdin -f 7z -o "$W/pw" src >/dev/null 2>&1
check "password: 7z written" test -f "$W/pw.7z"
mkdir "$W/nopw"
"$BIN" unpack -q -o "$W/nopw" "$W/pw.7z" >/dev/null 2>&1; check "password: unpack without it is refused" test $? -ne 0
mkdir "$W/pwout"; printf 'secret\n' | "$BIN" unpack -q --password-stdin -o "$W/pwout" "$W/pw.7z" >/dev/null 2>&1
check "password: unpack with it works" test "$(cat "$W/pwout/src/a.txt" 2>/dev/null)" = one
# A single item at the top is put down as it is, several go into a folder named after the archive
"$BIN" pack -q -o "$W/one.zip" src/a.txt >/dev/null 2>&1; mkdir "$W/o1"; "$BIN" unpack -q -o "$W/o1" "$W/one.zip" >/dev/null 2>&1
check "single item extracted as it is" test -f "$W/o1/a.txt"
(cd src && "$BIN" pack -q -o "$W/two.zip" a.txt sub >/dev/null 2>&1); mkdir "$W/o2"; "$BIN" unpack -q -o "$W/o2" "$W/two.zip" >/dev/null 2>&1
check "several items extracted into a folder" test -f "$W/o2/two/a.txt"
# Refusals and the About card
"$BIN" pack -q -f rar src >/dev/null 2>&1; check "unknown format is refused" test $? -ne 0
"$BIN" pack -q "$W/nope" >/dev/null 2>&1; check "missing input is refused" test $? -ne 0
check "about prints the card" bash -c "'$BIN' about | grep -q 'Also known as'"
check "about lists both commands" bash -c "'$BIN' about | grep -q 'archiveur, fili-archive'"
check "about is translated" bash -c "FILI_LANG=fr '$BIN' about | grep -q 'Aussi connu sous le nom de'"
check "fili-file archive does the same" bash -c "'$REAL' archive about | grep -q 'Also known as'"
check "--version" bash -c "'$BIN' --version | grep -q '^fili-archive '"
exit $fail
