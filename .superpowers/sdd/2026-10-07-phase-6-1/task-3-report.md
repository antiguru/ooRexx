# Phase 6.1 Task 3 report: USE ARG targets, `.context~condition`, `CONDITION('D')`, stem attributes, OPTIONS, USE LOCAL, objects in DO/FORWARD/RAISE

## Status

DONE_WITH_CONCERNS. Every group's witnesses match the oracle on all three descriptors, and they all
refuse (rc 120) on the base binary. The per-task check and `whole_groups` pass. One constructor the
brief lists, `Loud::object_position`, is kept (Concerns 1). A gated concurrency test fails, and it
fails the same way at base (Concerns 2).

## Commits

- `f763836ce` c2: USE ARG into a message or bracket term.
- `8f080d2fa` c3: `.context~condition` answers a copy of the condition.
- `de455d5dc` c4: `CONDITION('D')` for a NOVALUE with no description. `Loud::builtin_option_object` deleted.
- `f51f1c93c` c5/c6: `::ATTRIBUTE` and `DELEGATE` on a stem or compound name. `Loud::accessor_variable` and `Loud::delegate_variable` deleted.
- `2b7babfee` rustfmt over c5's `variables.rs` helper, which `f51f1c93c` committed unformatted.
- `4605eb332` b1: OPTIONS.
- `1f054f1c5` b2: USE LOCAL as a method's first instruction. `Loud::use_local_in_a_method` deleted.
- `652826b54` b10: objects in DO headers and control variables, FORWARD ARGUMENTS, RAISE ADDITIONAL.
- `61d441781` whole-group expectation lines, `GUARD_PASSING`, gate record `## Task 3`.
- this report and the gate record's closing lines follow in one commit.

The `refusal-sites.tsv` row for `builtin_option_object` in `8f080d2fa` was edited by hand, for the
state between c3 and c4: `body`, `off-send-surface`, `-`, which is what the refresh at
`tests/refusal_sites.rs:472-493` writes for a row off the send surface. `de455d5dc` deletes the row
with a refresh. Every other commit's table comes from `REXX_REFUSAL_SITES_REFRESH=1`.

## Evidence method

Oracle runs used the wrapper from the global constraints. Each program ran from a fresh empty
directory, and stdout, stderr and status were compared separately (`/tmp/claude-1000/p61/t3/wit.sh`,
`run.sh`). RED is the release binary built from `1d308cb9d` (`/tmp/claude-1000/p61/t3/bin/base/rexx-run`,
sha256 `b98c5fb111fbc135dffbd84604585e47334f7dd864192536055e78b667c9986b`). GREEN is the release
binary at `652826b54`. All 25 witnesses: RED rc 120 against the oracle, GREEN identical on all three
descriptors (`/tmp/claude-1000/p61/t3/red.txt`, `green.txt`). Probes, scout A's plus mine: 61 of 73
agree on GREEN. The 12 that do not are listed under Concerns 3 to 5. I read each witness's oracle
stdout to check that every path its comment names prints.

## Per group

**c2 USE ARG into a message or bracket term.** `bind_use_target` (`run.rs`) traces the argument's
`>>>` and then sends a `Message` target through `assign_expr_target`'s message arm, the one PARSE
uses. That arm is `RexxExpressionMessage::assign`: it traces the term's own arguments and the
assignment. `drop` does nothing for a message term (`ExpressionBaseVariable.hpp:65`), so an omitted
argument without a default leaves the term unchanged. USE STRICT ARG still raises.
Witnesses: `use_arg_msg`, `use_arg_bracket`, `use_strict_arg_msg` (ends in 40.5, rc 216),
`use_arg_msg_default`, `use_arg_msg_method`, `use_arg_msg_trace` (`trace r`, `trace i`, defaults).

**c3 `.context~condition`.** Answers `condition_copy` of the frame's condition object, and `.nil`
when there is none. Witnesses: `context_condition` (`.nil` outside a handler, a new copy on each
send, an internal call inherits it) and `context_condition_call` (CALL ON ERROR).

**c4 `CONDITION('D')`.** Answers `''` for `(None, NOVALUE)`. Witness: `condition_d_raise_novalue`
(no DESCRIPTION, a DESCRIPTION, and a variable read).

