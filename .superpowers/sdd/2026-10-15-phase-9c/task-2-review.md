# Phase 9c Task 2 review (spec + quality)

Range `cdbbc1568..aade11443`. The plan-only commits are controller rulings and were not reviewed.
Every check below ran in a detached worktree of `aade11443` (and one of `cdbbc1568` for the perf
control), built in `scratchpad/9c-r2/target` and `target-base`.

### Spec Compliance

- ✅ Spec compliant, with the ruled boundary (`expr.rs:344` `missing_expression` stays on
  `ParseError::new` for Task 3 Step 1b).
  - Step 1: `Msg<N>` with private fields and `const fn code` (`rexx-inventory/build.rs:190-199`).
    There is one `E{major}_{sub}` per row (`build.rs:201-210`).
  - Step 2: `raise`, `inserts()`, a single `Option<Box<..>>` (`token.rs:50`, `:67-81`),
    `message() -> Vec<u8>` with no fallback (`error.rs:29-39`), and `Display` decoding lossily
    (`error.rs:49-57`). The `message()` asserts moved to byte literals
    (`tests/errors.rs` 47.1 and 99.908 tests, `error/tests.rs`). Task 1's comparison dropped
    `into_bytes` (`tests/errors.rs` `message_mismatches`). The size is pinned at 24
    (`error/tests.rs` `a_parse_error_is_three_words`). The `u32` two-words sentence is gone
    (`token.rs:42-43`).
  - Step 3: `From<&ParseError>` passes `inserts().to_vec()` (`rexx-exec/src/error.rs:1984`),
    and a test covers it (`error.rs:2420`).
  - Step 4: `grep -n "ParseError::new\|\.error(" scanner.rs expr.rs` finds only
    `expr.rs:345`, the ruled site. The run-time choices pick between same-arity constants:
    `scanner.rs:820` (E6_2/E6_3, `Msg<0>`), `expr.rs:307` (E31_2/E31_3, `Msg<1>`) and `expr.rs:370`
    (E36_901/E36_902, `Msg<2>`).
  - Step 5: probes were run on both sides and recorded (`task-2-probes/results.txt`).
- ⚠️ Cannot verify from the diff: nothing.

### Checks run

1. **Arity over the whole catalogue.** A scratch script parsed `rexxmsg.xml` independently: the
   highest `<Sub position=..>` per `Message`/`SubMessage`. It compared that with every
   `pub const E.._..: Msg<N>` line in the generated
   `target/release/build/rexx-inventory-*/out/errors.rs`. The output was
   `xml rows 704 constants 704`, `missing constants []`, `extra constants []` and
   `arity mismatches []`. No catalogue text has a `&k` with k above 4
   (`grep -o '<Sub position="[0-9]*"' rexxmsg.xml | sort | uniq -c`), so `highest_insert`
   reading one digit is enough.
2. **Oracle sites, read.** The sites checked match the Rust inserts.
   `Scanner.cpp:209-212` reads `startLine` after `stepPosition(2)`, as `scanner.rs:376` does.
   For 13.1 at `Scanner.cpp:1171-1207`, `%c` on the invalid path and `%s` over `bytesCount`
   bytes on the valid path both come out empty for NUL, matching `scanner.rs:610-622`. The
   15.1 and 15.2 inserts are `new_integer(i)` in the right-to-left loop (`Scanner.cpp:453`,
   `:607`). 99.943 is `endMarker, name` (`DirectiveParser.cpp:2337`). 31.2/31.3 pass `token`
   (`LanguageParser.cpp:894`, `:898`). 36.901/36.902 use `firstToken` through `errorPosition`
   (`:3133`, `:3139`, `:4114`). The missing right operand inserts the operator token (`:2963`).
   20.930 uses `nextReal()` (`:3729`). `displayValue` is `Token.cpp:308-360`.
3. **Oracle sites, run.** 44 new probes (`scratchpad/9c-r2/p1`, `run.sh`), one run per side,
   each from its own fresh directory, comparing stdout, stderr and rc. They cover every alternate
   operator spelling (`><`, `<>`, `/=`, `\==`, `/==`, `>>=`, `\<<`, `&&`, `//`), assignment
   operators (`-=`, `||=`), a surrogate, a code point above U+10FFFF, U+00A0, `¬`, a tab at
   the end of a hex string, mixed-case, dotted and numeric over-long symbols, a comment
   unclosed on line 1, an opener on line 3, `~~b(`, `~'q'(`, and `>` followed by `.abc`,
   `a.b`, `(` and `+`.
   All are identical on all three streams, except two Task 3 sites still on `new`.
   `raise syntax 1 description 'x' 'y'` (25.907) and `address x with input stream ,`
   (35.935) print a literal `&1`. The pin prints the same, so these are not regressions.
4. **`message()` and `Raised`.** I mutated `From<&ParseError>` to pass `Vec::new()`, and
   `a_parse_errors_inserts_render_the_same_secondary_line_through_raised` went red (exit 101).
   With the code restored it passes.
5. **PENDING per row.** Changing the entry `drop 5` to `drop  5` turned both tests red:
   `every_pending_message_row_still_differs_from_the_oracle` (`PENDING entries that now match:
   ["drop  5"]`) and `every_secondary_message_matches_the_oracle` (`row 361: 31.2 ...`). A string
   key cannot be invalidated by inserting a row. A typo'd key fails the converse test.
