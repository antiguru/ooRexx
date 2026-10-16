# Phase 9c Task 3b report: DO OVER / DO WITH subkey keyword error (Deviation 38)

Status: DONE. Code commit 90b1412ae (base d88044e92).

## 1. Oracle shapes for an unexpected keyword in a DO header

Measured with the oracle from a fresh empty directory
(`scratchpad/9c-t3b/probe1/`). Each program was `x = .array~of(1,2)`, then
the header, then `nop` and `end`:

| header | oracle |
|---|---|
| `do forever to 2` (also `by`, `for`) | rc 229, 27.901 `Incorrect data following FOREVER keyword on the loop; found "TO".` |
| `do while 1 to 2`, `do i over x while 1 to 2`, `do with item v over x while 1 to 2`, `do i = 1 while 1 to 2` | rc 222, runtime 34.3 (the WHILE expression takes in `1 TO 2`) |
| `do until 1 by 2` | rc 222, runtime 34.4 |
| `do 3 to 2` (also `by`, `for`) | rc 230, runtime 26.2 |
| `do i over x to 2`, `do i over x by 2` | rc 0 (TERM_OVER folds it into the expression) |
| `do with item v over x to 2` | rc 159, runtime 97.1 (folded) |
| `do i over x for 1 to 2` | rc 207, 49.2 `unmatched DO OVER loop subkey 29` |
| `do i over x for 1 by 2` | rc 207, 49.2 `... subkey 2` |
| `do with item v over x for 1 to 2` / `by 2` | rc 207, 49.2 `unmatched DO WITH conditional 0` |

27.901 at `InstructionParser.cpp:1869` is the oracle's only keyword error
for an unexpected TO, BY or FOR in a DO header. Its text names FOREVER. None
of the other 27.x minors is generic (`RexxErrorMessages.h:276-281`).

## 2. Chosen error

**25.927, `Error_Invalid_subkeyword_following`: `Unknown keyword following "&1"; found "&2".`**
The inserts are `FOR` and the keyword as the token renders it (`TO` / `BY`).
The oracle raises this error from `DirectiveParser.cpp:1105` and nearby lines
(`::OPTIONS`), with &1 the preceding keyword and &2 the token found.

This choice does not follow 27.901. On a DO OVER or DO WITH line, 27.901
would print "following FOREVER keyword", which is false. I put options (a)
27.901 and (b) 25.927 to team-lead and proceeded with (b), recommended. No
reply had arrived when this report was written. Switching to (a) changes the
one `self.error(...)` call in `for_and_conditional`
(`rust/crates/rexx-parse/src/instruction/loops.rs`), the predicate's pair, and
the expected message in the errors.rs test and the deviation text.

Only a TO or BY that ended the FOR expression reaches the arm. TERM_OVER
stops only at FOR and WHILE, and a second FOR is 27.902, so the first insert
is always `FOR`. The line is the clause's line, the same as the oracle's 49.2
and 27.901 (`self.error` on `clause_byte`). The `with` parameter and
`unmatched_loop_keyword` existed only to rebuild the oracle's enum values.
Both are removed.

This crate, measured on a release `rexx-run`:

| header | this crate |
|---|---|
| `do i over x for 1 to 2` | rc 231, `Error 25.927:  Unknown keyword following "FOR"; found "TO".` |
| `do i over x for 1 by 2` | rc 231, `... found "BY".` |
| `do with item v over x for 1 to 2` | rc 231, `... found "TO".` |
| `do with item v over x for 1 by 2` | rc 231, `... found "BY".` |
| `do forever to 2` | rc 229, 27.901 (unchanged, same as the oracle) |
| `do i over x to 2` / `by 2` | rc 0, same as the oracle |
| `do with item v over x to 2` | rc 159, 97.1, same as the oracle |

## 3. Deviation 38

Added to `docs/superpowers/plans/phase-4-exclusions.txt` after Deviation 37,
with the title as briefed. It records Moritz's 2026-10-16 ruling, cites
`InstructionParser.cpp:1491` and `:1700`, gives the measured shapes (oracle,
then this crate, 1 run a side), says why 27.901 was not used, and names the
test exemption.

## 4. Test exemption

In `rust/crates/rexx-parse/tests/errors.rs`:
- `do_subkey_deviation(ours, oracle)` is true only for `(Some((25, 927)), (49, 2))`.
- `every_program_the_oracle_refuses_to_translate_this_parser_rejects_the_same_way` skips rows the predicate matches, in the same way it skips `label_colon_deviation`. The `checked >= 565` floor still holds.
- `message_mismatches` (which feeds `every_secondary_message_matches_the_oracle`) skips rows the predicate matches.
- `the_do_subkey_rule_fires_for_the_pair_it_was_written_for` checks the predicate matches its pair.
- `the_do_subkey_rule_does_not_fire_for_a_near_miss` covers 49.2/49.2, 25.927/25.927, 27.901/49.2, 25.927/49.1, and None/49.2.
- `the_do_subkey_rows_differ_from_the_oracle_as_deviation_38_says` checks the rows the predicate covers. They must be exactly the three corpus rows. Each must render this crate's 25.927 message, differ from the oracle's recorded message, and report the oracle's line. If the oracle or this crate changes, this test fails, so the exemption cannot go stale silently.

`parse-errors.tsv` is untouched. `bash corpus/errors/derive.sh <scratch>` was
re-run, and its output is byte-identical to the committed TSV (`cmp`).

The unit test `the_loop_grammars_unreachable_switch_is_reachable_after_all`
became `a_to_or_by_after_a_do_over_or_with_for_is_a_keyword_error` in
`instruction/tests.rs`. It covers all four headers and keeps the two
near-neighbour rc-0 shapes.

## 5. Mutation

Mutation: the new arm was put back to
`self.unreachable_switch("DO OVER loop subkey", Some(token))` (49.2), and
`cargo test -p rexx-parse --no-fail-fast` was run. Exit 101, with three red:
`a_to_or_by_after_a_do_over_or_with_for_is_a_keyword_error`,
`every_secondary_message_matches_the_oracle` and
`the_do_subkey_rows_differ_from_the_oracle_as_deviation_38_says`. The file
was then restored from a saved copy.

## 6. Checks (at 90b1412ae, each exit status read unpiped)

- `cargo fmt --all --check`: 0
- `cargo clippy --workspace --all-targets -- -D warnings`: 0
- `memcap 8G cargo test -p rexx-parse`: 0
- `memcap 8G cargo test -p rexx-inventory`: 0
- `memcap 8G cargo test -p rexx-exec --lib error::`: 0 (13 passed)
- `REXX_CORPUS_GATE=1 memcap 8G cargo test --release -p rexx-exec --test corpus`: 0 (29 passed, 1 ignored)
- Refusal tables: no refusal changed, so not run.

## Concerns

- The error choice was decided without the lead's answer (section 2). If
  27.901 is preferred despite its FOREVER text, the change is small.
- 25.927's "Unknown keyword" wording is loose: TO and BY are keywords, just
  not ones a DO OVER or DO WITH header accepts.