**c5/c6 stem and compound accessors and delegates.** `Interp::pool_value` and `set_pool_value`
(`variables.rs`) implement `RexxVariableBase::getValue` and `set` on a variable dictionary. A stem is
the pool's stem, created empty on first use. A compound is one tail of that stem, and the tail is
everything after the first period, taken literally: `getRetriever` builds the name with
`buildCompoundVariable(name, true)` (`LanguageParser.cpp:2524`). I measured this before
implementing it: `o~a.i = ...` writes tail `I` even when the pool's `I` holds `'B'`. An unset tail
answers the stem's default, or else the stem's own name with the tail appended
(`stem_object_compound`). The setter on a stem wraps a non-stem value as a new stem's default.
Witnesses: `attr_compound`, `attr_stem`, `delegate_compound` (ends in 97.1, rc 159), and
`delegate_stem`. The two `dispatch/tests.rs` pins now run, with the oracle's `5` and `a.b`.

**b1 OPTIONS.** Evaluates the expression, converts it with `required_string_value`, traces `>>>`, and
does nothing else (`RexxInstructionOptions::execute`). Other changes:
- `instruction_owner` no longer gives `Options` an owner.
- `owners.rs` has `Options` as `InScope`, so `EXPECTED_OUT_OF_SCOPE` is empty and the Phase 5 count
  is 0.
- `loud.rs` has no instruction witnesses. `Category::Instruction` carries the
  `#[expect(dead_code)]` that `Category::Expr` already had.
- `spike.rs`'s two tests now use `.local['STDQUE']` (a Phase 10 refusal) as their loud example.
- The OPTIONS row in `dispatch/tests.rs` is deleted.
- The exclusions known-gap row for OPTIONS is deleted.

Witnesses: `options` and `options_expr` (MAKESTRING traced under `trace r`).

**b2 USE LOCAL.** `Interp::use_local` implements `RexxActivation::autoExpose`. The listed names and
`SELF`, `SUPER`, `RC`, `SIGL` and `RESULT` stay local. Every other simple or stem name the plan or
`extra` holds is bound to the receiver's pool, through the same exposure list EXPOSE uses. That list
is also what GUARD WHEN watches. A name first met later is bound when `slot_of` creates its extra
slot (`plan.rs`), using `ActivationCold::auto_expose`. Internal calls inherit it and PROCEDURE
clears it. `Activation` is still 512 bytes. Witnesses: `use_local_method`,
`use_local_method_expose`, `use_local_method_dynamic` (VALUE, INTERPRET, a stem tail, an internal
call, PROCEDURE) and `use_local_method_guard`.

**b10 objects in DO, FORWARD ARGUMENTS and RAISE ADDITIONAL.**
- A DO header's initial, TO or BY object is sent unary `+` through `apply_prefix`, giving 97.1 where
  the object has no `+`. If `+` answers a non-object, the loop runs on that answer. If BY's `+`
  answers an object, that object is sent `<` against 0.
- A control variable holding an object is sent `+` with BY at the increment. If it answers an object,
  that object is sent the TO comparison (`>`, or `<` when BY is negative).
- If the oracle would go on with an object (an object initial or TO answer, or an object control
  value that answers the comparison or has no TO), the loop still refuses through
  `Loud::object_position`.
- FORWARD ARGUMENTS on an instance, a native object or a weak reference, and RAISE SYNTAX ADDITIONAL
  on an object, both go through `request_array_value`. That function is renamed from
  `request_array_for_over` and implements `requestArray`. No array, or a multi-dimensional one,
  raises 98.946 for FORWARD and 98.939 for RAISE (`Raised::syntax_additional`, new). The condition
  object carries the converted array. RAISE traces `>K>` before converting, as the oracle does.
- The pins in `object_operand_tests.rs` now assert the oracle's bytes. The R12 table in the
  exclusions file moves these rows to "agrees".

Witnesses: `do_to_class`, `do_header_objects`, `do_ctrl_class` (the oracle's `*-*   end`
traceback), `do_ctrl_objects` (including `trace r` with a negative BY), `forward_args_object`, and
`raise_additional_object` (through a `::routine`'s `RAISE ... RETURN`; RAISE without RETURN is
handled as EXIT and is not trapped in the caller).

## Files

