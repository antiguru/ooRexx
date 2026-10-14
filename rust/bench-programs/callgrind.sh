#!/bin/bash
# Callgrind instruction counts per program per binary: the whole process, and the same
# with libc.so.6 and ld-linux subtracted.
#
# usage: callgrind.sh [-r ROUNDS] [-j JOBS] [-o OUTDIR] [-p PROGRAMS] [-B PCT] [-T] NAME=BINARY...
#   ROUNDS  rounds (default 3); round r starts at binary r
#   JOBS    programs run at once (default 8)
#   OUTDIR  output directory (default: mktemp -d)
#   PROGRAMS  space-separated subset of PROGRAMS (default: all)
#   PCT     budget: a whole-process delta over PCT percent fails (flagged OVER)
#   -T      print the table for OUTDIR's summary.tsv without running
# env: REXX_LIB_DIR  put on LD_LIBRARY_PATH (default: the oracle's build/lib)
#
# Table: the whole-process median per binary and its delta against the first binary,
# which the budget judges; the libc-and-ld-subtracted median and delta; the
# whole-process spread across rounds.
# Exits 1 if a run exits non-zero, if its stdout differs from the first binary's
# (except on TIMED), if a whole-process spread exceeds 0.01% (flagged SPREAD), or if
# a whole-process delta exceeds PCT.
set -u
here=$(cd "$(dirname "$0")" && pwd)
rounds=3 jobs=8 out= table_only= sel= budget=
while getopts r:j:o:p:B:T opt; do
    case $opt in
        r) rounds=$OPTARG ;;
        j) jobs=$OPTARG ;;
        o) out=$OPTARG ;;
        p) sel=$OPTARG ;;
        B) budget=$OPTARG ;;
        T) table_only=1 ;;
        *) exit 2 ;;
    esac
