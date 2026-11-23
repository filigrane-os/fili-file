#!/bin/bash
# Reordering the favorites by dragging them with the mouse, headless: the real qml/Sidebar.qml and the shared kit (Fili.UI,
# found through cargo) with a stand-in for the Rust object. Needs qmltestrunner (Void: qt6-quick-test).
#   tests/sidebar-drag.sh
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RUNNER="$(command -v qmltestrunner || ls /usr/lib/qt6/bin/qmltestrunner /usr/lib64/qt6/bin/qmltestrunner 2>/dev/null | head -1)"
[ -n "$RUNNER" ] || { echo "not ok - qmltestrunner not found (package qt6-quick-test)" >&2; exit 1; }
KIT="$(cd "$ROOT" && cargo metadata --format-version 1 --locked 2>/dev/null | python3 -I -c 'import json,sys; print(next(p["manifest_path"] for p in json.load(sys.stdin)["packages"] if p["name"] == "fili-ui"))')"
[ -n "$KIT" ] || { echo "not ok - fili-ui not found by cargo metadata" >&2; exit 1; }
W="$(mktemp -d)"
trap 'find "$W" -mindepth 1 -delete 2>/dev/null; rmdir "$W" 2>/dev/null' EXIT
cp -r "$ROOT/tests/sidebar-drag/Fili" "$ROOT/tests/sidebar-drag/tst_dnd.qml" "$W/"
cp "$ROOT/qml/Sidebar.qml" "$W/Fili/Files/"
python3 -I - "$ROOT/theme.toml" >"$W/theme.json" <<'PY'
import json, sys, tomllib
t = tomllib.load(open(sys.argv[1], "rb"))["theme"]
base = {k: v for k, v in t.items() if not isinstance(v, dict)}
print(json.dumps({"dark": {**base, **t["dark"]}, "light": {**base, **t["light"]}, "accents": t["accents"]}))
PY
cd "$W" && QML_XHR_ALLOW_FILE_READ=1 QT_QPA_PLATFORM=offscreen "$RUNNER" -import "$W" -import "$(dirname "$KIT")/qml" -input tst_dnd.qml 2>&1 | grep -vE "Unable to assign" | tee "$W/out.txt"
grep -q "^Totals: .* 0 failed" "$W/out.txt"