`rust/crates/rexx-exec/src/`: `run.rs`, `run/loops.rs`, `run/condition.rs`, `plan.rs`, `activation.rs`,
`variables.rs`, `stem.rs`, `redirect.rs`, `dispatch.rs`, `dispatch/context.rs`, `dispatch/library.rs`,
`dispatch/tests.rs`, `builtin/state.rs`, `error.rs`, `lib.rs`, `eval/object_operand_tests.rs`.
`rust/crates/rexx-exec/tests/`: `owners.rs`, `loud.rs`, `spike.rs`, `collect_stress.rs` (three
programs added to its no-collection list), `state_builtin_oracle.rs`, `bif_assertions.rs`,
`keyword_assertions.rs`, `concurrency_tests.rs`. Also `rust/corpus/phase-6-1.txt`,
`rust/corpus/refusal-sites.tsv`, the 25 `rust/corpus/lang/` witnesses and their
`rexx-parse/tests/sourceline_oracle/` files, `docs/superpowers/plans/phase-4-exclusions.txt`, and
`docs/superpowers/plans/phase-6-1-gate.md`.

## Checks

- `cargo fmt --all --check`: exit 0. `memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D
  warnings`: exit 0 at `652826b54`. `61d441781` changes only `concurrency_tests.rs` and the gate
  record, and passes `cargo fmt --check`.
- At `652826b54` (`/tmp/claude-1000/p61/t3/gate/status.txt`):
  - `memcap 8G cargo test -j 4 --workspace --no-fail-fast`: exit 0.
  - `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus --test
    ir_recorded_oracle`: exit 0.
  - `whole_groups`: exit 101, on the expectation rows that `61d441781` rewrites.
- At `61d441781`: `whole_groups` exit 0, 6 passed (`whole2.log`).
- The full gated `concurrency_tests` file: 37 passed, 1 failed (Concerns 2).
- The row-by-row reasons for the whole-group changes are in the gate record. In summary: RexxContext
  whole now refuses at TESTCOPY01 (Phase 9); RexxContext rest gains 3 assertions; TRACE rest gains 1
  (TEST_TRACE_OPTIONS); the GUARD derived rows agree with the oracle; TEST_WHEN_USE_LOCAL_NO_WAIT
  joins `GUARD_PASSING`.

## Self-review

- Comments that the changes made false are deleted or cut: `instruction_owner`'s notes on Use, Raise
  and Forward; the matching notes in `owners.rs`; `object_position`'s doc; the `parse_loud` notes in
  the bif and keyword assertion files; `state_builtin_oracle.rs`'s note on `DECLARED_GAPS`; the
  `GUARD_PASSING` doc.
- Rooting: each new send's answer is pushed as a temp before the next send.
  `single_dimension_request` roots the converted array. The USE LOCAL owner and scope are in
  `object_roots`.
- No hot path changed. The new DO code is behind the existing object checks, in `#[cold]`
  functions. `slot_of`'s new branch runs only when an extra slot is created.

## Concerns

1. **`Loud::object_position` is kept.** The brief lists it for deletion, but two sites still use it.
   - DO OVER's directory refusal (`loops.rs:665`). Scout A filed this as b11, a GUARD. It is
     reachable: `object_operand_tests::an_object_as_a_do_over_target_is_loud`, `do e over
     .context~package~local`, oracle rc 0.
   - The object-answer cases in the DO header and control variable. Four probes stay loud where the
     oracle answers: `do i = .environment to 5` and `do i = .local to 3` (oracle 97.1 on `>`), `do
     i = 1 to .local` (the oracle runs the loop with a string comparison against `.nil`), and a
     control variable set to `.local` with no TO (the body runs once more with `.nil`). Supporting
     them needs a loop whose control value is an object.
2. **`the_s2_rows_of_the_derived_list_in_both_modes` fails.** TRACE_TraceObject
   TEST_TRACEOBJECT_COLLECTOR differs between the two scheduler modes. It fails the same way on the
   base tree `1d308cb9d`, built from `git archive` in its own target directory with the
   repository's `ootest`/`extensions` linked in. So it is not caused by this task, but the gated
   `concurrency_tests` file is not green. It is outside the `whole_groups` filter the brief names.
3. **A silent divergence that a refusal used to hide.** `c_condition_d_raise7` (`call on any`, then
   `raise novalue return` in a callee) was loud. It now runs: ours enters the handler, while the
   oracle prints only `back`. This is the reflected-condition defect in scout A's section 4: CALL ON
   ANY traps a condition that CALL cannot trap. `raise lostdigits return` already showed the same
   divergence at base (my probe `x_any_raise_syntax`). It is not fixed here.
