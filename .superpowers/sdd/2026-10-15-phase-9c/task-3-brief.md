### Task 3: The instruction sites

**Files:** `rust/crates/rexx-parse/src/instruction.rs` (helper `error` at `:239`),
`src/instruction/loops.rs`, `src/instruction/address.rs`, `src/instruction/parse.rs`, `src/expr.rs`
(the `missing` parameter), and the `directive.rs` callers of `parse_expr`.

**Consumes:** `Msg<N>`, `ParseError::raise` from Task 2.

- [ ] **Step 1:** Convert every raise site in these files, with the report table as in Task 2. The
      queued probe `2026-10-10-translation-error-inserts.md` (`trace ?s`, 24.1 found `"S"`) is a
      witness here.
- [ ] **Step 1b** (ruling, Task 2): `expr.rs`'s `parse_expr`, `parse_logical` and
      `parse_case_when_list` take `missing: u16`, the 35.x sub raised when the expression is absent,
      and Task 2 left that raise (`parse_expr`'s `None` arm and `comma_list`'s missing arm) on
      `ParseError::new`. Change the parameter to `missing: Msg<0>` with every caller (in
      `instruction.rs`, `loops.rs`, `address.rs` and `directive.rs`; the `directive.rs` callers
      move here too), and convert those two raises. `instruction.rs:499` passes 35.1, which takes
      one insert: restructure it so the assignment raises 35.1 itself with the `=` or
      assignment-operator token as its insert, as `InstructionParser.cpp:233` and `:250` do.
      The 47.1 test pins `found "&1"` (Task 2): update it when converting `instruction.rs:424`.
- [ ] **Step 2:** Shrink `PENDING`, which since Task 2 is keyed per row, not per (major, sub). Each
      converted site needs a `parse-errors.tsv` row that reaches it (added and re-derived with
      `derive.sh` in this task), shown red under a mutation of the site. Per-task check. Commit.

