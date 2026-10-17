# Phase 9c Task 4 review (d5ea1577b..fb69dafb0)

Reviewer: opus. Read the brief, global constraints, the ledger, the report and the review package once.
Probes from fresh dirs under the session scratchpad `9c-r4/`; oracle under `ulimit -v 1048576`, crate
under `memcap 2G`, tests under `memcap 8G -j 4 CARGO_INCREMENTAL=0`, own target dirs. Mutations in a
`git worktree add --detach` copy, each reversed by an exact edit with `git diff --stat` empty after.

## Spec compliance

Issues found: one, Important I1 below (a `NoProgram` record the oracle contradicts). Everything else
in Steps 1 to 5 is present and checked by running.

Cannot verify from the diff: whether a numbered Deviation licenses the label-colon rows. The brief
says the exemption "names the Deviation". `label_colon_deviation`'s doc
(`rust/crates/rexx-parse/tests/errors.rs:471`) says "the recorded label-colon deviation" and names
none, and grepping `phase-4-exclusions.txt`, the 9c plan and the Phase 9 spec found no Deviation
covering 18.x against 35.1. The controller should say which Deviation it is, or whether one exists (M1).

## Strengths

* The conversion is complete. `grep -rn 'ParseError::new\|ParseErrorFields {\|ParseError(Box' --include=*.rs`
  over `rust/` finds only the struct, the two `Deref` impls and the literal in `raise`
  (`token.rs:32-76`). No code changes `.code`/`.sub`/`.inserts` after construction. `PENDING`
  and `missing_then_sub` are gone (grep finds neither).
* The insert builders are cold and out of line, and the per-iteration `let bad = self.error(..)`
  allocations in the directive loops are gone.
* `NotSigned` gets the oracle's two different tokens right: 19.916 names the sign
  (`DirectiveParser.cpp:1891`/`:1898`/`:1905`), 19.923 the signed value at `:2249`. Both match the
  oracle in the witness.
* The coverage test reads the C++ tree for `oracle_site` and looks up the symbolic name in
  `RexxErrorCodes.h`, so a stale citation fails the test.
* The mutation record is thorough, and both of my own block.rs mutants were caught (below).

## Checks run

1. **Old API gone.** Greps as above: nothing remains. Both exemptions are still checked to differ:
   `the_label_colon_rule_covers_exactly_the_two_corpus_rows_it_was_written_for` asserts
   `assert_ne!` against the oracle message for both rows and pins the two programs.
   `the_do_subkey_rows_differ_from_the_oracle_as_deviation_38_says` (`errors.rs:575`) is unchanged
   and also uses `assert_ne!`. `cargo test -p rexx-parse --test errors`: 35 passed, exit 0.
