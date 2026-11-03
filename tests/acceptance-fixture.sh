#!/bin/sh
# Set of files for the hand-run acceptance checklist of Casier (ACCEPTANCE.md): /tmp/casier-acceptance (or $1).
# Hard names, a big folder, links, archives (one protected), image, PDF, read-only folder.
# Rebuilt from scratch at every run. Usage: tests/acceptance-fixture.sh [FOLDER]
set -eu
D="${1:-/tmp/casier-acceptance}"
case "$D" in /tmp/*) ;; *) echo "folder outside /tmp refused: $D" >&2; exit 2 ;; esac
# The read-only folder of a previous run prevents its own removal
[ -d "$D" ] && chmod -R u+w "$D"
rm -rf "$D"
mkdir -p "$D/names" "$D/big" "$D/links" "$D/archives" "$D/media" "$D/read-only" "$D/empty" "$D/destination" "$D/conflict/source" "$D/conflict/target"

# Hard names: spaces, accents, emoji, very long, leading dot, apostrophes
for n in "with spaces.txt" "été à la plage.txt" "emoji 🎉.txt" "it's.txt" ".hidden.txt" "UPPERCASE.TXT" "no extension" "two.dots.in.name.tar.txt"; do
    printf 'content of %s\n' "$n" > "$D/names/$n"
done
long="$(printf 'very-long-name-%.0s' 1 2 3 4 5 6 7 8 9 10 11 12 13 14)"
printf 'long\n' > "$D/names/$long.txt"
mkdir -p "$D/names/folder with spaces" "$D/names/empty folder"
printf 'x\n' > "$D/names/folder with spaces/inside.txt"

# Big folder (scrolling, sorting, selection)
i=0; while [ "$i" -lt 5000 ]; do : > "$D/big/file-$(printf %05d "$i").txt"; i=$((i + 1)); done

# Links: valid, broken, to a folder
printf 'target\n' > "$D/links/target.txt"
ln -s target.txt "$D/links/valid-link"
ln -s missing.txt "$D/links/broken-link"
ln -s ../names "$D/links/folder-link"

# Archives: plain and protected (password: secret)
mkdir -p "$D/archives/src" && printf 'in the archive\n' > "$D/archives/src/readme.txt"
(cd "$D/archives/src" && 7z a -bd -y ../plain.zip readme.txt >/dev/null && 7z a -bd -y -psecret ../protected.zip readme.txt >/dev/null && 7z a -bd -y ../plain.7z readme.txt >/dev/null) 2>/dev/null || echo "7z missing: archives not created" >&2

# Media: minimal PNG and PDF
python3 -I - "$D/media" <<'PY'
import struct, sys, zlib
d = sys.argv[1]
def chunk(t, b): return struct.pack(">I", len(b)) + t + b + struct.pack(">I", zlib.crc32(t + b) & 0xffffffff)
w = h = 64
raw = b"".join(b"\x00" + b"".join(bytes((x * 4 % 256, y * 4 % 256, 160)) for x in range(w)) for y in range(h))
open(f"{d}/image.png", "wb").write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))
objs = [b"<</Type/Catalog/Pages 2 0 R>>", b"<</Type/Pages/Kids[3 0 R]/Count 1>>",
        b"<</Type/Page/Parent 2 0 R/MediaBox[0 0 200 100]/Contents 4 0 R/Resources<</Font<</F1 5 0 R>>>>>>",
        b"<</Length 44>>\nstream\nBT /F1 18 Tf 20 50 Td (Casier acceptance) Tj ET\nendstream", b"<</Type/Font/Subtype/Type1/BaseFont/Helvetica>>"]
out = b"%PDF-1.4\n"; offs = []
for i, o in enumerate(objs, 1):
    offs.append(len(out)); out += f"{i} 0 obj\n".encode() + o + b"\nendobj\n"
x = len(out); out += f"xref\n0 {len(objs)+1}\n0000000000 65535 f \n".encode() + b"".join(f"{o:010d} 00000 n \n".encode() for o in offs)
out += f"trailer\n<</Size {len(objs)+1}/Root 1 0 R>>\nstartxref\n{x}\n%%EOF\n".encode()
open(f"{d}/document.pdf", "wb").write(out)
PY
printf 'Text for the preview.\nSecond line.\n' > "$D/media/notes.txt"

# Read-only folder ("permission denied" error when creating an item)
printf 'protected\n' > "$D/read-only/file.txt"; chmod 555 "$D/read-only"

# Name conflict: same name, different content
printf 'SOURCE (new)\n' > "$D/conflict/source/report.txt"; printf 'TARGET (old)\n' > "$D/conflict/target/report.txt"
printf 'other\n' > "$D/conflict/source/other.txt"
echo "$D"
