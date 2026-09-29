#!/bin/bash
# Callgrind instruction counts, glibc and ld-linux excluded, for two or more
# interpreter binaries over every benchmark program, interleaved.
#
# usage: callgrind.sh [-r ROUNDS] [-j JOBS] [-o OUTDIR] NAME=BINARY NAME=BINARY...
#   ROUNDS  rounds per binary and program (default 2); round r starts the
#           binary order at binary r, so no binary always runs first
#   JOBS    programs measured at once (default 8); counts do not depend on load
#   OUTDIR  where callgrind files, outputs and summary.tsv go (default: mktemp -d)
# env: REXX_LIB_DIR  directory holding liborxfunction.so for extcall.rex, put
#      on LD_LIBRARY_PATH for every run (default: the oracle's build/lib)
#
# Prints one table: per program, the mean exlibc count per binary, each
# binary's delta against the first, and each binary's spread across rounds.
# Every run must exit 0, and every binary must print the same stdout.
set -u
here=$(cd "$(dirname "$0")" && pwd)
rounds=2 jobs=8 out=
while getopts r:j:o: opt; do
    case $opt in
        r) rounds=$OPTARG ;;
        j) jobs=$OPTARG ;;
        o) out=$OPTARG ;;
        *) exit 2 ;;
    esac
done
shift $((OPTIND - 1))
if [ $# -lt 2 ]; then
    echo "usage: callgrind.sh [-r ROUNDS] [-j JOBS] [-o OUTDIR] NAME=BINARY NAME=BINARY..." >&2
    exit 2
fi
lib=${REXX_LIB_DIR:-/home/moritz/dev/repos/ooRexx/build/lib}
[ -f "$lib/liborxfunction.so" ] || { echo "no liborxfunction.so in $lib; set REXX_LIB_DIR" >&2; exit 2; }
[ -n "$out" ] || out=$(mktemp -d)
mkdir -p "$out"
out=$(cd "$out" && pwd)

# Every program in this directory, plus rexxcps, which lives in its own.
# The list is explicit so a run records what it measured; it is checked
# against the directory so a new program cannot be left out unnoticed.
PROGRAMS="alloc alloc4c arith assign compound decloop decrender dispatch
dispatchclass emptyloop extcall fibcall fibfunc heapshape nop parse sayloop
sendloop startup strings textnum varlookup rexxcps"
on_disk=$(cd "$here" && ls ./*.rex | sed 's|^\./||; s|\.rex$||' | sort)
listed=$(echo $PROGRAMS | tr ' ' '\n' | grep -vx rexxcps | sort)
if [ "$on_disk" != "$listed" ]; then
    echo "PROGRAMS and $here/*.rex disagree:" >&2
    diff <(echo "$listed") <(echo "$on_disk") >&2
    exit 2
fi
prog() {
    if [ "$1" = rexxcps ]; then echo "$here/../bench-rexxcps/rexxcps.rex"; else echo "$here/$1.rex"; fi
}

names=() bins=()
for arg in "$@"; do
    names+=("${arg%%=*}")
    bins+=("$(readlink -f "${arg#*=}")")
    [ -x "${bins[-1]}" ] || { echo "not executable: ${arg#*=}" >&2; exit 2; }
done
n=${#names[@]}
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

status=0
: > "$out/summary.tsv"
for p in $PROGRAMS; do
    for ((r = 1; r <= rounds; r++)); do
        for name in "${names[@]}"; do
            cat "$out/$p.$name.r$r.row" >> "$out/summary.tsv"
            if [ "$(cat "$out/$p.$name.r$r.rc")" != 0 ]; then
                echo "$p $name r$r exited $(cat "$out/$p.$name.r$r.rc")" >&2; status=1
            fi
            if ! cmp -s "$out/$p.$name.r$r.out" "$out/$p.${names[0]}.r1.out"; then
                echo "$p $name r$r stdout differs from ${names[0]} r1" >&2; status=1
            fi
        done
    done
done
python3 - "$out/summary.tsv" "${names[@]}" <<'EOF' | tee "$out/table.txt"
import sys
rows = {}
for line in open(sys.argv[1]):
    b, p, r, summ, libc, ld, ex, rc = line.rstrip("\n").split("\t")
    rows.setdefault((b, p), []).append(int(ex))
names = sys.argv[2:]
progs = list(dict.fromkeys(p for (_, p) in rows))
head = ["program"] + names + [f"{b} d%" for b in names[1:]] + [f"{b} spread%" for b in names]
print("\t".join(head))
for p in progs:
    mean = {b: sum(rows[(b, p)]) / len(rows[(b, p)]) for b in names}
    base = mean[names[0]]
    cells = [p] + [f"{mean[b]:.0f}" for b in names]
    cells += [f"{100 * (mean[b] - base) / base:+.3f}" for b in names[1:]]
    cells += [f"{100 * (max(rows[(b, p)]) - min(rows[(b, p)])) / mean[b]:.4f}" for b in names]
    print("\t".join(cells))
EOF
echo "# outputs in $out" >&2
exit $status