2. **Coverage test.** It went red with one row removed. In the worktree I deleted the
   `signal off junk` 25.4 row (not the report's `call off junk`) and got exit 101:
   `named by the parser, raised by no corpus row, and not recorded: 25.4 (.../instruction.rs)`.
   Restored, `git diff --stat` empty. All four `NoProgram` sites resolve, because
   `every_unreached_reason_still_holds` passes and each cited line holds the name
   (`InstructionParser.cpp:453` `Error_Interpretation_switch`, `:1650`
   `Error_Invalid_do_with_no_control`, `:3421` `Error_Variable_expected_PARSE`,
   `LanguageParser.cpp:3882` `Error_Unexpected_comma_comma`). Every probe went through oracle
   `rexxc -s`. 27.903, 37.1 and 89.2 never raised their pair (`do with` and `do with over x` are
   rc 0, and `parse arg .a` is 31.3). I also tried twelve more comma shapes for 37.1 on the oracle,
   and none reaches it. **For 49.2, two probes do raise the pair on the oracle** (I1):
   `do i over x for 1 to 2` raised 49.2 "unmatched DO OVER loop subkey 29", and
   `do with index i over x for 1 by 2` raised 49.2 "unmatched DO WITH conditional 0", both rc 207.
3. **ARITY_OVERRIDES.** I printed every cited `DirectiveParser.cpp` line with `sed -n`. Each is
   `syntaxError(<name>, token)`, or `parseClassReference(<name>)` reaching `:293`
   `syntaxError(error, token)`. Every pair maps to the cited name in `RexxErrorCodes.h`. The sites
   the crate raises with no insert do pass none on the oracle: `:340` 19.901, `:323` 20.921,
   `:2820` 20.920, `:2227` 19.924, `:2829` 99.944, `:82`/`:89`/`:142`. The witness
   `parse_error_directive_insert.rex` is identical to the oracle on stdout, stderr and rc (rc 0).
   I also wrote a second probe, 40 more `.Package~new` shapes that print code, position,
   `ADDITIONAL` and message. It covers 26.5/26.6 inserts, 24.1, 25.927/20.929 for several options,
   99.917 with an empty string, the second EXTERNAL, 36.901 in `::CONSTANT`, 21.913, 19.925,
   20.919, 25.928 with a number, 99.925, 25.90x, 25.11, 25.924, 25.935, `&1` in names, and
   two-line sources for 99.933/938/934/939. It is identical to the oracle on all three descriptors.
4. **INTERPRET line.** The witness `parse_error_interpret_line.rex` is identical to the oracle. I
   ran seven probes on the base binary (`rexx-run-9c-t4-d5ea1577b`), the head binary
   (`-16e89ac59`) and the oracle. They were five untrapped programs (36.901 at line 5, 13.1 from an
   embedded newline, a runtime 42 in INTERPRET, a nested INTERPRET 14.1, and 10.2), one trapped
   program, and a trace program with `trace i` and `trace r` over multi-clause INTERPRET plus a
   trapped 36.901. The trapped program printed `~position`, `SIGL`, `SOURCELINE(pos)`, the
   traceback, `SOURCELINE` inside INTERPRET, and `SIGL` inside INTERPRET. Results:
   * The main `Error N running ... line L:` line, `~position`, `SOURCELINE`, `SIGL`, the traceback
     and every trace line are the same at base and head. The only base-to-head change is the
     line number inside the sub-message.
   * Head is identical to the oracle on every probe. Stderr differs only by the per-side directory
     name, and is identical after normalising the path. Base differed from the oracle only in that
     line insert (and showed `&1` placeholders for 14.1 and 10.2).
   * Interactive debug input (`trace ?r`, stdin `y = (1` and `do; nop`) also goes through
     `run_fragment` and is identical to the oracle (`... on line 3`).
   * Other callers: `ProgramSource::interpret` has one caller (`run/interpret.rs:29`), and both
     `run_fragment` call sites (`run.rs:810`, `run/interpret.rs:408`) reach it. `parse_interpret`
     still builds a line-1 source, but no non-test caller uses it.
5. **Review Focus rows.** `message_mismatches` compares `Vec<u8>` against the unescaped bytes, and
   all four Focus rows match. A lossy `\x80` would render `EF BF BD` and fail. `derive.sh` at head
   ran in 17 s with exit 0, and its output is `cmp`-identical to the committed TSV.
   `incorrectCharacters` is out of `suite-9d-set.txt` (the header names it at `:17`).
   `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/source.file/incorrectCharacters ... --test suite l3_core_suite`
   exited 0, and `tests.tsv` shows `passed`/`passed`.
6. **Implementer concern 1.** I confirmed each statement is false:
   * `:2292` KNOWN GAPS row: crate and oracle both now print `found "THEN"` for
     `interpret "do forever then"` (trapped message identical, `ADDITIONAL` 1 item, untrapped
     stderr identical apart from the path). Its title and first sentences are false, and its
     "Ruling R3 keeps the 2026-07-28 decision" is superseded by R6. The rest of the row (rc, code,
     position, traceback agree) is still true, so the fix round deletes the row and keeps that
     measurement only if it is wanted elsewhere. No test pins KNOWN GAPS titles (grep of `rust/crates`).
   * `:2485` "Also still unsubstituted" paragraph: false, for the same reason.
   * The 2026-08-20 note: "This crate exits NOT_IMPLEMENTED_EXIT and prints one line" (`:5263`)
     and the bullet "The exit code is deliberate too" (`:5273`) are both false. The crate answers
     `256 - major` with the oracle's three-line report (u1: rc 220, stderr identical). With both
     gone, "Both halves are already decided" no longer has two halves.
   * Also false and not R6's: `:2463-2483`, "We emit the second line only ... ParseError carries
     the clause's START byte with no end". The fragment echo `1 *-* do forever then` is now
     emitted and matches the oracle. The fix round can delete it in the same edit (M3).
7. **Perf.** I recomputed the report's figures from `task-4-callgrind-summary.tsv` (medians of
   three): startup -0.0225% / -0.0176%, interpret -0.5470% / +0.0197%, parse -0.0015% / -0.0006%.
   All match. Column 4 is total minus libc minus the third column, which is the libc-subtracted
   figure. The interpret libc drop is not fewer allocations. In `interpret.{base,head}.r2.cg`,
   `malloc` was called 6,951,203 vs 6,950,833 times, `free` 7,011,260 vs 7,010,890 times, and
   `realloc`, `calloc` and `__rust_alloc` are equal. The cost per call fell instead
   (`callgrind_annotate`): `__libc_malloc2` -6.0M, `_int_free_merge_chunk` -4.3M, `_int_malloc`
   -4.1M, `_int_free_create_chunk` -2.6M, `unlink_chunk` -2.0M, `memcpy` +2.0M. That points to
   heap state or size classes, not the parser's work (`ProgramSource` grew by a `usize` in
   `16e89ac59`, and is one candidate). It is the known glibc spread: the libc-subtracted
   +0.0197% is the figure to read, and the whole-process -0.55% should not be credited to the task.
