#!/bin/bash
# Wall-clock seconds per program per binary, interleaved, median of ROUNDS.
#
# usage: wallclock.sh [-r ROUNDS] [-o OUTDIR] [-x PROGRAMS] [-T] NAME=BINARY...
#   ROUNDS    rounds (default 5); round r starts at arm r
#   OUTDIR    output directory (default: mktemp -d)
#   PROGRAMS  programs also run on the oracle (arm name "oracle")
#   -T        print the table for OUTDIR's wall.tsv without running
# env: REXX_LIB_DIR  put on LD_LIBRARY_PATH (default: the oracle's build/lib)
#      ORACLE_ROOT   the oracle's build directory (default /home/moritz/dev/repos/ooRexx/build)
#      PROGRAMS      the programs to run (default: callgrind.sh's PROGRAMS)
#
# Each binary is copied to OUTDIR/stage/rexx-run before each of its runs; bash's
# `time` times the interpreter process.
# Exits 1 if a run exits non-zero or its stdout differs from the first binary's
# (except on TIMED).
set -u
here=$(cd "$(dirname "$0")" && pwd)
rounds=5 out= with_oracle= table_only=
while getopts r:o:x:T opt; do
    case $opt in
        r) rounds=$OPTARG ;;
        o) out=$OPTARG ;;
        x) with_oracle=$OPTARG ;;
        T) table_only=1 ;;
        *) exit 2 ;;
    esac
done
shift $((OPTIND - 1))
if [ $# -lt 1 ]; then
    echo "usage: wallclock.sh [-r ROUNDS] [-o OUTDIR] [-x PROGRAMS] [-T] NAME=BINARY..." >&2
    exit 2
fi
oracle_root=${ORACLE_ROOT:-/home/moritz/dev/repos/ooRexx/build}
lib=${REXX_LIB_DIR:-$oracle_root/lib}
[ -n "$out" ] || out=$(mktemp -d)
mkdir -p "$out/stage"
out=$(cd "$out" && pwd)
PROGRAMS=${PROGRAMS:-$(sed -n '/^PROGRAMS="/,/"$/p' "$here/callgrind.sh" | tr -d '"' | sed 's/^PROGRAMS=//')}
prog() {
    if [ "$1" = rexxcps ]; then echo "$here/../bench-rexxcps/rexxcps.rex"; else echo "$here/$1.rex"; fi
}
declare -A bin
names=()
for arg in "$@"; do
    names+=("${arg%%=*}")
    bin[${arg%%=*}]=$(readlink -f "${arg#*=}")
done
status=0
if [ -z "$table_only" ]; then
{
    echo "# $(date -u +%FT%TZ) rounds=$rounds REXX_LIB_DIR=$lib load=$(cut -d' ' -f1-3 /proc/loadavg)"
    for name in "${names[@]}"; do echo "# $name ${bin[$name]} $(sha256sum < "${bin[$name]}" | cut -d' ' -f1)"; done
} > "$out/binaries.txt"

TIMED="heapshape rexxcps"
TIMEFORMAT=%R
run() { # arm program round
    local d t f="$out/${2//\//_}.$1.r$3"
    d=$(mktemp -d)
    if [ "$1" = oracle ]; then
        t=$( { cd "$d"; ( ulimit -v 1048576; export LD_LIBRARY_PATH=$lib; time "$oracle_root/bin/rexx" "$(prog "$2")" > "$f.out" 2> "$f.err"; echo $? > "$f.rc" ); } 2>&1 )
    else
        cp "${bin[$1]}" "$out/stage/rexx-run"
        t=$( { cd "$d"; ( export LD_LIBRARY_PATH=$lib; time "$out/stage/rexx-run" "$(prog "$2")" > "$f.out" 2> "$f.err"; echo $? > "$f.rc" ); } 2>&1 )
    fi
    rmdir "$d"
    printf '%s\t%s\tr%s\t%s\n' "$1" "$2" "$3" "$t" >> "$out/wall.tsv"
}
: > "$out/wall.tsv"
for ((r = 1; r <= rounds; r++)); do
    for p in $PROGRAMS; do
        arms=("${names[@]}")
        [[ " $with_oracle " == *" $p "* ]] && arms+=(oracle)
        n=${#arms[@]}
        for ((k = 0; k < n; k++)); do run "${arms[$(((k + r - 1) % n))]}" "$p" "$r"; done
    done
done
echo "# end load=$(cut -d' ' -f1-3 /proc/loadavg)" >> "$out/binaries.txt"
for f in "$out"/*.rc; do
    [ "$(cat "$f")" = 0 ] || { echo "$(basename "$f" .rc) exited $(cat "$f")" >&2; status=1; }
done
for p in $PROGRAMS; do
    [[ " $TIMED " == *" $p "* ]] && continue
    for f in "$out/${p//\//_}".*.out; do
        cmp -s "$f" "$out/${p//\//_}.${names[0]}.r1.out" || { echo "$(basename "$f" .out) stdout differs from ${names[0]} r1" >&2; status=1; }
    done
done
fi
python3 - "$out/wall.tsv" "${names[@]}" oracle <<'EOF' | tee "$out/table.txt"
import statistics
import sys
rows = {}
for line in open(sys.argv[1]):
    a, p, r, t = line.rstrip("\n").split("\t")
    # bash's `time` carries a rounded-up digit as the next character: "1.:00" is 2.000.
    whole, frac = t.split(".")
    secs = int(whole) + sum((ord(c) - 48) / 10 ** (i + 1) for i, c in enumerate(frac))
    rows.setdefault(p, {}).setdefault(a, []).append(secs)
arms = sys.argv[2:]
print("\t".join(["program"] + [f"{a} median" for a in arms] + [f"{a} d%" for a in arms[1:]]))
for p, d in rows.items():
    mid = {a: statistics.median(d[a]) for a in arms if a in d}
    base = mid[arms[0]]
    cells = [p] + [f"{mid[a]:.3f}" if a in mid else "-" for a in arms]
    cells += [f"{100 * (mid[a] - base) / base:+.2f}" if a in mid else "-" for a in arms[1:]]
    print("\t".join(cells))
EOF
echo "# outputs in $out" >&2
exit $status
