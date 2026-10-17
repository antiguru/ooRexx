# An invalid character in a later clause is reported before an earlier clause's parse error

Found 2026-10-17 by the Phase 9c final review (`.superpowers/sdd/2026-10-15-phase-9c/final-review.md`,
finding 3). The program `say 1` / `say (` / ``say ` `` answers 35.1 on line 2 on the oracle, which
scans clause by clause, and 13.1 on line 3 here. The cause is that `invalid_character`
(`rust/crates/rexx-parse/src/scanner.rs:961`) is reached by a scan running ahead of the parse. The
same selection happens inside `INTERPRET` strings: `"trace -;" || "0a"x`, `say (;`, `x = ;` and
`say 1;`, each followed by `'0a'x`. The oracle answers 35.1 and the crate 13.1 on the line feed.
The pre-9c binary `ooRexx-p9-pin/rexx-run-9c-t2-cdbbc1568` makes the same choice, so 9c did not
introduce it. The fix is to defer the scanner's invalid-character error until the parse reaches
that clause, and add the shapes as `parse-errors.tsv` rows.
