#!/bin/sh
# Memory of a file manager with a folder open, so that Casier can be compared with others on the same machine.
#
#   scripts/measure.sh COMMAND [ARGS...]      e.g. scripts/measure.sh fili-file ~/Documents
#                                                  scripts/measure.sh thunar ~/Documents
#
# Starts the command, waits MEASURE_WAIT seconds (default 6) for the window to settle, prints the proportional
# (PSS), resident (RSS) and private memory of the process and its children in MiB, then stops it. PSS is the fair
# number: shared libraries are split between the processes that use them. Close other instances of the program first.
set -eu
[ $# -gt 0 ] || { echo "usage: $0 COMMAND [ARGS...]" >&2; exit 2; }
"$@" >/dev/null 2>&1 &
pid=$!
sleep "${MEASURE_WAIT:-6}"
pids="$pid $(pgrep -P "$pid" 2>/dev/null || true)"
for p in $pids; do cat "/proc/$p/smaps_rollup" 2>/dev/null; done | awk '
    /^Pss:/ {pss += $2} /^Rss:/ {rss += $2} /^Private_(Clean|Dirty):/ {priv += $2}
    END {printf "PSS %.0f MiB   RSS %.0f MiB   private %.0f MiB\n", pss / 1024, rss / 1024, priv / 1024}'
kill "$pid" 2>/dev/null || true
