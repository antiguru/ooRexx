#!/bin/bash
# Control for the hoisting finding: the walker built with walk_step #[inline(never)]
# against the measured walker, same programs, two rounds interleaved.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/round3/tw-vs-oracle
M=$S/mc
mkdir "$M" || exit 1
for b in tw noinline; do
    objcopy -O binary --only-section=.text "$S/bin/$b-rexx-run" "$M/$b.text"
    sha256sum "$M/$b.text" >> "$M/text-hashes.txt"
done
for round in 1 2; do
    if [ "$round" = 1 ]; then bins="tw noinline"; else bins="noinline tw"; fi
    for p in ctrl ctrl2 nop assign_var incr if_eq call_r; do
        for b in $bins; do
            d=$M/run.$round.$p.$b
            mkdir "$d" || exit 1
            ( cd "$d" && REXX_ENGINE=tree-walker valgrind --tool=callgrind --compress-strings=no \
                --callgrind-out-file="$M/cg.$round.$p.$b" "$S/bin/$b-rexx-run" "$S/progs/$p.rex" \
                >"$M/out.$round.$p.$b" 2>"$M/err.$round.$p.$b"; echo $? > "$M/rc.$round.$p.$b" ) &
        done
    done
    wait
done
for f in "$M"/cg.*; do
    n=${f##*/cg.}
    printf '%s\t%s\t%s\n' "$n" "$(cat "$M/rc.$n")" "$(python3 "$S/sumobj.py" "$f")"
done > "$M/results.tsv"
echo finished > "$M/status"
