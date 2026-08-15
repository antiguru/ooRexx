# Task 3.7b report: the public entry point

## Status

Done. `parse_program` and `parse_interpret` are implemented in
`rust/crates/rexx-parse/src/lib.rs`, composing `scan`, `split_clauses`,
`ClauseCursor`, `parse_instructions`, `parse_instruction` and
`parse_directive` into `Program` and `Fragment`. All three remaining
`#[allow(dead_code)]` attributes are deleted; the phase-gate grep prints
nothing. 219 `rexx-parse` tests pass (196 lib + 23 in the new
`tests/program.rs`), plus the rest of the crate's existing integration
suites unchanged (35 + 13 + 9). Full workspace: 0 failed across 48 test
binaries. `cargo clippy --offline --all-targets -- -D warnings` and
`cargo fmt --check` are both clean.

## Pre-flight findings (raised and confirmed before implementing)

### 1. `Program::labels`'s key type in the brief is wrong

The brief specifies `BTreeMap<Box<str>, usize>`. Measured against
`build/bin/rexx` in a fresh `mktemp -d`, a literal label holding a raw
non-UTF-8 byte works as a `SIGNAL VALUE` target:

```
$ xxd label_binary2.rex
00000000: 7369 676e 616c 2076 616c 7565 2027 6666  signal value 'ff
00000010: 2778 0a73 6179 2022 7368 6f75 6c64 2062  'x.say "should b
00000020: 6520 736b 6970 7065 6422 0a27 ff27 3a0a  e skipped".'.':.
00000030: 7361 7920 2272 6561 6368 6564 206c 6162  say "reached lab
00000040: 656c 220a                                el".
$ build/bin/rexx label_binary2.rex
reached label
```

("should be skipped" did not print, confirming the jump landed rather than
execution simply falling through linearly.) `InstructionKind::Label::name`
is already `Box<[u8]>` for exactly this reason. **Resolution (confirmed by
the coordinator):** `Program::labels: BTreeMap<Box<[u8]>, usize>`.

### 2. First occurrence wins on a duplicate label

Measured, fresh `mktemp -d`:

```
$ cat dup2.rex
signal a
say "unreachable"
a:
say "first"
exit
a:
say "second"
exit
$ build/bin/rexx dup2.rex
first
```

**Resolution:** `build_labels` uses `labels.entry(name.clone()).or_insert(index)`,
never an unconditional overwrite. Pinned by
`a_duplicate_label_keeps_the_first_occurrence_not_the_last`, and a mutation
to last-wins (`.insert` instead of `.entry().or_insert()`) is caught (see
Mutation testing below).

### 3. `directive.rs`'s module doc cited the wrong error number for a directive inside `INTERPRET`

It said 99.915; the actual number, measured via a `signal on syntax` trap in
a fresh `mktemp -d`:

```
$ cat t3.rex
signal on syntax name oops
interpret "::routine r"
exit 0
oops:
say condition('o')~code
say condition('o')~position
exit 1
$ build/bin/rexx t3.rex
99.914
2
```

Message: "INTERPRET data must not contain directive instructions."
(`Error_Translation_directive_interpret`, `RexxErrorCodes.h:683`.) 99.915 is
a real, unrelated error (`Error_Translation_use_local_interpret`), already
correctly implemented in `instruction.rs`'s `use_local`. **Resolution:**
fixed the stale comment in `directive.rs`; implemented the check as one test
right after the main body (not per directive), reported against the first
would-be-directive clause, mirroring `LanguageParser.cpp:1119`'s
`nextClause(); syntaxError(...)`.

## The cursor-sharing redesign (coordinator's suggestion, implemented)

The coordinator asked why not use a single `ClauseCursor` throughout instead
of building a second one from a fresh `split_clauses` call and fast-forwarding
it. I agreed and implemented it: `parse_instructions`'s signature changed from

```rust
pub(crate) fn parse_instructions(ctx: &ParseCtx) -> Result<Vec<Instruction>, ParseError>
```

to

```rust
pub(crate) fn parse_instructions(
    ctx: &ParseCtx,
    cursor: &mut ClauseCursor,
) -> Result<Vec<Instruction>, ParseError>
```

removing its internal `ClauseCursor::new(split_clauses(ctx.tokens)?)`. This
had exactly **one** non-definition call site in the whole crate
(`instruction/tests.rs`'s `parse_kind` helper, which every instruction-grammar
test in that file funnels through), so the blast radius was one function, not
the 200-plus tests that call through it. `lib.rs`'s `parse` now builds one
`ClauseCursor` and reuses it: once for the main body, then again (via the same
`parse_instructions`) for every directive body that has one, since
`parse_instructions` already stops at the first `::` clause and leaves the
cursor sitting there. No second `split_clauses` call, no fast-forward loop,
and no assumption that two independent splits agree with each other.

