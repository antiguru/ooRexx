# Phase 9c Task 3 review (spec + quality)

Range `93de0ca29..b17078946`: code `7436ca94e`, report `b17078946`, and `git diff 7436ca94e b17078946 -- rust`
is empty. Every check ran in a detached worktree of `b17078946` (`scratchpad/9c-r3/wt`), built in
`scratchpad/9c-r3/target`. Oracle runs used the global-constraints wrapper. Each probe ran from the same
fresh directory path on both sides, so the file name in stderr matches.

### Spec Compliance

- ✅ Spec compliant.
  - Step 1: `grep -nE 'ParseError::new|\.error\([0-9]' instruction.rs instruction/*.rs expr.rs` finds
    nothing. The only `ParseError::new` left in these paths is `block.rs:660`, which is Task 4's
    (plan commit `7706d5f9f`).
  - Step 1b: `parse_expr`, `parse_logical`, `Parser::logical` and `comma_list` take `Msg<0>`
    (`expr.rs:396`, `:419`, `:484`, `:500`). `missing_expression` raises via `raise`
    (`expr.rs:460`). `directive.rs:719` is that file's only `parse_expr`/`parse_logical`/
    `parse_case_when_list` caller (`grep -n` over `directive.rs` and `block.rs`). The message
    assignment raises 35.1 naming the `=` or operator token (`instruction.rs:555`). The 47.1 test
    pins `found "X"` (`tests/errors.rs` `an_interpret_label_error_names_the_label`).
  - Step 2: PENDING has 204 entries. I parsed it against the TSV: every entry is a TSV row, and
    every entry that is not a `::` row raises 7.x, 10.x, 14.x, 47.2-47.4, an EOF 18.x, or 35.1 for
    one of the two label-colon Deviation rows. None of `E7_`, `E10_`, `E14_` or `E47_2`-`E47_4` is
    named in `instruction*.rs` or `expr.rs`.
- ⚠️ Cannot verify from the diff: nothing.

### Checks run

1. **Inserts against the oracle, run.** I wrote 61 probes that are not TSV rows
   (`scratchpad/9c-r3/p3`, `p4`, runner `scripts/run.sh`) and ran each once per side. All 61 are
   identical on stdout, stderr and rc. I read each oracle stderr line to confirm the path it
   claims. They cover:
   - the "next" inserts of 21.903, 21.904, 21.906, 21.908, 21.909 and 21.911;
   - 25.2, 25.4, 25.12, 25.15, 25.17, 25.905, 25.907, 25.912, 25.916, 25.923, 25.929 and 25.931;
   - 27.901, 27.902 (BY, INDEX, FOR), 31.2/31.3 through `need_variable_class`, 35.1 (`-=`, `||=`,
     `==`, `a[1]~b =`, `raise failure`, `trace +`), 35.935 (STREAM, USING), 38.2, 46.1, 46.902,
     20.929 (ITEM) and 20.931 (a literal);
   - 18.1/18.2 with blank lines between the IF or WHEN and the offending clause;
   - 24.1 on a literal (`trace '?s'` gives `found "s"`, lower case, as `badOption` takes
     `value->getChar(pos)`);
   - 49.2 on `DO OVER`/`DO WITH` with `FOR n TO`/`BY`, untrapped and under an INTERPRET trap
     (`c~additional` is `[DO OVER loop subkey] [29]` on both).

   The Task 2 review's two literal-`&1` cases (`raise syntax 1 description 'x' 'y'` and
   `address x with input stream ,`) now match the oracle.
2. **Arity, enumerated.** `scripts/arity.py` matches every `syntaxError`/`error*`/
   `requiredEndOfClause` call in `InstructionParser.cpp` and `LanguageParser.cpp`: 145 and 50
   calls, the same counts as a looser regex. It compares the number of inserts passed with the
   highest `&N` in `RexxErrorMessages.h`. The only mismatches are `IP:1122` (20.922), `:3421`
   (89.2), `:4393` (89.1) and `:4616`/`:4631` (27.1). That is the report's list, and no site
   passes fewer inserts than its message uses.
3. **TSV re-derived.** `bash corpus/errors/derive.sh SCRATCH > derived.tsv` exits 0, and its output
   `cmp`s identical to the committed `parse-errors.tsv` (1181 lines).
4. **Mutations** (`--profile mutation -p rexx-parse --test errors --no-fail-fast`, one at a time,
   each restored from a copy, `git diff --stat` empty after each). Baseline: 30 passed, exit 0.
   Each mutant exited 101 with only `every_secondary_message_matches_the_oracle` failing.
   - B, `unmatched_loop_keyword` TO 29 to 28: row 129 only (`do i over x for 1 to 2`).
   - C, 35.935 `STREAM` to `USING`: row 363 only.
   - D, message-assignment insert to `None` (`instruction.rs:555`): rows 1124 and 1125 (`a~b =`,
     `a~b +=`).
   - E, `bad_trace_setting` skipping `!` instead of `?`: row 1130 (`trace ?s`).
   - F, `then_expected` taking the clause's line, not the IF's: 10 rows (101, 136, 375, ...).
   - A, `required_end` inserting `None`: 15 rows, two of them new (1176, 1177).