4. Five FORWARD probes (`b_forward_args_*`, `b2_forward_args_*`) and `use arg >o~a` stop at a
   top-level parse error, which is group c8 and not this task.
5. The c3 commit's hand-edited table row (see Commits) was never checked by building that commit on
   its own.

## Fix round 1

Commits: `1d7b27de8` (message-driven loops, DO OVER a native directory), `9172b7085` (those paths
out of line), `150611a38` (records and the collector row), and this report's commit.

1. **Important 1 and Concern 1: loops whose control value is an object.** These are now implemented
   as `DoBlock::checkControl`, not refused. Once a controlled header value is an operator receiver,
   `accept_object_header` keeps each `+` answer as it is. `BY`'s answer is sent `<` against 0 for
   the direction. The loop then runs as `LoopState::ObjectControlled`:
   - At each re-test, `+` with BY is sent to whatever the control holds (`arith_general`). The answer
     is traced, then bound unchanged.
   - The TO comparison (`>`, or `<` for a negative BY) is then sent to it. The loop ends when that
     answers the true object. FOR is checked after the comparison, as the oracle does.
   - A numeric loop whose control variable is given an object in the body switches to this state at
     the increment. TO and BY become the objects for their numbers.
   - A failure on the first test is blamed on the DO clause at the body's indent, as the oracle
     blames it.

   The FlatLoop states' objects are rooted from `Activity::object_roots`. The reviewer's
   `do_plus_string.rex` now agrees: `'abc'` ends the loop, and `' 2 '` prints `[ 2 ]`. All four
   loud probes from Concern 1 agree. `do i = .nil to 3` agrees as well. I corrected the exclusions
   R12 text and table.
2. **DO OVER `.context~package~local`** iterates the native directory's indexes (sorted, as a
   StringTable's are). `Loud::object_position` and `HeaderRole::value_name` have no site left and are
   deleted. `refusal-sites.tsv` is refreshed.
3. **`the_s2_rows` bisect.** The test passes at `3cee3e622`, where the row refuses with "DO is not
   implemented", and fails at `5b7acef35` (Task 2's COUNTER). It is a gap that the removed refusal
   exposed, not a regression: both modes end in the same 97.1 at rc 2, with the same masked stdout and
   the same stderr lines, but the REPLY continuation's trace lines come in a different order.
   `TRACE_INTERLEAVES` now allows that case. The reason is in the gate record.
4. **Records.**
   - Exclusions: a known-gap row for `raise novalue return` / `raise lostdigits return` reaching a
     caller's CALL ON ANY.
   - `oracle-crashes.txt` entry 28: FORWARD ARGUMENTS over a List, SIGSEGV rc 139 in 5 of 5 runs.
   - `oracle-crashes.txt` entry 29: GUARD WHEN on a compound tail, killed at the timeout in 5 of 5
     runs.

Witnesses: `do_object_control` (header objects, a control switching mid-loop, non-canonical
answers, FOR, negative BY, LEAVE, ITERATE, a compound control, a control object whose `>` answers
true, `.local`), `do_object_control_trace`, and `do_over_package_local`. On the binary before the
fix, `do_object_control` and `do_over_package_local` refuse with rc 120. `do_object_control_trace`
already agreed there: it checks the new path's trace and adds no new failing case. The 73 probes:
67 agree. The six that do not are the c8 parse-error ones, `use arg >o~a`, and the two CALL ON ANY
ones now recorded.

Checks at `150611a38`:
- `cargo fmt --check` and `clippy -D warnings`: exit 0.
- `memcap 8G cargo test -j 4 --workspace --no-fail-fast`: exit 0.
- `REXX_CORPUS_GATE=1 ... --test corpus --test ir_recorded_oracle`: exit 0 (29 passed, 1 ignored;
  21 passed).
- The whole gated `concurrency_tests` file in release: exit 0, 38 passed. This includes
  `whole_groups` and `the_s2_rows_of_the_derived_list_in_both_modes`.

