### Task 5: Parse errors as SYNTAX, with the traceback line

**Files:** `rexx-parse/src/token.rs` (`ParseError` `:28`: the end of the last scanned token),
the scanner, `rexx-exec/src/lib.rs` (`:3196-3232` main program; `required_source` `:632`;
`:2208` external call), `install.rs` (`:657` `::REQUIRES`, `:953` `Package~new`, `:1778` `newFile`),
`run/interpret.rs` (`run_fragment` `:28-37`), `error.rs` (`From<&ParseError> for Raised`
`:1836-1840`), `docs/superpowers/plans/phase-4-exclusions.txt` (rows `:530` and `:1895-1925`), `tests/concurrency_tests.rs`
lines with `does not parse here`, `corpus/errors/parse-errors.tsv`.

**Interfaces:** `ParseError` gains the clause's end offset; one function turns a `ParseError` plus its
`ProgramSource` into a raised SYNTAX condition whose traceback's first line is the failing clause
(its line number in the failing source, text from the clause's first token to the last scanned, as
`LanguageParser.cpp:866-876`). Inserts stay as today (R3).

- [ ] **Step 1:** Witnesses: scout A's b3 probes (`b_requires_bad`, `b12_requires_trapped`,
      `b_package_new_bad`, `b12_package_new_bad_untrapped`, `b_external_call_bad`,
      `b12_method_newfile_bad_abs`, `b12_routine_newfile_bad_abs`, `b12_routine_newfile_bad_untrapped`),
      c8 (`c_parse_error_main`), scout B's `interp`, `interpreply` and `i2` 1-8 rewritten to print
      code, position and the traceback (not the message, which keeps `&1` under R3). Multi-file probes
      use `.d` directories. Compare rc, the traceback lines and `condition('O')~code`.
- [ ] **Step 2:** Implement the function and call it from the main program path (rc as the oracle's,
      the standard error report), the four loaded-source paths and `run_fragment`. Delete the
      `required_source` and `library_source`-style refusals that become reachable answers; b17
      (embedded `.orx`, sha-pinned) stays a guard for Task 7. Rewrite `docs/superpowers/plans/phase-4-exclusions.txt:530` and
      the KNOWN GAP at `:1895-1925`; update the whole-group expectation lines.
- [ ] **Step 3:** Witnesses agree; `parse-errors.tsv` still passes through `rexxc`; the per-task check;
      perf on `startup` and `parse` under `## Task 5`. Commit.