5. **Perf.** I read `cg-9c-t3/summary.tsv` and recomputed the medians: startup +0.0094%, interpret
   -0.0015%, parse +0.0007% (head vs base), and base/head vs pin match the report's columns.
   `binaries.txt` names the head binary `rexx-run-9c-t3-7436ca94e` (sha256 `788caae6...`).
   `git diff dd253ad15 93de0ca29 -- rust/crates` is empty, so that binary is a valid base. Per
   global constraints, callgrind belongs to Task 10, so this run was outside the per-task check.
   It costs nothing in the tree.
6. **Prose.** No em-dash and no ` -- ` in the added lines. There is one set size (Minor 2).

### Strengths

- Every token lookup sits inside a `#[cold]` helper, and the perf columns are flat.
- `required_end(Msg<1>)` and `block_name(symbol, data)` let the type system choose the arity, so
  the `E20_907`/`E21_907` pairing cannot be crossed.
- `unmatched_loop_keyword` reproduces an oracle default branch that the report found by running
  it, not by reading it. A row witnesses each of its two arms.
- The report's arity list is complete. An independent enumeration of the C++ found exactly those
  four sites.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

1. **`instruction.rs:1275` (89.1), `instruction.rs:1373`/`:1376` (20.922) and
   `instruction/loops.rs:429` (27.1): a trapped error carries the wrong `additional` array.** The oracle passes one insert that the message text never uses.
   `Msg<0>` cannot carry it, so `condition('A')` and `condition('O')~additional` are empty here
   where the oracle's have one item. Measured, one run per side, both under
   `signal on syntax; interpret '<shape>'` and under `.routine~new('r', '<shape>')`:
   - `use arg (a)`: oracle `A items 1`, `[(]`. Crate `A items 0`.
   - `call ns:"x"`: oracle `[x]`. Crate empty.
   - `do while 1 until 2; end`: oracle `[UNTIL]`. Crate empty.

   The controls match on both sides: `nop 1` gives `[1]`, `x: nop` gives `[X]`, and `expose a`
   gives an empty array. The message and rc are identical. 89.2 (`IP:3421`) cannot be reached on
   the oracle either: `parseVariableOrMessageTerm` answers null only for a non-symbol, and that arm
   holds a symbol. This divergence is not new, because the base raised all four with no inserts.
   It is a wrong answer under trap that this task's API cannot express. The plan's arity ruling
   says the controller rules on it. The report named the sites but says "no ruling requested". One
   possible ruling: a build-time arity override to `Msg<1>` for these messages.

#### Minor (Nice to Have)

1. `instruction/loops.rs:376-377`: "after a `DO OVER` or `DO WITH` header's expression, which stops
   there" reads as the OVER expression. That expression does not stop at `TO`/`BY`:
   `do i over q to 2` is rc 0 on both. It is the `FOR` expression that stops there. Name it.
2. `instruction/loops.rs:399-400`: "one of the two" is a set size. Name the set ("at `WHILE` or
   `UNTIL`").

### On the implementer's two concerns

- **The `DO OVER`/`DO WITH` 49.2 inserts.** Confirmed on the oracle, rc 207:
  - `do i over q for 1 to 2` gives `unmatched DO OVER loop subkey 29`.
  - `do with index i over q for 1 to 2` and `do with item i over q for 1 by 2` both give
    `unmatched DO WITH conditional 0`.

  The source marks both as defensive defaults: `// invalid DO OVER loop subkey (should really never
  happen)` (`IP:1491`) and `// invalid DO WITH conditional (should really never happen)` (`:1700`).
  They are reachable because `FOR`'s `TERM_CONTROL` stops at `TO` and `BY`, and neither switch has
  a case for them. The `DO WITH` arm reports `conditionalType`, the post-loop switch's insert
  (compare `:1774`), not the token. That variable is always `SUBKEY_NONE` at that point, because a
  conditional ends the clause. So user input reaches an internal-error message that exposes an
  enum value. That reads as an oracle defect, not intended behaviour. Not ruled.
- **The four sites with an insert the text never uses.** Measured: the difference is observable
  under trap. See Important 1.

### Assessment

**Task quality:** Needs fixes (a ruling on Important 1). The rest is approved.

**Reasoning:** Every converted site I probed or mutated matches the oracle, and the TSV re-derives
byte for byte. The one gap is the `additional` array at three oracle sites whose insert `Msg<0>`
drops. The plan's own arity ruling reserves that decision for the controller.

Scratch worktree and target dir removed.