Performance (added to the round): `012bf8ab1` asks `operator_message_receiver` only after a
conversion fails. Task 2's callgrind command against the 6.1 base, pads at 0: emptyloop -0.0019%,
decloop -2.8190%, rexxcps -0.0008%, all inside +0.5%. Wall clock, five interleaved runs: emptyloop
+3.96%, decloop -5.88%, rexxcps -0.87%, all inside ±4%. The binary before that commit was decloop
+1.02%, over budget. Figures and commands are in the gate record under `### Task 3 fix round 1`.

## Fix round 2

Commits: `308386167` (rooting), `b0b72baa4` (re-review minors), `1a46f2eb9` (true-object known gap),
and this report's commit.

1. **Critical: TO/BY rooting.** A flat loop is out of `flat_top` for the whole pass boundary, so its
   state's objects were not roots there.
   - A header's `+` answers are now written into the registers their header values were evaluated
     into (`ObjectHeader::home`, set in `file_header_value`, written in `flat_loop_start`). Those
     registers live as long as the loop, the same way the DO OVER snapshot's register does.
   - A TO, BY or initial value that converted as a number stays a `LoopBound::Number`. Each pass that
     sends to it makes its object as a temp in that pass. This includes the switch from a numeric
     loop, which now builds no objects (`object_control_from_numbers`, a free function).
   - The reviewer's `dead*.rex` probes agree with the oracle (`dead2` and `dead3` 3 of 3 runs each),
     as do `mixed`, `mixed2`, `switch`, `trace_switch`, `label` and `four`.
   - Witness `do_object_bounds_rooted` (a switch with a heap TO and BY and an allocating `+`/`>`,
     and a header whose BY answer is an object).
   - Crate test `collect_stress::a_message_driven_loops_to_and_by_survive_collect_on_every_allocation`
     runs it under collect-on-every-allocation. RED: with `012bf8ab1`'s `loops.rs` swapped in, the
     test fails with "a message send to a value whose object is no longer live". GREEN with the fix,
     and it asserts collections > 0.
   - `012bf8ab1`'s reordering keeps this right: an object header value still reaches
     `accept_object_header`, and its answer is homed.
2. **Important: `is_true_object`.** Ruled (Moritz, 2026-10-08): this crate's behaviour is right, and
   the oracle's identity-only `== TheTrueObject` test has a defect. That test is at `DoBlock.cpp:213`
   and `DoBlockComponents.cpp:173`. WHILE and UNTIL fall back to `truthValue` (`:277-316`), and Rexx
   logical values are the strings 0 and 1. There is no code change.
   - The known-gap row from `1a46f2eb9` is replaced by DEVIATIONS entry 25 (owner none), with the
     citations and the gt_one, gt_one2 and by_dir2 measurements. Its
     `LICENSED DIVERGENCE WITNESS: do-compare-computed-true` row in `tests/licensed_divergences.rs`
     runs a `>` answering `left('12', 1)` on both sides: the oracle prints `pass` three times and
     `end`, this crate prints `end`.
   - The `is_true_object` doc now points at the deviation.
   - The cases that agree (a comparison, `.true`, a literal `1`, `'1'`, `0 + 1`) are the corpus
     witness `do_object_compare_true`.
3. **Minors.**
   - The known-gap row's LOSTDIGITS output now reads `D=[] LOSTDIGITS`.
   - The `over_snapshot` and `hash_collection_indexes` comments and Deviation 8's WHY now name the
     native `Directory`.
   - New witness `do_object_first_test_error`, the untrapped first-test traceback.

Checks:
- At `1a46f2eb9`: `cargo fmt --check` and `clippy -D warnings` exit 0 (again after the Deviation 25
  commit, with the gated corpus, `sourceline_oracle` and `licensed_divergences` passing).
- `memcap 8G cargo test -j 4 --workspace --no-fail-fast`: exit 0.
- `REXX_CORPUS_GATE=1 ... --test corpus --test ir_recorded_oracle`: exit 0.
- At `308386167`, whose code is unchanged since: the gated release `concurrency_tests` file, run as
  `whole_groups` (6 passed) and the rest (32 passed).
- `collect_stress`: 37 passed.
- Witnesses: all agree with the oracle. Probes: 81 of 87 agree; the six others are the same known
  exceptions as before.
- Instructions against the 6.1 base (`callgrind.sh -r 1`): emptyloop -0.0018%, decloop -2.38%,
  rexxcps +0.09%, fibcall +0.17%.