## Design confirmed correct by measurement

* **Directive bodies are parsed and discarded, not skipped.** Measured, fresh
  `mktemp -d`:

  ```
  $ cat body_if.rex
  say "main"
  ::routine r
  if 1 = 1
  $ build/bin/rexxc body_if.rex
       3 *-* if 1 = 1
  Error 18 running .../body_if.rex line 3:  THEN expected.
  Error 18.1:  IF instruction on line 3 requires matching THEN clause.

  $ cat body_synerr.rex
  say "main"
  ::routine r
  say )
  $ build/bin/rexxc body_synerr.rex
       3 *-* say )
  Error 37 running .../body_synerr.rex line 3:  Unexpected ",", ")", or "]".
  Error 37.2:  Unmatched ")" in expression.
  ```

  So a directive's body gets the same clause-grammar validation as the main
  body, even though assembling its own chain (control stack, `END` matching)
  is Task 3.7c's job and there is no field on `RoutineDirective`/
  `MethodDirective`/`AttributeDirective` yet to hold the result.
  Pinned by `a_body_directives_body_is_validated_not_merely_skipped_missing_then`
  and `..._unmatched_paren`.

* **A bodiless directive needs no special-case check for trailing code**: the
  next `parse_directive` call's own `::`-check raises 99.916 on its own.
  Confirmed for all five directive kinds that never set a body flag, each in
  its own fresh `mktemp -d`:

  ```
  $ build/bin/rexxc class_body.rex     # ::class c / say "trailing"
  Error 99.916:  Unrecognized directive instruction.
  $ build/bin/rexxc options_body.rex   # ::options digits 5 / say "trailing"
  Error 99.916:  Unrecognized directive instruction.
  $ build/bin/rexxc requires_body.rex  # ::requires "x" / say "trailing"
  Error 99.916:  Unrecognized directive instruction.
  $ build/bin/rexxc annotate_body.rex  # ::annotate package a 1 / say "trailing"
  Error 99.916:  Unrecognized directive instruction.
  $ build/bin/rexxc resource_body2.rex # ::resource d / some / data / ::END / say "trailing"
  Error 99.916:  Unrecognized directive instruction.
  ```

  (The first attempt at the `::RESOURCE` case used a lower-case `::end`
  marker against the default upper-case `::END` and got 99.943 instead --
  a marker mismatch, not a body-detection issue; corrected and re-measured
  above.) Pinned by
  `trailing_code_after_a_bodiless_directive_is_99_916_for_every_kind_that_never_has_a_body`,
  which runs all five in one test.

  `::CONSTANT` is the counter-case: it already raises its OWN specific
  number, 99.938, from inside `parse_directive` before the composition loop
  gets a chance to call it again:

  ```
  $ build/bin/rexxc constant_body.rex  # ::constant c / say "trailing"
  Error 99.938:  Constant methods cannot have a method body.
  ```

  Pinned by `a_constant_directives_own_body_check_still_wins_over_the_generic_one`.

* **`::METHOD` and `::ATTRIBUTE` bodies work the same way as `::ROUTINE`'s.**
  This was NOT covered by my first test pass (see Mutation testing below,
  M10/M11) -- the original suite only exercised `::ROUTINE`'s body flag.
  Measured and now pinned:

  ```
  $ build/bin/rexxc method_body.rex     # ::class c / ::method m / return 5
  rc=0
  $ build/bin/rexxc attribute_body.rex  # ::class c / ::attribute a get / return 5
  rc=0
  ```

## Mutation testing