done
shift $((OPTIND - 1))
if [ $# -lt 1 ]; then
    echo "usage: callgrind.sh [-r ROUNDS] [-j JOBS] [-o OUTDIR] [-p PROGRAMS] [-B PCT] [-T] NAME=BINARY..." >&2
    exit 2
fi
lib=${REXX_LIB_DIR:-/home/moritz/dev/repos/ooRexx/build/lib}
[ -f "$lib/liborxfunction.so" ] || { echo "no liborxfunction.so in $lib; set REXX_LIB_DIR" >&2; exit 2; }
[ -n "$out" ] || out=$(mktemp -d)
mkdir -p "$out"
out=$(cd "$out" && pwd)

# Every *.rex here, rexxcps and pingpong/*.rex; checked against the directories.
PINGPONG="pingguard pingmsg pingsem"
PROGRAMS="alloc alloc4c arith assign compound decloop decrender dirread dispatch
dispatchclass emptyloop extcall fibcall fibfunc hashnew heapshape intcall nop parse pkglocal sayloop
sendloop startup strings textnum varlookup rexxcps $PINGPONG"
on_disk=$(cd "$here" && ls ./*.rex pingpong/*.rex | sed 's|^\./||; s|^pingpong/||; s|\.rex$||' | sort)
listed=$(echo $PROGRAMS | tr ' ' '\n' | grep -vx rexxcps | sort)
if [ "$on_disk" != "$listed" ]; then
    echo "PROGRAMS and $here/*.rex disagree:" >&2
    diff <(echo "$listed") <(echo "$on_disk") >&2
    exit 2
fi
if [ -n "$sel" ]; then
    for p in $sel; do
        [[ " $(echo $PROGRAMS) " == *" $p "* ]] || { echo "not in PROGRAMS: $p" >&2; exit 2; }
    done
    PROGRAMS=$sel
fi
TIMED="heapshape rexxcps"
prog() {
    if [ "$1" = rexxcps ]; then echo "$here/../bench-rexxcps/rexxcps.rex"
    elif [[ " $PINGPONG " == *" $1 "* ]]; then echo "$here/pingpong/$1.rex"
    else echo "$here/$1.rex"; fi
}

names=() bins=()
for arg in "$@"; do
    names+=("${arg%%=*}")
    bins+=("$(readlink -f "${arg#*=}")")
    [ -x "${bins[-1]}" ] || { echo "not executable: ${arg#*=}" >&2; exit 2; }
done
n=${#names[@]}
status=0
if [ -z "$table_only" ]; then
{
    echo "# $(date -u +%FT%TZ) rounds=$rounds REXX_LIB_DIR=$lib"
    echo "# $(valgrind --version)"
    for i in "${!names[@]}"; do echo "# ${names[$i]} ${bins[$i]} $(sha256sum < "${bins[$i]}" | cut -d' ' -f1)"; done
} > "$out/binaries.txt"

one() { # binary-index program round
    local d name=${names[$1]} f="$out/$2.${names[$1]}.r$3"
    d=$(mktemp -d)
    (cd "$d" && LD_LIBRARY_PATH=$lib valgrind --tool=callgrind --compress-strings=no \
        --log-file="$f.vg" --callgrind-out-file="$f.cg" "${bins[$1]}" "$(prog "$2")" \
        > "$f.out" 2> "$f.err"; echo $? > "$f.rc")
    rmdir "$d" 2>/dev/null
    printf '%s\t%s\tr%s\t%s\trc=%s\n' "$name" "$2" "$3" "$(python3 "$here/cgsum.py" "$f.cg")" "$(cat "$f.rc")" > "$f.row"
}
job() { # program round: every binary in turn, starting at binary (round - 1) mod n
    local k
    for ((k = 0; k < n; k++)); do one $(((k + $2 - 1) % n)) "$1" "$2"; done
}
running=0
for ((r = 1; r <= rounds; r++)); do
    for p in $PROGRAMS; do
        job "$p" "$r" &
        running=$((running + 1))
        if [ $running -ge "$jobs" ]; then wait -n; running=$((running - 1)); fi
    done
done
wait

: > "$out/summary.tsv"
for p in $PROGRAMS; do
    for ((r = 1; r <= rounds; r++)); do
        for name in "${names[@]}"; do
            cat "$out/$p.$name.r$r.row" >> "$out/summary.tsv"
            if [ "$(cat "$out/$p.$name.r$r.rc")" != 0 ]; then
                echo "$p $name r$r exited $(cat "$out/$p.$name.r$r.rc")" >&2; status=1
            fi
            if ! [[ " $TIMED " == *" $p "* ]] && ! cmp -s "$out/$p.$name.r$r.out" "$out/$p.${names[0]}.r1.out"; then
                echo "$p $name r$r stdout differs from ${names[0]} r1" >&2; status=1
            fi
        done
    done
done
fi
python3 - "$out/summary.tsv" "$budget" "${names[@]}" > "$out/table.txt" <<'EOF'
import statistics
import sys
whole = {}
less = {}
flagged = False
for line in open(sys.argv[1]):
    b, p, r, summ, libc, ld, ex, rc = line.rstrip("\n").split("\t")
    whole.setdefault((b, p), []).append(int(summ))
    less.setdefault((b, p), []).append(int(ex))
budget = float(sys.argv[2]) if sys.argv[2] else None
names = sys.argv[3:]
progs = list(dict.fromkeys(p for (_, p) in whole))
head = ["program"] + names + [f"{b} d%" for b in names[1:]]
head += [f"{b} -libc" for b in names] + [f"{b} -libc d%" for b in names[1:]]
head += [f"{b} spread%" for b in names]
print("\t".join(head))
for p in progs:
    cells = [p]
    deltas = {}
    for kind, counts in (("whole", whole), ("less", less)):
        mid = {b: statistics.median(counts[(b, p)]) for b in names}
        deltas[kind] = {b: 100 * (mid[b] - mid[names[0]]) / mid[names[0]] for b in names[1:]}
        cells += [f"{mid[b]:.0f}" for b in names]
        cells += [f"{deltas[kind][b]:+.4f}" for b in names[1:]]
    spread = {
        b: 100 * (max(whole[(b, p)]) - min(whole[(b, p)])) / statistics.median(whole[(b, p)])
        for b in names
    }
    cells += [f"{spread[b]:.4f}" for b in names]
    print("\t".join(cells))
    for b in names:
        if spread[b] > 0.01:
            print(f"SPREAD {p} {b} {spread[b]:.4f}%", file=sys.stderr)
            flagged = True
    for b in names[1:]:
        if budget is not None and deltas["whole"][b] > budget:
            print(f"OVER {p} {b} {deltas['whole'][b]:+.4f}% > {budget}%", file=sys.stderr)
            flagged = True
sys.exit(1 if flagged else 0)
EOF
[ $? = 0 ] || status=1
cat "$out/table.txt"
echo "# outputs in $out" >&2
exit $status
