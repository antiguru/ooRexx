# bash docs/superpowers/records/2026-10-01-phase-6-s2-s5/task-24-enumerations/run.sh REXX_RUN ORACLE_TREE
# Criterion 8's two enumerations, run from the repository root. Each section
# prints its command, then its output between the markers.
R=docs/superpowers/records/2026-10-01-phase-6-s2-s5/task-24-enumerations
section() { echo "\$ $1"; echo "--- begin"; }
done_() { echo "--- end"; echo; }
echo "HEAD $(git rev-parse HEAD)"
echo
section "/bin/grep -a -rn 'Phase 6' rust/crates --include=*.rs | /bin/grep -av '/tests\.rs\|/tests/' | /bin/grep -av ':[0-9]*:\s*//'"
/bin/grep -a -rn 'Phase 6' rust/crates --include=*.rs | /bin/grep -av '/tests\.rs\|/tests/' | /bin/grep -av ':[0-9]*:\s*//'
done_
section "/bin/grep -a -rn 'Phase 6' rust/crates --include=*.rs | /bin/grep -av '/tests\.rs\|/tests/'"
/bin/grep -a -rn 'Phase 6' rust/crates --include=*.rs | /bin/grep -av '/tests\.rs\|/tests/'
done_
# Split text: a Rust string continued with a backslash drops the newline and
# the next line's indentation, so those are joined before the search.
section "find rust/crates -name '*.rs' | /bin/grep -av '/tests\.rs\$\|/tests/' | sort | while read -r f; do perl -0pe 's/\\\\\\n\\s*//g' \"\$f\" | /bin/grep -aPzq 'Phase\s+6' && echo \"\$f\"; done"
find rust/crates -name '*.rs' | /bin/grep -av '/tests\.rs$\|/tests/' | sort | while read -r f; do perl -0pe 's/\\\n\s*//g' "$f" | /bin/grep -aPzq 'Phase\s+6' && echo "$f"; done
done_
section "/bin/grep -a -rln 'Phase 6' rust/crates --include=*.rs   # liveness: the test files still match"
/bin/grep -a -rln 'Phase 6' rust/crates --include=*.rs
done_
section "/bin/grep -a -c 'Phase 6' rust/corpus/refusal-sites.tsv"
/bin/grep -a -c 'Phase 6' rust/corpus/refusal-sites.tsv
done_
section "python3 $R/methods.py ORACLE_TREE | /bin/grep MISSING"
python3 $R/methods.py "$2" | /bin/grep MISSING
done_
section "python3 $R/methods.py ORACLE_TREE"
python3 $R/methods.py "$2"
done_
section "bash $R/probe.sh REXX_RUN method-probes"
bash $R/probe.sh "$1" method-probes
done_
section "bash $R/probe.sh REXX_RUN control"
bash $R/probe.sh "$1" control
done_