12 mutations against `lib.rs`'s new composition logic (`directive_has_body`,
the 99.914 check, `build_labels`, the `SourceKind` choice in each entry
point, and the body-consuming call itself), applied one at a time to a
snapshot of the finished file, `cargo test -p rexx-parse --offline --test
program` run against each, then reverted. First pass caught 9/10 and missed
one (M10, `::METHOD`'s body flag ignored) because no test yet exercised a
`::METHOD` body; I added
`a_method_directives_body_is_recognised_and_consumed` and
`an_attribute_directives_body_is_recognised_and_consumed`, which closed that
gap and an analogous one for `::ATTRIBUTE`. Final run, 12/12 caught, 0
survived:

```
M1_has_body_always_false                      CAUGHT               test result: FAILED. 11 passed; 12 failed
M2_has_body_always_true                       CAUGHT               test result: FAILED. 22 passed; 1 failed
M3_no_interpret_directive_check               CAUGHT               test result: FAILED. 22 passed; 1 failed
M4_wrong_interpret_error_number               CAUGHT               test result: FAILED. 22 passed; 1 failed
M5_labels_last_wins                           CAUGHT               test result: FAILED. 22 passed; 1 failed
M6_labels_off_by_one                          CAUGHT               test result: FAILED. 19 passed; 4 failed
M7_parse_program_uses_interpret_kind          CAUGHT               test result: FAILED. 4 passed; 19 failed
M8_parse_interpret_uses_program_kind          CAUGHT               test result: FAILED. 21 passed; 2 failed
M9_body_never_consumed                        CAUGHT               test result: FAILED. 11 passed; 12 failed
M10_directive_has_body_ignores_method         CAUGHT               test result: FAILED. 22 passed; 1 failed
M11_directive_has_body_ignores_attribute      CAUGHT               test result: FAILED. 22 passed; 1 failed
M12_directive_has_body_ignores_routine        CAUGHT               test result: FAILED. 13 passed; 10 failed
BASELINE_after_revert                         PASS                 test result: ok. 23 passed; 0 failed
```

`diff` against the pre-mutation snapshot after the run confirmed a clean
revert (`IDENTICAL - clean revert confirmed`).

## Housekeeping

All three `#[allow(dead_code)]` attributes named for Task 3.7b are deleted
and each is reachable from `lib.rs`'s composition:

* `directive.rs`'s `parse_directive` -- called from `parse`'s directive loop.
* `instruction.rs`'s `parse_instructions` -- called for the main body and for
  every directive body that has one.
* `instruction.rs`'s `parse_instruction` -- called transitively through
  `parse_instructions`' loop (unchanged from Task 3.6/3.7).

Gate grep, run from `rust/`:

```
$ grep -rnE '^\s*#\[allow\(dead_code\)\]' crates/rexx-parse/src/ | grep -v 'Task 3\.[0-9]'
(no output)
```

None turned out unreachable, so there is no finding to report there.

## Files changed

* `rust/crates/rexx-parse/src/lib.rs` -- `Program`, `Fragment`,
  `parse_program`, `parse_interpret`, the shared `parse` helper,
  `directive_has_body`, `build_labels`. Updated the crate-level comment
  about dead-code allowances (now zero, was three).
* `rust/crates/rexx-parse/src/instruction.rs` -- `parse_instructions` takes a
  caller-supplied `&mut ClauseCursor` instead of building its own; doc
  comment updated to describe the new contract and why it exists (the same
  "one code body" loop is now reused for directive bodies); both
  `#[allow(dead_code)]` attributes removed; unused `split_clauses` import
  removed.
* `rust/crates/rexx-parse/src/instruction/tests.rs` -- the one call site
  (`parse_kind`) updated to build its own `ClauseCursor` and pass it in.
* `rust/crates/rexx-parse/src/directive.rs` -- `#[allow(dead_code)]` removed
  from `parse_directive`; the module doc's error-number comment corrected
  from 99.915 to 99.914 with the measurement backing it.
* `rust/crates/rexx-parse/tests/program.rs` (new) -- 23 tests: the brief's
  own Step 1 test verbatim, main/directive boundary in both directions,
  directive-body validation (both a block-structure error and an
  expression-grammar error), the five-kind bodiless-directive sweep, the
  `::CONSTANT` counter-case, `::METHOD`/`::ATTRIBUTE` body recognition,
  `Program::labels` (symbol upcased, literal verbatim, non-UTF-8 bytes,
  first-occurrence-wins, directive-body labels excluded), `parse_interpret`
  rejecting both a directive (99.914) and a label (47.1) plus the positive
  case, `Fragment`'s independent `SymbolTable` across two calls, a
  `Directive::clause_span` sanity check, and the full `rust/corpus/lang/`
  walk (counts the directory rather than hard-coding 14, with a floor
  assertion so an emptied directory cannot pass vacuously).

## Test summary

`cargo test -p rexx-parse --offline`: 219 rexx-parse tests (196 lib + 23
`tests/program.rs`) plus 35 + 13 + 9 in the crate's other integration suites,
0 failed. Full workspace (`cargo test --offline`): 48 test binaries, 0
failed. `cargo clippy -p rexx-parse --offline --all-targets -- -D warnings`:
clean. Workspace-wide clippy: clean. `cargo fmt -p rexx-parse --check`:
clean.

## Concerns / open items for the coordinator

* `parse_interpret`'s `debug_assert!(parsed.directives.is_empty(), ...)` is a
  debug-only sanity check, not load-bearing: the actual guarantee is the
  99.914 `Err` return inside `parse`, which fires before the directive loop
  ever runs for `SourceKind::Interpret`. Flagging in case a release-profile
  guarantee was expected here instead.
* Nothing in this task's scope required touching `rexx-num`; the workspace's
  earlier uncommitted `rexx-num` changes (visible in the session's initial
  `git status`) were not present by the time I checked and are untouched by
  me either way.