6. **New rows reach the converted sites.** Each mutant below turned the named row red. All runs
   used `-p rexx-parse --test errors` and exited 101.
   - A, `expr.rs:239` unmatched with fixed inserts: row 1111 only.
   - B, `expr.rs:197`: row 1112 only.
   - C, `unmatched`'s line insert fixed at `1`: row 1114 only. This is the only row with an
     opener off line 1.
   - D, 13.1 width forced to 1: rows 1044 and 1117.
   - E, inserts substituted twice: rows 1120 (99.943 `&2`/`&1`), 1118 and 1116.
   - F, 6.1 line taken at EOF: rows 1036 to 1039 and 1121.
   - G, the subterm 35.1 fallback arm: rows 835, 1066, 1083, 1084 and 1122.
   - H, literal display constant: rows 1107, 1118 and 1119.

   The NUL row 1115 and the clause-end row 1113 were witnessed by the implementer's M12 and M13.
   After the runs, `git diff --stat` in the worktree was empty and the baseline was green again
   (rexx-parse lib 261 passed, `errors` 30 passed).
   One procedural note. Restoring from a `.orig` copy kept the copy's older mtime, and cargo then
   reused the mutant-G library for the PENDING run. The rows that stayed at `"X"` showed this. I
   touched the files, rebuilt (`Compiling rexx-parse` printed) and re-ran the baseline. The
   PENDING result above does not depend on the lib.
7. **derive.sh.** `derive.sh SCRATCH wt/rust/corpus/errors/parse-errors.tsv > derived.tsv`
   exited 0 in 16 s, and `cmp` with the committed TSV reported them identical.
8. **Perf.** `callgrind.sh -B 0.5 -j 4 -p 'startup interpret parse'`, 3 rounds, all spreads at
   most 0.0034%:

   | program | pin to `aade11443` | `cdbbc1568` to `aade11443` (Task 2 alone) |
   |---|---|---|
   | startup | +0.7246% (OVER) | +0.3611% |
   | interpret | +0.1969% | +0.1490% (-libc +0.3041%) |
   | parse | -0.0616% | +0.0262% |

   The per-function self-Ir diff of the `.cg` files from base to Task 2 shows the cause.
   `rexx_parse::scanner::scan` is no longer inlined into `rexx_parse::parse`. Startup gains
   +4.78M in `scan` and loses -4.40M in `parse`, net +0.37M of the +0.43M total. Interpret gains
   +79.76M and loses -75.89M, net +3.87M of +6.30M. `need_variable` becomes an out-of-line
   function (+0.86M on interpret), and `arg_list_inner` adds +0.25M. Every malloc and free
   count is identical, so the success path allocates nothing new.

### Strengths

- The arity is carried by the type, and it is correct for the whole catalogue (check 1).
  `raise` cannot be called with the wrong number of inserts.
- `display_value` follows `RexxToken::displayValue` arm for arm. It produced the oracle's bytes
  for every operator spelling probed, including the assignment forms.
- `unmatched` takes the opener explicitly, so each caller names its own `(` or `[`. Checks 3 and
  6 found the positions and lines right off line 1 and inside continued clauses.
- Re-keying PENDING by row closed a real hole: M11, M12 and M14 were green under the pair key.
  The converse test also catches a mistyped key.
- The probe records are reproducible, and the committed TSV re-derives byte for byte.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

1. **`startup` is already over the 9c close budget, and this task contributes half of it**
   (`scanner.rs:610-623`, `expr.rs:305-308`, `expr.rs:840`; budget at plan `:617`, every
   program within +0.5% of the pin). Startup is +0.7246% against the pin at `aade11443`, and
   Task 2 alone is +0.3611%. The per-function diff (check 8) puts most of it on `scan` losing its
   inlining into `parse`, with `need_variable` outlined after it. Both grew error-construction
   code on paths that do not error. This is a codegen effect, not layout noise: the deltas are
   in the functions this task changed. Tasks 3 and 4 add more raise sites to the same crate.
   Suggested fix: move each insert-building error construction into a `#[cold]` out-of-line
   helper, starting with the 13.1 arm and the `need_variable` error tail. Then re-measure
   startup against `cdbbc1568` before Task 3 piles on more. If the controller defers it, Task
   10 starts already OVER.

#### Minor (Nice to Have)

1. `error/tests.rs` `an_insert_is_not_substituted_again` passes under mutant E (substitution
   applied twice, 16 of 17 `error::` tests passed). `"&1"` substituted for `&1` is a fixed
   point. Only corpus row 1120 (99.943 with `&2`/`&1` swapped) catches re-substitution. Use
   `E99_943` with `[b"&2", b"&1"]` in the unit test.
2. `rexx-inventory/tests/errors.rs` checks five constants. That is what the brief asks for, and
   check 1 found all 704 right, but nothing committed would catch a regression in the rest. A
   generated `const ARITIES: &[((u16, u16), usize)]`, checked against `lookup` text, would
   cover them all.
3. The report's witness-row table says "Each reaches a converted site that no existing row
   reached". That is false for rows 1121 (6.1, already reached by rows 1036 to 1039, mutant F),
   1116 and 1117 (13.1, row 1044, mutant D), 1118 and 1119 (20.930 `:806`, row 1107, mutant H)
   and 1122 (35.1 fallback arm, row 1066, mutant G). These rows witness distinct insert
   behaviours (nesting, NUL, raw byte, markup) at sites that existing rows already reached. The
   sentence should say that.
4. `expr.rs:840` (`arg_list_inner`) and `expr.rs:194` (`parse_paren_expression`) look up
   `previous()` on every call, success included. The cost is part of the +0.25M in check 8.
   Capturing `cursor.position() - 1` and indexing only on the error path costs nothing.

### Assessment

**Task quality:** Approved, conditional on the controller ruling on Important 1 before Task 3.

**Reasoning:** Every insert I probed matches the oracle byte for byte, the arity typing is
complete, and the per-row PENDING gate is live in both directions. The open item is
performance: the raise sites cost `startup` +0.36%, which already puts the phase over its close
budget.
