#!/bin/bash
# Screenshots and an animated GIF of Casier for the README and a website: docs/screenshots/*.png and docs/casier.gif.
#
#   make && scripts/screenshots.sh
#
# Needs a running niri session (screenshots come from niri), wtype, ImageMagick and a built binary. Everything runs on
# a throw-away HOME with neutral files, so nothing personal appears. The Casier window floats on the current workspace
# for about two minutes and keys are typed into it: do not touch the keyboard meanwhile. Every step checks that the
# focused window is Casier and stops otherwise, so no key can land in another application.
#   SCREENSHOT_DIR   where niri writes screenshots (default ~/Screenshots, else ~/Pictures/Screenshots)
#   FILI_FILE_BIN    binary to use (default: target/release or $CARGO_TARGET_DIR)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${FILI_FILE_BIN:-${CARGO_TARGET_DIR:-$ROOT/target}/release/fili-file}"
OUT="$ROOT/docs/screenshots"
SHOTS="${SCREENSHOT_DIR:-$HOME/Screenshots}"
[ -d "$SHOTS" ] || SHOTS="$HOME/Pictures/Screenshots"
for tool in niri wtype magick python3; do command -v "$tool" >/dev/null || { echo "$tool is required" >&2; exit 1; }; done
[ -x "$BIN" ] || { echo "build first: make" >&2; exit 1; }
[ -d "$SHOTS" ] || { echo "no screenshot directory: set SCREENSHOT_DIR" >&2; exit 1; }

H="$(mktemp -d /tmp/casier-shots.XXXXXX)"
HOME_DIR="$H/home"
PID=""
trap '[ -n "$PID" ] && kill "$PID" 2>/dev/null; rm -rf "$H"' EXIT INT TERM
mkdir -p "$OUT" "$HOME_DIR/.config/filigrane" "$HOME_DIR/.local/state/filigrane"

# --- neutral files ----------------------------------------------------------------------------------------------
D="$HOME_DIR/Documents"; P="$HOME_DIR/Pictures"
mkdir -p "$D/Invoices" "$D/Notes" "$D/Travel" "$HOME_DIR/Downloads" "$HOME_DIR/Music" "$HOME_DIR/Videos" "$HOME_DIR/Projects" "$P"
printf 'Quarterly report\n' > "$D/Report-Q3.txt"; printf 'Quarterly report, draft\n' > "$D/Report-Q2.txt"; printf '# Ideas\n' > "$D/Ideas.md"
: > "$D/Budget.ods"; : > "$D/Slides.odp"; printf 'a,b\n1,2\n' > "$D/data.csv"; printf '#!/bin/sh\n' > "$D/backup.sh"; chmod +x "$D/backup.sh"
python3 -I - "$D" "$P" <<'PY'
import colorsys, struct, sys, zlib
docs, pics = sys.argv[1], sys.argv[2]
def chunk(t, b): return struct.pack(">I", len(b)) + t + b + struct.pack(">I", zlib.crc32(t + b) & 0xffffffff)
def png(path, hue, w=240, h=160):
    rows = []
    for y in range(h):
        row = b"\x00"
        for x in range(w):
            r, g, b = colorsys.hsv_to_rgb((hue + x / w * 0.18) % 1, 0.55 + 0.35 * y / h, 0.95 - 0.35 * y / h)
            row += bytes((int(r * 255), int(g * 255), int(b * 255)))
        rows.append(row)
    open(path, "wb").write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(b"".join(rows))) + chunk(b"IEND", b""))
for i, name in enumerate(("Sunrise", "Harbor", "Forest", "Desert", "Glacier", "Orchard", "Canyon", "Meadow")):
    png(f"{pics}/{name}.png", i / 8)
png(f"{docs}/Photo.png", 0.55)
objs = [b"<</Type/Catalog/Pages 2 0 R>>", b"<</Type/Pages/Kids[3 0 R]/Count 1>>",
        b"<</Type/Page/Parent 2 0 R/MediaBox[0 0 300 200]/Contents 4 0 R/Resources<</Font<</F1 5 0 R>>>>>>",
        b"<</Length 60>>\nstream\nBT /F1 28 Tf 40 100 Td (Contract) Tj ET\nendstream", b"<</Type/Font/Subtype/Type1/BaseFont/Helvetica>>"]
out = b"%PDF-1.4\n"; offs = []
for i, o in enumerate(objs, 1):
    offs.append(len(out)); out += f"{i} 0 obj\n".encode() + o + b"\nendobj\n"
