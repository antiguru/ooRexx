# Phase 9c Task 4 re-review (fb69dafb0..15b29ae3a)

Verdict: approved, no open findings.

1. I1: all nine remaining 49.2 probes run on the oracle `rexxc -s` from a fresh dir (`ulimit -v 1048576`):
   27.902, 27.1 x3, 8.1, 8.2, 9.2, 9.1, 10.1. None is 49.2. `errors` (35 tests) passes, so the crate parser
   raises none either. `InstructionParser.cpp:453` is the `Error_Interpretation_switch` default of the keyword
   switch; `then`, `else`, `otherwise`, `when`, `end` are cases of that same switch, so the probes reach it
   without taking the default, which only a corrupt keyword value takes. The two Deviation 38 shapes are gone
   from the record and stay under `do_subkey_deviation`.
2. M3: ran `interpret "do forever then"` and `x = (1` on the oracle and the crate (release build of the head
   tree). Both print the same echo lines, `found "THEN"` / the 36.901 text with inserts, and rc 229 / 220. So
   the deleted placeholder row, the "second line only / no top-level echo" paragraph, and the "exits
   NOT_IMPLEMENTED_EXIT and prints one line" sentences were false. The surviving text of the 2026-08-20 note
   (the oracle's exit codes, the gating by `errors.rs`) is untouched and true. `coverage` (21), `loud` (9),
   `owners` (6) pass.
3. M1: `2026-07-28-phase-3-parser.md:1169` is the "A named deviation" paragraph (`if 1 = 1` / `then: nop`, 35.1
   oracle vs 18.1 here) and `:2557` lists it under "Recorded exceptions". The doc comment says it has no number.
4. M2 comment deleted; M5 `line_insert(byte)` shared by `error_line`, `then_expected`, `end_mismatch`. Added
   lines contain no em-dash and no ` -- `, no set sizes. `cargo fmt --all --check` exit 0.
5. Mutation in a detached worktree: deleting the `signal off junk` row from `parse-errors.tsv` turns
   `every_message_the_parser_names_is_raised_by_a_corpus_row_or_recorded_as_unreached` red ("named by the
   parser, raised by no corpus row, and not recorded"). Restored, `git diff --stat` empty.

Minor, no action: the label-colon doc comment and the in-test comment say the same thing twice.