## Fix round 3

Commits: `dcd1db992` (truth value; loop objects once per loop), `0c75907d3` (perf round 1), and this
report's commit.

1. **DO TO test and BY's direction test by truth value** (Moritz's refined ruling).
   - `loop_truth` replaces the identity test, answering the way WHILE and UNTIL do: text exactly
     `1` ends the loop or means a negative step, and `0` goes on.
   - Anything else raises 34.901 (`Error_Logical_value_method`). That is the subcode the oracle's
     truthValue uses for a method's logical answer (`IntegerClass.cpp:1432`, `StringClass.cpp:645`);
     no 34.x subcode names a DO header.
   - Deviation 25 is rewritten with the rule, its reason and the measured table:
     - a comparison, `.true` and `abbrev(...)` end the loop on both sides;
     - `0` and `1 = 2` go on on both sides;
     - `0 + 1`, `1 * 1`, literal `1`/`'1'`, `left('12', 1)`, `.true~copy` and similar now end it here,
       where the oracle runs 3 passes;
     - `'abc'` raises 34.901 here, where the oracle runs 3 passes;
     - BY's `<` answering `left('12', 1)` counts down here.
   - The licensed witness `do-compare-computed-true` now uses `return 0 + 1` (oracle 3 passes, ours
     none).
   - `do_object_compare_true` holds the agreeing cases.
   - The rename the brief allowed: `is_true_object` is replaced by `loop_truth`, whose doc names
     Deviation 25.
2. **Objects once per loop.**
   - `ObjectControl`'s initial, TO and BY are objects again. Numbers are made into objects once,
     when the loop starts or when its control switches to an object.
   - They are held on `Activity::loop_objects`, keyed by the boxed state's address, for the loop's
     life. That is what keeps them alive while a pass boundary has the loop out of `flat_top`.
   - They are released where the loop ends, where the start or a pass boundary fails, and in
     `unwind_frames`. They move with a REPLY continuation's loops.
   - This replaces round 2's register homes and per-pass objects.
   - `ident2.rex`: every line matches the oracle except the first `to same` after the switch. There
     the oracle answers `1` because a freed object's address was reused (identityHash is an address),
     which is not a property to match. The rest of `ident2`, and `dead*`, `reg1` and `reg2`, match.
     The collect-stress test from round 2 still passes.
3. **Perf.** Task 2's callgrind command against the 6.1 base, pads at 0, on the binary built from
   `git archive 0c75907d3`: emptyloop -0.9620%, decloop -2.6728%, rexxcps -0.4940%. The first build
   of this round, `dcd1db992`, was emptyloop +0.9614%; perf round 1 fixed it by moving the release
   out of line. Details are in the gate record.

Checks at `0c75907d3`:
- fmt and clippy: exit 0.
- `--lib`: 1016 passed. `collect_stress`: 37 passed.
- At `dcd1db992`: gated corpus and `ir_recorded_oracle`, `sourceline_oracle` and gated
  `licensed_divergences` pass.
- Gated release `concurrency_tests`: `whole_groups` 6 passed and the rest 32 passed.
- Witnesses: all agree with the oracle.
- Probes: the only new differences are the Deviation 25 cases (`true1`, `gt_one`, `gt_one2`,
  `by_dir2`, `z_nonlogical`) and `ident2`'s address-reuse line.
- The debug workspace run was not re-run this round.

### Fix round 3 addendum: references, commands, rationale

`945e31f33` makes `loop_truth` read the answer's string value (`string_value_text`, the counterpart
of truthValue's `requestString`) rather than its rendering, so an array answer is reported as
`found "an Array"`. Deviation 25 states this and adds the row. Line numbers below are at `945e31f33`;
`rust/crates/rexx-exec/src/` is abbreviated.

- `run/loops.rs:2718` `loop_truth`: `LOGICAL_TRUE` gives true and `LOGICAL_FALSE` gives false.
  Anything else is taken through `string_value_text` and `eval::logical_value`, and is exactly `1`
  or `0` or raises 34.901 (`Raised::not_logical`) with the string as the insert. `' 1'` raises.