x = len(out); out += f"xref\n0 {len(objs)+1}\n0000000000 65535 f \n".encode() + b"".join(f"{o:010d} 00000 n \n".encode() for o in offs)
out += f"trailer\n<</Size {len(objs)+1}/Root 1 0 R>>\nstartxref\n{x}\n%%EOF\n".encode()
open(f"{docs}/Contract.pdf", "wb").write(out)
PY
printf 'icon_theme = "Filigrane-Icons"\nview = "list"\n' > "$HOME_DIR/.config/filigrane/fili-file.toml"
STATE="$HOME_DIR/.local/state/filigrane"; echo dark > "$STATE/theme"

# --- driving ----------------------------------------------------------------------------------------------------
ctl() { "$BIN" ctl --pid "$PID" "$@" >/dev/null; }
focused_pid() { niri msg -j focused-window | python3 -c 'import json,sys; print(json.load(sys.stdin).get("pid"))'; }
need_focus() { [ "$(focused_pid)" = "$PID" ] || { echo "Casier lost the focus: stopping" >&2; exit 1; }; }
type_text() { need_focus; wtype "$1"; }
press() { need_focus; wtype -k "$1"; }
shot() {
    sleep 1.3; need_focus; touch "$H/mark"
    niri msg action screenshot-window --write-to-disk true >/dev/null; sleep 1.3
    local f; f="$(find "$SHOTS" -maxdepth 1 -name '*.png' -newer "$H/mark" | head -1)"
    [ -n "$f" ] || { echo "no screenshot found in $SHOTS" >&2; exit 1; }
    mv "$f" "$OUT/$1.png"; echo "  $1"
}
select_paths() { printf '%s\n' "$@" | "$BIN" ctl --pid "$PID" select >/dev/null; }

env HOME="$HOME_DIR" XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}" FILI_LANG=en "$BIN" "$D" >/dev/null 2>&1 &
PID=$!
for _ in $(seq 1 100); do "$BIN" ctl --pid "$PID" state >/dev/null 2>&1 && break; sleep 0.2; done
sleep 1.5; need_focus
niri msg action toggle-window-floating   # a floating window instead of a tall tile
niri msg action set-window-width 980 >/dev/null 2>&1 || true
niri msg action set-window-height 620 >/dev/null 2>&1 || true
sleep 1

echo "Taking screenshots in $OUT"
ctl goto "$D"; select_paths "$D/Report-Q3.txt"
shot 01-list-dark

ctl goto "$P"; ctl view icons; ctl select --none
shot 02-grid-dark

ctl view list; select_paths "$P/Harbor.png"; ctl preview on
shot 03-preview

ctl preview off; ctl goto "$D"; ctl select --none
ctl action search report
shot 04-search
ctl goto "$HOME_DIR"; ctl goto "$D"   # changing folder clears the search
sleep 0.5

ctl action path; sleep 0.5; need_focus; wtype -M ctrl -k a -m ctrl; type_text "~/Doc"
shot 05-path
press Escape

select_paths "$D/Report-Q3.txt"; ctl action rename
shot 06-rename
press Escape

select_paths "$D/Report-Q3.txt" "$D/Report-Q2.txt" "$D/Ideas.md"; ctl action rename
shot 07-bulk-rename
ctl dialog cancel

select_paths "$D/Report-Q3.txt" "$D/Ideas.md" "$D/data.csv"; ctl action compress
shot 08-compress
ctl dialog cancel

ctl goto "$D"; select_paths "$D/Report-Q3.txt"
echo light > "$STATE/theme"; sleep 1.5
shot 09-list-light
ctl goto "$P"; ctl view icons; ctl select --none
shot 10-grid-light
ctl view list; ctl goto "$D"; select_paths "$D/Report-Q3.txt"

for accent in sauge sable cendre; do
    echo "$accent" > "$STATE/accent"; sleep 1.2
    shot "11-accent-$accent-light"
done
echo dark > "$STATE/theme"; for accent in voile sauge cendre; do
    echo "$accent" > "$STATE/accent"; sleep 1.2
    shot "12-accent-$accent-dark"
done
echo default > "$STATE/accent"; sleep 1

if "$BIN" ctl --pid "$PID" actions 2>/dev/null | grep -q '"nearby","label":"Nearby"[^}]*"enabled":true'; then
    ctl action nearby; sleep 3
    shot 13-nearby
    echo "  (13-nearby shows the devices of your network: look at it before publishing it)"
fi

# --- animated GIF -----------------------------------------------------------------------------------------------
echo "Building docs/casier.gif"
cd "$OUT"
frames=(01-list-dark 02-grid-dark 03-preview 04-search 05-path 06-rename 07-bulk-rename 08-compress 09-list-light 11-accent-sauge-light 12-accent-cendre-dark)
args=()
for f in "${frames[@]}"; do [ -f "$f.png" ] && args+=(-delay 130 "$f.png"); done
magick "${args[@]}" -loop 0 -resize 760x -layers Optimize ../casier.gif
ls -l ../casier.gif | awk '{printf "  casier.gif %.1f MiB\n", $5/1048576}'