8. **Prose and tests.** No em-dash and no ` -- ` in added lines. No set sizes. My own mutants:
   (a) `then_expected` naming `line_of(byte)` instead of the opener, and (b) the at-end 14.3 arm
   naming `frame.index` instead of `frame.parent`. Built together, they failed
   `every_secondary_message_matches_the_oracle` with rows 423 (18.1), 314 (18.2) and 1019 (14.3).
   Reversed, diff empty. The refusal tables, which the report skipped: `closed_phases`, `owners`,
   `phase_ten_reasons`, `refusal_dispositions`, `refusal_sites` and `representation_refusals` at
   head all passed, exit 0.

## Issues

### Critical

None.

### Important

**I1. `UNREACHED_BY_FILE`'s 49.2 record is false on the oracle**
(`rust/crates/rexx-parse/tests/errors.rs`, the `(49, 2)` `NoProgram` entry, probes
`b"do i over x for 1 to 2\nend\n"` and `b"do with index i over x for 1 by 2\nend\n"`).
`Unreached::NoProgram`'s doc says "No program reaches it". The oracle raises 49.2 for both of those
probes (rc 207, at `InstructionParser.cpp:1491` and `:1700`). They are the Deviation 38 shapes, and
the TSV already holds them as `translation 49.2` rows. The crate names `E49_2` at three sites
(`instruction.rs:825` "keyword", `instruction/loops.rs:225` "loop subkey", `:399` "loop
conditional"). They correspond to the oracle's `:453`, `:1491` and `:1700`, and only `:453` is
unreachable on the oracle. So 49.2 is unraised here because of a ruled deviation, not because no
program reaches it. The record cites the one site that fits and hides the other two. The closed
enum has no variant for "unraised because of a Deviation", so the fix needs a ruling. Either:
(a) add a variant whose check is that `do_subkey_deviation` covers the corpus rows the oracle
answers 49.2 for, or (b) keep `NoProgram` but cite the `:453` keyword switch only, drop the two
deviation shapes from its probes, and replace them with probes aimed at that switch.

### Minor

**M1.** The label-colon exemption does not name a Deviation (`errors.rs:471`), and no numbered
Deviation for 18.x against 35.1 was found. The brief asks the exemption to name one. Either cite
it, or record that it has no number.

**M2.** `the_scan_finds_the_messages_it_was_written_against`'s comment "One from each raising file"
is false. The pairs it checks come from `scanner.rs`, `expr.rs`, `block.rs`, `directive.rs`,
`lib.rs` and one of `instruction.rs`/`expr.rs`. `instruction/{address,loops,parse}.rs` also import
`rexx_inventory::errors`, and none of them has a pair in the list. Delete the comment or add a pair
per file.

**M3.** The stale exclusions prose at `:2292`, `:2485`, `:2463-2483` and the 2026-08-20 note's
NOT_IMPLEMENTED_EXIT sentences is all confirmed false above, ready for the fix round to delete.

**M4.** The report skips the per-task check's refusal tables ("No `Loud` site changed, so the
refusal tables were not run"). The global constraints list them unconditionally. They pass (check 8),
so nothing is open.

**M5.** `Block::then_expected` (`block.rs:378-383`) repeats `line_insert`'s
`message_line(line_of(..)).to_string().into_bytes()` for a byte where `line_insert` takes an index.
It could share one helper, but this is a nit.

## Assessment

**Spec compliance:** compliant apart from I1. One ⚠️ remains on the label-colon Deviation's name.

**Task quality:** Needs fixes.

**Reasoning:** The conversion, the override table and the INTERPRET line fix are correct, and every
shape I ran is byte-identical to the oracle on all three descriptors. Base-to-head moved nothing but
the message's line insert. The one Important is a record in the new coverage test that the oracle
contradicts: 49.2 is reached on the oracle, and only the Deviation keeps it out of the crate.
