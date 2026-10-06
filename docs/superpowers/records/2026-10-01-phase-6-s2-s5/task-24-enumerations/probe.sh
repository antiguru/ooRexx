# bash docs/superpowers/records/2026-10-01-phase-6-s2-s5/task-24-enumerations/probe.sh REXX_RUN DIR
# Runs each DIR/*.rex with this crate's rexx-run from a fresh empty directory
# and prints each line that answers the generic native-method refusal
# (`method "X" of class "Y" is not implemented`), prefixed by the probe.
# DIR is method-probes (one per Message, EventSemaphore and MutexSemaphore
# method) or control (a program known to answer that refusal).
here=$(cd "$(dirname "$0")" && pwd)
for f in "$here/$2"/*.rex; do
  d=$(mktemp -d)
  cp "$f" "$d/p.rex"
  ( cd "$d" && timeout -s KILL 20 "$1" p.rex 2>&1 ) | grep -a 'of class ".*" is not implemented' | sed "s|^|$(basename "$f"): |"
  rm -f "$d/p.rex"
  rmdir "$d"
done