- It is called at `run/loops.rs:963` (BY's sign check, `accept_object_header`) and
  `run/loops.rs:2768` (the TO test, `object_control_pass`).
- `docs/superpowers/plans/phase-4-exclusions.txt:1786`, Deviation 25: the rule, its reason, 34.901
  and why that subcode, and the measured table.
- `tests/licensed_divergences.rs:69`: the row `do-compare-computed-true` (`return 0 + 1`).
- `rust/corpus/lang/do_object_compare_true.rex`: the cases that agree with the oracle.
- `run/loops.rs:981` `object_control_state` and `:2691` `object_control_from_numbers` turn the
  numbers into objects once per loop.
- `:1022` `hold_object_control` holds them, and `:1035` `release_loop_objects` releases them. Its
  callers are `run/loops.rs:1789`, `:1799`, `:1860`, `:1875` and `ir/drive.rs:3714`.
- `:339` `FlatLoop::object_key`.
- `activity.rs:85` declares the field and `:546` roots it. `ir/drive.rs:2970` moves the entries with
  a REPLY continuation.

**Why `loop_objects` sits on `Activity`, not `Activation` or the header registers.**
- The roots have to cover exactly the window round 2's defect came from. A pass boundary
  (`flat_loop_step_top`) takes the loop's box out of `flat_top` and runs user code while it is out:
  the UNTIL test, a trap delivered at the boundary clause, and the `+` and comparison sends.
- The flat loops are on the activity, so their roots belong beside them. A REPLY continuation moves
  `flat_loops`/`flat_top` to a new activity (`ir/drive.rs:2958-2961`), and the entries move with them
  by key.
- An `Activation` does not own its loops. An internal call without PROCEDURE runs further loops in
  the caller's frame, and recursion stacks loops of different activations on one activity, so
  per-activation storage would have to search the stack to release.
- Header registers held the header's own answers in round 2, but a numeric loop that switches to an
  object mid-loop has no register access in `loop_advance`. The objects it makes then had no home,
  which is why round 2 made them afresh each pass and lost their identity.
- One activity-level list keyed by the boxed state's address covers the header case and the switch
  case alike. It is empty outside message-driven loops, and the empty check sits in the cold release
  path.

**Commands and results at `945e31f33`** (from `rust/`):
- `cargo fmt --all --check`: exit 0.
- `memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D warnings`: exit 0.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --no-fail-fast -p rexx-exec --lib --test corpus
  --test ir_recorded_oracle --test licensed_divergences`: exit 0. lib 1016 passed; corpus 29 passed
  and 1 ignored; `ir_recorded_oracle` 21 passed; `licensed_divergences` 22 passed.
- At `0c75907d3`, whose code differs from `945e31f33` only in `loop_truth`'s string read:
  - `memcap 8G cargo test -j 4 -p rexx-exec --lib --test collect_stress`: 1016 and 37 passed.
  - `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --release --test concurrency_tests
    whole_groups`: 6 passed.
  - The same command with `-- --skip whole_groups`: 32 passed.
- Probes `z_arr` and `z_nonlogical` (`/tmp/claude-1000/p61/t3/probes/`) show the 34.901 cases as
  Deviation 25 tabulates them.

**Perf against the 6.1 base**, callgrind Ir. The command, from the repository root:

    bash rust/bench-programs/callgrind.sh -r 3 -j 10 -o $S/cg-h3 -p "emptyloop decloop rexxcps" base=$B/base/rexx-run pad1=$B/pad1/rexx-run pad3=$B/pad3/rexx-run t3=$S/bin/h3/rexx-run

`B=/tmp/claude-1000/p61/t1/bin`, `S=/tmp/claude-1000/p61/t3`. The binary is `git archive
0c75907d3`, sha256 `f5ea13e6...`. Results, against a band of 0:

| program | base Ir | t3 Ir | delta |
|---|---:|---:|---:|
| emptyloop | 7,786,100,369 | 7,711,200,008 | -0.9620% |
| decloop | 2,469,011,119 | 2,403,018,557 | -2.6728% |
| rexxcps | 17,788,421,116 | 17,700,549,564 | -0.4940% |

`945e31f33` changes only `loop_truth`'s non-fast path, which none of these programs reach, and it
was not re-measured.

**Concerns.**
- The debug workspace run was last done at `1a46f2eb9` (exit 0) and not repeated this round.
- When the oracle reuses a freed object's address, identity probes show `same` where we show a new
  object (`ident2`'s first line after a switch). That is not a behaviour to match.
