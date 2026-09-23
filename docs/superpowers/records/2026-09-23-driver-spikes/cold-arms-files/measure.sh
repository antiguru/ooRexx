#!/bin/bash
# Two interleaved callgrind rounds per binary per axis; the second round runs
# the binaries in reverse order. Each run from a fresh empty directory.
S=/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/spikes/cold-arms
R=/home/moritz/dev/repos/ooRexx-rust-rewrite/.claude/worktrees/agent-a179fbec2e954af52/rust
M=$S/m
mkdir -p "$M"
sha256sum "$S"/bin/*.text > "$M/text-hashes.txt"
for round in 1 2; do
    if [ "$round" = 1 ]; then bins="base full half fullni"; else bins="fullni half full base"; fi
    for ax in rexxcps varlookup arith emptyloop dispatch compound; do
        if [ "$ax" = rexxcps ]; then f=$R/bench-rexxcps/rexxcps.rex; else f=$R/bench-programs/$ax.rex; fi
        for b in $bins; do
            d=$M/run.$round.$ax.$b
            mkdir "$d" || exit 1
            ( cd "$d" && objcopy -O binary --only-section=.text "$S/bin/$b-rexx-run" "$d/text" \
                && sha256sum "$d/text" > "$d/text.sha" && rm "$d/text" \
                && valgrind --tool=callgrind --compress-strings=no \
                    --callgrind-out-file="$M/cg.$round.$ax.$b" "$S/bin/$b-rexx-run" "$f" \
                    >"$M/out.$round.$ax.$b" 2>"$M/err.$round.$ax.$b" ) &
        done
    done
    wait
done
for f in "$M"/cg.*; do
    n=${f##*/cg.}
    printf '%s\t%s\n' "$n" "$(python3 "$S/sumobj.py" "$f")"
done > "$M/results.tsv"
echo finished
