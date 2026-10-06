# bash docs/superpowers/records/2026-10-01-phase-6-s2-s5/task-24-enumerations/probe.sh REXX_RUN DIR
# Runs each DIR/*.rex with this crate's rexx-run from a fresh empty directory
# and prints each line that answers the generic native-method refusal
# (`method "X" of class "Y" is not implemented`), prefixed by the probe, and
# `not reached` for a probe whose method never answered: no line starting
# `ok` (a probe prints one after its call) and no Error 91.999 (a call that
# answered no result).
# DIR is method-probes (one per Message, EventSemaphore and MutexSemaphore
# method) or control (a program known to answer that refusal).
here=$(cd "$(dirname "$0")" && pwd)
for f in "$here/$2"/*.rex; do
  d=$(mktemp -d)
  cp "$f" "$d/p.rex"
  out=$( cd "$d" && timeout -s KILL 20 "$1" p.rex 2>&1 )
  printf '%s\n' "$out" | grep -a 'of class ".*" is not implemented' | sed "s|^|$(basename "$f"): |"
  printf '%s\n' "$out" | grep -aq '^ok\|^Error 91\.999: ' || echo "$(basename "$f"): not reached"
  rm -f "$d/p.rex"
  rmdir "$d"
done
