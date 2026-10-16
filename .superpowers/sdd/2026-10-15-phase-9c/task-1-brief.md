### Task 1: The message column, its script and its reader

**Files:**
- Modify: `rust/corpus/errors/parse-errors.tsv` (header and every row).
- Create: `rust/corpus/errors/derive.sh`.
- Modify: `rust/crates/rexx-parse/tests/errors.rs` (`Case` at `:33`, `cases()` at `:86`).

**Produces:** a `message` column, placed before `program` (the last field stays the only one that can
hold anything), holding the oracle's secondary line after `Error N.M:  ` with the same escapes as
`program`, or `-` where the oracle printed none. `errors.rs` gains
`const PENDING: &[(u16, u16)]` and the test `every_secondary_message_matches_the_oracle`.

- [ ] **Step 1:** Write `derive.sh`, whose first line is `#!/usr/bin/env bash` (`/bin/sh` here is
      dash, whose `printf` does not decode `\xNN`). For each row it decodes `program`
      (`printf '%b'` covers `\\`, `\t`, `\n`, `\r`, `\xNN`), writes it to a fresh empty directory
      under a scratch root given as `$1`, runs the oracle's `rexxc FILE -s` under the oracle
      wrapper, and reads stderr's `Error N.M:  ` line and the `line L:` of the main line, each
      anchored at line start (`^Error `): a clause echo is indented, a program line is not. It
      writes the new file to stdout, keeping each row's `class`, checking `expect` and `line`
      against the oracle and failing loudly on a disagreement. A row whose fields other than
      `program` are `?` is filled from the oracle (`install` for 98.903 and 90.999, `translation`
      otherwise). Put the command in the TSV header, and drop the header's row and field counts
      ("four tab-separated fields", "Nine rows", "the nine rows"), which Task 1 makes stale.
- [ ] **Step 2:** Run it from a fresh directory and replace the file. `git diff` must show the header
      and one inserted column only. Record the wall time and the run command in the report.
- [ ] **Step 3:** Teach `cases()` the new column, decoded to bytes, and add the test: for every
      `translation` row, the parser's error renders the recorded message byte for byte (today's
      `message()` as `String::into_bytes` until Task 2 makes it bytes), unless its (major, sub) is
      in `PENDING`. Fill `PENDING` from the test's own failure list. Add the converse check:
      every `PENDING` pair still fails, so a stale entry is red. Delete
      `no_message_the_corpus_raises_holds_an_unfilled_substitution` (`errors.rs:436`), whose
      premise R6 reverses.
- [ ] **Step 4:** Per-task check (`cargo test -p rexx-parse --test errors`). Commit.

