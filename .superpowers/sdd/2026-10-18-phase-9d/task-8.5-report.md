# Phase 9d Task 8.5 report: variables

Status: DONE_WITH_CONCERNS. Base `6e0bf5e5b`. Code `b69a1babc` and `2589e88b3` (the second keeps
the run path's reads at their base cost, see Perf); records `d7eb10cbc` and the commit that adds this
report. The closing lines and inventory rows name `2589e88b3`. Records: `docs/superpowers/records/2026-10-18-phase-9d/t8.5/`
(`probes/` holds each probe with its oracle `.o.*` and head `.t.*` outputs).

## What changed, per item

**2026-09-30-drop-unset-stem-symbol.md (closed).** Derived rule: rexxref intro.xml "Stems" (the Stem
object "is automatically created the first time you use the stem variable or a compound variable
containing the stem variable name") and collclasses.xml's Stem class ("Stems are automatically
created whenever a Rexx stem variable or Rexx compound variable is used"). The crate created the stem
only on a bare-stem read or a tail assignment, so after `drop w.1`, `say y.3`, `value('Z.1')`,
`symbol('A.1')`, a `PARSE` or `INTERPRET` read, or a read through `PROCEDURE EXPOSE j.`, `SYMBOL('W.')`
answered `LIT` and `.context~variables` omitted the stem (`probes/compound_uses.rex`). The oracle
follows the rule. Now a compound read (`stem_get_at`, `stem_get`, through the cold
`tail_of_unused_stem`) and `stem_drop_tail_at` create the stem variable where the slot holds none
(`Interp::create_stem_variable`, cold, `stem.rs`), as `read_stem_at` already did. `SYMBOL`
of the bare stem name still creates nothing, as on the oracle. The `VALUE` builtin's rooting comment
(`builtin/datatype.rs`) described the old "no object at all" branch and is rewritten.

**2026-10-19-local-novalue-hook.md (closed).** No rexxref text describes the hook (a search of
`oodocs/` for NOVALUE beside `.local`, `.environment`, handler or exit finds none); the item is ruled a
gap, so the crate models the oracle's `RexxActivation::handleNovalueEvent`
(`RexxActivation.cpp:2645-2674`) and `novalueHandler` (`:4763-4773`): `::OPTIONS NOVALUE SYNTAX`'s
98.986 first, then `.local`'s `NOVALUE` entry sent `NOVALUE` with the derived name, an answer other
than `.nil` being the value with the variable left unset, then the `NOVALUE` condition. The hook is
sent from inside an expression's unset read, the cold path that already derives the name
(`Interp::novalue_answer`, `run/condition.rs`, called from `variables.rs`'s `unset_read`, `stem.rs`'s
`tail_of_unused_stem` and `unset_tail` where `HOOK` holds, and `run.rs`'s `PROCEDURE EXPOSE`
compound selector). The read still answers `Novalue::Unset`; what the hook came to (answered, or its
failure) waits in `Interp::novalue_hooked` for the `novalue_check` right after the read, which takes
it first. `stem_get` (`SYMBOL`, `VALUE`, the native API) reads without the hook. The send runs under
a new pinning frame `PinKind::NovalueHook`, added to `concurrency_tests.rs`'s `FRAME_PROBES`.
Measured on both (`probes/novalue_hook_contexts.rex`, and `local_novalue_hook.rex`): a simple variable, a compound (not its tail
pieces), `PARSE VAR`, `DROP (v)` and `PROCEDURE EXPOSE (v)` selectors, an object variable, a loop
control variable and an `INTERPRET` read reach the hook; `VALUE`, `SYMBOL`, a bare stem, a variable
reference and `.environment~novalue` do not.

**2026-10-19-variable-reference-gaps.md (closed).** Each of the item's shapes now matches the oracle.
* `>a.` on an unset caller stem: a reference uses the stem variable it names, so `variable_reference`
  (`run/call.rs`) creates it first (the same rexxref rule as above); a `USE LOCAL abc.` alias then
  reads `A.1` and the caller's `SYMBOL('a.')` is `VAR` (`t6.8/probes/p5`).
* A reference made from an alias of an object variable: `variable_reference` named the reference by
  the local spelling (`Q`), which for an `EXPOSE`-shaped alias is also the pool key, so the next
  callee read the pool's `Q`. A reference now names the variable its name resolves to, under that
  variable's own name (the `InstanceVar`'s name, or the cell's owner by `cell_owner_name`): rexxref
  intro.xml "Variable Reference Term" ("allowing the original variable to be modified"),
  utilityclasses.xml `name` ("Returns the name of the variable referenced"). `t6.8/probes/p14` now
  answers `again set through q`. This also changes, to the oracle's answers, `(>q)~name` for an alias
  of the caller's `loc` (`LOC`, was `Q`), the 88.930 kind-mismatch text (`found "LOC"`) and the `>R>`
  line (`"LOC" => "S"`) (`probes/alias_reference_names.rex`). See concern 2.

**2026-10-15-routines-table-silent-none.md and gate:phase-9c-gate.md:138 (closed; the gate line is a
pointer to the same file, `t8.5/closed.tsv`).** `build_package_string_table` propagates a failed
`StringTable` and a failed `directory_put`; the package data read after the table is made is an
`expect` (the package is read at the top and rooted). `find_written_routine` `expect`s the table a
written package holds: `routines_written` is set only by `note_routines_write`, for the package whose
`.ROUTINES` table was written, so `package_string_table` answers it at its first check. The
`rexx-bench` bench-list comments no longer begin "Added in ...". No Rexx program reaches the
failures (`StringTable` cannot be made abstract from Rexx: `.StringTable~makeAbstract` is 97.1 on
both, `probes/stringtable_make_abstract.rex`), so the witness is the unit test
`environment::tests::a_package_table_that_cannot_be_made_fails`, which marks the class abstract.

**2026-10-19-variable-reference-deviation-candidates.md (closed; Deviation candidates below).** The
crate already answers each ruled rule; the witnesses pin it.

**m1 of 2026-10-15-9b1-task-8b-review-minors.md (status line added; the file stays open).** The crate
lists the alias as `Z.`/`Z`, as ruled; pinned by `tests::the_context_variables_list_an_alias_under_its_own_name`.

**2026-10-10-variable-reference-concatenation.md: STOPPED, not changed, not closed.** See "Stopped
item" below.

## Witnesses

New `corpus/lang` programs, in `phase-9.txt`, each with its `sourceline_oracle` file (generated by
the test's driver from copies outside the repository): `compound_use_creates_stem`,
`local_novalue_hook`, `local_novalue_hook_syntax`, `variable_reference_through_alias`.
`witnesses/summary.tsv`: at base (`rexx-base-6e0bf5e5b`, built from a `git archive` tree)
`compound_use_creates_stem` differs on stdout, `local_novalue_hook` and
`variable_reference_through_alias` on stdout and stderr; at head all four are identical to the oracle
on all three descriptors. `local_novalue_hook_syntax` is identical at base too: it pins the order
(98.986 before the hook), which a hook placed before the syntax check would break. Head is
`rexx-head-2589e88b3`. I read each oracle stdout for the paths it claims: `compound_use_creates_stem`
prints `VAR` for each stem a line uses (`a LIT` before any use, `K.` never used) and `n 1 1 1 0`;
`local_novalue_hook` prints a `hook` line before s1, s2, s3, s5, s6, both selectors, s9, s10, both of
s11's names, s13, s15 (twice) and s16, none before s4, s7, s8, s12, s14 and s19, the `nilhook` and
`nothing` lines, then under `SIGNAL ON NOVALUE` `s20 NV_YYY2` with no condition and `NOVALUE cond
YYY3` after the `.nil` answer; `variable_reference_through_alias` prints
`cell LOC`, `again set through q`, `inst V`, `r= V`, `v= via r`, `refLocalStem: A.1`, `a.1 = A.1 VAR
LIT` and the two `>R>` lines.

Unit tests (`src/tests.rs` unless named): `a_drop_list_runs_left_to_right`,
`a_variable_lists_selectors_are_read_left_to_right`,
`a_procedure_expose_selectors_novalue_is_handled_before_isolation`,
`an_empty_stems_name_is_compared_by_its_characters`,
`the_context_variables_list_an_alias_under_its_own_name`,
`environment::tests::a_package_table_that_cannot_be_made_fails`. Copied into the base tree
(`witnesses/unit-tests-base-6e0bf5e5b.txt`): the pins pass at base (the crate already followed each
rule), and the table test fails there (base answers `Ok(None)`).

## Deviation candidates

**A. A variable list runs left to right.** Rule: rexxref instrc.xml, DROP "Variables are dropped from
left to right" (:1075), EXPOSE "Variables are exposed in sequence from left to right" (:1228), and
PROCEDURE EXPOSE "Variables are exposed from left to right" (:2483) and "The variables in a
subsidiary list are also exposed from left to right" (:2515); a `(name)` subsidiary list is used
where it stands. Oracle bug: `LanguageParser::processVariableList` pushes plain names on the front
of `subTerms` (`pushSubTerm`, `InstructionParser.cpp:4495`) and queues `(name)` selectors at the back
(`subTerms->queue`, `:4523`), while the instruction constructors pop the queue back to front
(`DropInstruction.cpp:68`; `initializeObjectArray`, `RexxInstruction.hpp:201-207`, for
`ExposeInstruction.cpp:62` and `ProcedureInstruction.cpp:63`), so a list runs its selectors first, in
reverse order, then its plain names. Measured: `drop list (list)` with `list = 'X'` answers `LIT LIT`
for `symbol('X') symbol('LIST')` on the oracle and `VAR LIT` here (`probes/drop_list_order.rex`);
under `::OPTIONS TRACE I`, `expose (a) (b)`, `procedure expose (a) (b)` and `drop (a) (b)` trace `B`
before `A` on the oracle and `A` before `B` here (`probes/selector_order_trace.rex`). The PROCEDURE
EXPOSE handler split follows from it: with `procedure expose c (nolist2)` under an inherited `SIGNAL
ON NOVALUE`, the oracle's handler reads `c=C d=D` (the selector ran before `c` was exposed, in the
isolated frame) and leaves the caller's `c d` at `3 4`; here it reads `c=3 d=4` and sets the caller's
to `h h` (`probes/procedure_expose_handler.rex`, the item's `t6.8/probes/p9`). Witnesses:
`a_drop_list_runs_left_to_right`, `a_variable_lists_selectors_are_read_left_to_right`,
`a_procedure_expose_selectors_novalue_is_handled_before_isolation`.

**B. Deviation 4 sentence: an empty stem's name is compared by its characters.** `USE ARG abc., >abc.`
binds where the first argument is an empty stem whose name is `ABC.`, whether that name was written
as a literal or computed. Oracle: `RexxVariable.cpp:270` compares `stem->getValue() == variableName`
as pointers, so `.stem~new('AB' || 'C.')` raises 98.995 there and binds here, while the literal
`'ABC.'` (the interned string) binds on both (`probes/stem_name_identity.rex`, the item's
`t6.8/probes/p16`). Witness: `an_empty_stems_name_is_compared_by_its_characters`.

**C. `.context~variables` lists a `USE ARG` alias under the name it is reachable by.** Rule: the
listing is of the running context's variables, and the alias is reachable there only as `Z.`/`Z`.
Oracle bug: `VariableDictionary::getAllVariables` files each value under `variable->getName()`
(`VariableDictionary.cpp:656`), the shared variable object's own name, not the key the alias is filed
under (`RexxLocalVariables::aliasVariable`, `RexxLocalVariables.cpp:544`). Measured: after `say q.`
and `call r >q.`, `r`'s `hasIndex('Z.') hasIndex('Q.') items` answers `0 1 1` on the oracle and `1 0 1`
here; the simple form `0 1 1` and `1 0 1` (`probes/context_variables_alias.rex`). Witness:
`the_context_variables_list_an_alias_under_its_own_name`.

## Stopped item: 2026-10-10-variable-reference-concatenation.md

Derived rule. A concatenation's left operand is sent the operator and answers "the concatenation of
the receiver object's string value with argument" (rexxref fundclasses.xml, Object concatenation
methods); a string value is obtained by sending `request("STRING")` (provide.xml, "Required String
Values"), and a VariableReference's `request` "Forwards to the request method of the referenced
variable" (utilityclasses.xml). So `(>e) || 'x'`, `e = .array~new(2)`, is `e~request('STRING')`
(`makeString`, the empty string) then `x`: what this crate answers, and what the item calls wrong.

Oracle departure. `VariableReference::stringValue` answers the referenced value's C++ `stringValue`
(`VariableReference.cpp:258-261`), its default name, without the `makeString` its `request` forwards
to (`:364-370`). Measured (`probes/varref_concat_matrix.rex`, `probes/varref_nostring.rex`):
`(>q)~request('STRING')` is `p` and `q` on both, yet under `SIGNAL ON NOSTRING` the oracle raises
NOSTRING (`an Array`) for `say (>q)`, so it treats a reference to an array with a string value as
having none. As a left operand, the oracle answers `an Arrayx` for `>e` and `>q`, `a Kx` for an
object whose `makeString` answers `kmake`, `a NOSTRx` for one with only a `string` method; this crate
answers `x`, `p`/`qx`, `kmakex` and `nstrx`.

This crate is not consistent with the rule either: as a right operand, a `SAY` operand or a builtin
argument it answers the oracle's default name (`'x' || (>e)` is `xan Array`, `length(>o)` is 3, on
both), and it raises no NOSTRING where the oracle does. Following the rule everywhere changes those
positions away from the oracle too, so it is a Deviation for Moritz: proposed, "the string value of a
variable reference is its referenced value's, by `request('STRING')`, in every position".

## Commands

All from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t85-target`, at `2589e88b3`;
records in `t8.5/final/`. The same set ran at `b69a1babc` (records at `t8.5/`, the suite tables
identical too).

| command | result |
|---|---|
| `cargo fmt --all --check` (`checks-fmt.txt`) | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec -p rexx-bench --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test signals --test dispatch_seam --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions` (`checks-touched.txt`) | 0: lib 1191, signals 42, dispatch_seam 6, every table binary passed |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress` (`checks-collect-stress.txt`), on an idle machine | 0, 74 passed. At `b69a1babc`, run beside callgrind, it was 73 of 74, the l0 subset on `command_redirected_target_waits_elsewhere.rex` (`guard 0 3 waited 1` under stress): the queued `2026-10-26-redirected-target-waits-timing-flake.md` (`t8.5/checks-touched.txt`) |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --features pinning --test concurrency_tests a_park_under_each_frame_kind` (`checks-pinning-frames.txt`) | 0, 1 passed (the `NovalueHook` row included) |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 101: 1200 of 1201, the one mismatch the known `rexxutil_file_date_time.rex` |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-parse --test sourceline_oracle --test errors` (`checks-parse.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test coverage` (`checks-coverage.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites` (`checks-unsafe.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-bench --lib` (`t8.5/checks-bench.txt`, at `b69a1babc`; `2589e88b3` does not touch `rexx-bench`) | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t8.5/final/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`suite/`) | 0; `tests.tsv`, `table.tsv`, `verdicts.tsv` and `oracle-differences.tsv` identical to `t8.4/r1/suite/` |

## Perf

`final/callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -j 4 -o
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.5/final2 base=.../t8.5/rexx-base-6e0bf5e5b
head=.../t8.5/rexx-head-2589e88b3`, every program, 3 rounds, the base built from a `git archive` of
`6e0bf5e5b` in its own target dir. No SPREAD or OVER flag. Libc subtracted, no program rises: every figure is -0.0000% or
below (the lowest define -0.0023%). Whole process the largest rise is hashnew +0.0147%, libc
subtracted -0.0001%.

The first full run, at `b69a1babc` (`t8.5/callgrind-b69a1babc.txt`), was over budget, libc
subtracted: parse +0.1678%, rexxcps +0.1193%, compound +0.1109%, decloop +0.0997%, sayloop
+0.0921%, decrender +0.0772%. `cgdiff.py` put it in `ops_loop_steady`, `exec_parse`, `loop_advance`
and `stem_get_at`. One-round runs of compound, decloop, sayloop and parse (`t8.5/perf-bisect/`)
separated two causes, each alone enough for the whole figure: `novalue_check` answering the read's
value (the hook's answer joined the hot path after a call that may write anything), and a third
`Novalue` variant (`try9`: base sources with only that variant added, decloop +0.1164%). Controls:
a comment-only change to `lib.rs` (`ctl1`) and the `Interp` field alone (`ctl2`) are flat; the
environment and comment changes alone (`try7`) are flat. `2589e88b3` keeps `Novalue` two-valued and
every read's return shape, sending the hook from the unset read's cold path (`try11`, then the full
run above).

## Concerns

1. **The stopped item** above needs Moritz's ruling before anything changes.
2. **A reference's name through an alias.** `(>q)~name`, the 88.930 text and the `>R>` line now
   name the variable an alias resolves to (`LOC`, `V`), as on the oracle; the Instance case needs it,
   because the name is that home's pool key. Candidate C rules the opposite way for
   `.context~variables` (the reachable name), on a different rule (the listing is the context's
   names). If Moritz wants a reference named by the name it was made through, `VarRefHome::Instance`
   needs the pool key as a field of its own (`rexx-core`), and the names then depart from the oracle.
3. **Item scope.** The stem rule fixed more than the item's `drop` shape: every compound read, a
   `VALUE`/`SYMBOL` inquiry and a stem reference now create the stem, each measured identical to the
   oracle (`probes/compound_uses.rex`).
4. **Residual, not queued: arithmetic on a variable reference.** `t = >s; say t + 0` (`s = 'abc'`)
   raises 41.1 on both, but the oracle's traceback has a `*-* Compiled method "+" with scope "String".`
   line this crate does not print (the `+` is forwarded to the value by `processUnknown`,
   `VariableReference.cpp:245-249`). Proposed owner: the tracebacks area, or a queued item.

5. **A glob `rm`.** Regenerating `witnesses/summary.tsv` I ran `rm -f` with a `*.head.*` glob in
   `t8.5/witnesses/`, against the rule; it removed only that directory's head outputs, which the same
   command wrote again.

## New oracle defects

Candidates A, B and C above, and the stopped item's `VariableReference::stringValue`. The controller
keeps `defects.json`.

## Fix round 1

Brief `task-8.5-fix-r1.md`, review `task-8.5-review.md`. Code `aeb913cff`, `074ada1da` (the second
keeps the conversion and EXPOSE paths at base cost, see Perf). Records `t8.5/r1/`; probes in
`r1/probes/` with oracle (`.o.*`), head at `074ada1da` (`.t.*`) and base `6e0bf5e5b` (`.b.*`)
outputs.

**1. Important 1: the hook's unwitnessed paths.** `corpus/lang/local_novalue_hook.rex` now reads an
unset tail of an existing stem (`a8.1 = 1; say 's3a' a8.2`, the review's p24) and runs `p5: procedure
expose c5. (c5.1)` (a reachable compound selector, p16); its `sourceline_oracle` file is regenerated.
The oracle prints `hook A8.2` / `s3a NV_A8.2` and `hook C5.1` / `s9a VAR`, and head matches on all
descriptors. Mutants from a `git archive` of `aeb913cff`: with `unset_tail`'s `if HOOK` made `if false
&& HOOK`, the witness loses `hook A8.2` and prints `s3a A8.2` (`r1/mutants/mutA-unset-tail.txt`); with
`run.rs`'s reachable selector read back to `stem_get`, it loses `hook C5.1`
(`r1/mutants/mutB-procedure-selector.txt`).

**2. Minor 1: EXPOSE and PROCEDURE EXPOSE of a stem create the stem variable** (rexxref intro.xml:2562).
`PROCEDURE EXPOSE` creates the caller's stem where the name's slot holds none, before aliasing it;
`EXPOSE` creates the object's (`bind_exposed`, through the cold `expose_object_stem`). The
`create_stem_variable` doc's rule now holds for every use the crate implements. Witness: the same
corpus program gains `p3: procedure expose c3.`, `p4: procedure expose (lst) d4.` and a method's
`expose s3.`; the oracle and head print `p3 in VAR`, `after p3 VAR`, `after p4 VAR`, `x VAR`, base
prints `LIT` on each (`r1/witnesses/compound_use_creates_stem.base.stdout`). The review's probes
`r1/probes/expose_stem.rex` (p08) and `stem_create_scopes.rex` (p05) match the oracle at head and
differ at base.

**3. Minor 2.** The two sentences now name no set size ("Each of the item's shapes", "the pins pass at
base").

**4. Minor 3.** Candidate A cites instrc.xml:1075 (DROP), :1228 (EXPOSE), :2483 and :2515 (PROCEDURE
EXPOSE and its subsidiary list).

**5. 2026-10-10-variable-reference-concatenation.md (closed, `074ada1da`).** Ruled by Moritz
2026-10-27 ("Rule everywhere, Deviation"). The required-string protocol classifies a reference by its
referenced value (`reference_conversion`, `dispatch/reqstr.rs`), an unset variable by its own name;
so `SAY`, a builtin argument and a concatenation's right operand read the referenced value's
`request('STRING')`, and NOSTRING follows that value. A concatenation whose left operand is a
reference to an object joins the reference's string value (`concat_reference`, `eval.rs`) instead of
sending the operator on. Creating a reference arms the required-string latch (`variable_reference`,
and the native API's `surface.rs`), since a reference's rendering is no longer its string value.
Every position follows the rule at no cost outside the reference path (Perf). Measured
(`r1/probes/varref_string_positions.rex`, `varref_nostring.rex`, three ways): a reference to `.array~of('p','q')`
is `p`/`q` in every position on head, `an Array` on the oracle and at base except base's left
operand; a `makeString` answer `kmake` likewise; an object with only a `STRING` method is `a NOSTR`
(the reference's `string`) in every position on head, where base's left operand sent the operator
on and answered `nstr`; `.directory` is `a Directory`. Under `SIGNAL ON NOSTRING`, head raises nothing
for `say (>q)`, `length(>q)` and `(>q) || 'x'` and raises for `say 'd' (>d)` with the reference as the
condition's object; the oracle raises on `say (>q)`, and base raises nowhere. No corpus program and no
suite row changed. Witnesses: `tests::a_references_string_value_is_its_referenced_values_in_every_position`
and `tests::a_reference_raises_nostring_only_where_its_value_has_no_string_value`; both fail at base
(`r1/witnesses/unit-tests-base-6e0bf5e5b.txt`).

**Deviation candidate D. A variable reference's string value is its referenced value's.** Rule: a
string value is what `request("STRING")` answers (rexxref provide.xml:758, "Required String Values",
which lists SAY data and builtin arguments at :750 and :747, and NOSTRING at :769); a concatenation
joins "the receiver object's string value" (fundclasses.xml:2683); a VariableReference's `request`
"Forwards to the request method of the referenced variable" (utilityclasses.xml:12631-12632). So in
every position, either operand of `||`, abuttal and blank, SAY and builtin arguments, a reference's
string value is the referenced value's `request('STRING')`, and NOSTRING is raised only where that
answers none. ORACLE BUG: `VariableReference::stringValue` answers the referenced value's C++
`stringValue()` (`VariableReference.cpp:258-261`), its default name, while `VariableReference::request`
forwards to the value's `request` (`:364-370`); the implicit conversion takes the first, so
`(>q)~request('STRING')` is the array's items while `say (>q)` prints `an Array` and, under `SIGNAL ON
NOSTRING`, raises NOSTRING. Measured: `r1/probes/varref_string_positions.rex` and
`r1/probes/varref_nostring.rex` (with `t8.5/probes/varref_concat_matrix.rex` and
`t8.5/probes/varref_nostring.rex`). Witnesses:
`tests::a_references_string_value_is_its_referenced_values_in_every_position`,
`tests::a_reference_raises_nostring_only_where_its_value_has_no_string_value`
(`rust/crates/rexx-exec/src/tests.rs`).

**Checks** at `074ada1da`, from `rust/`, `CARGO_INCREMENTAL=0`,
`CARGO_TARGET_DIR=/tmp/claude-1000/t85-target`, records `r1/final/`:

| command | result |
|---|---|
| `cargo fmt --all --check` (`checks-fmt.txt`) | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test signals --test dispatch_seam --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions` (`checks-touched.txt`) | 0: lib 1193, signals 42, dispatch_seam 6, every table binary passed |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress` (`checks-collect-stress.txt`), after the perf run | 0, 74 passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 101: 1200 of 1201, the known `rexxutil_file_date_time.rex` |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-parse --test sourceline_oracle --test errors` (`checks-parse.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test coverage` (`checks-coverage.txt`) | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t8.5/r1/final/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`suite/`) | 0; the four tables identical to `t8.5/final/suite/` |

**Perf.** `r1/final/callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -j 4 -o
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.5/r1final base=.../t8.5/rexx-base-6e0bf5e5b
head=.../t8.5/rexx-head-074ada1da`, every program, 3 rounds, no SPREAD or OVER flag. Libc
subtracted, the largest rises are fibfunc +0.0213% and fibcall +0.0209%; every other program is at
most +0.0000% or falls (intcall -0.1258%, dirread -0.0304%, alloc -0.0299%). Whole process, the
largest rise is fibcall +0.0196%.

The first full run, at `aeb913cff` (`r1/callgrind-aeb913cff.txt`), was over budget, libc
subtracted: dispatch +0.0967%, dirread +0.0758%, alloc +0.0746%, intcall +0.0502%. `cgdiff.py` put
dispatch in `bind_exposed` (+2 instructions per exposed name, the second shape test) and the others
in `classify_string_conversion` (the reference arm's lookup, inline). `074ada1da` matches the shape
once with the simple name first and moves the reference arm out of line (`reference_conversion`);
the one-round check of those programs is `r1/callgrind-r1try1.txt`, then the full run above.

## Fix round 2

Brief `task-8.5-fix-r2.md`, re-review `task-8.5-rereview-r1.md`. Code `b21493ec6`; records
`t8.5/r2/`. Probes in `r2/probes/` (the re-review's `pos.rex`, `ns2.rex`, `untrapped.rex`,
`meth.rex`, `instr.rex`, `tr.rex`, and `unit_positions.rex`, the unit test's program as a file),
each with oracle (`.o.*`), base `6e0bf5e5b` (`.b.*`) and head (`.t.*`) outputs, the run directory
normalised to `RUNDIR` in stderr.

**1. Important 1: every required-string position a reference reaches.** The behaviour is kept
(controller ruling 2026-10-27, inside "Rule everywhere"). Each position, its rexxref rule, and the
head answer for a reference whose value's `makeString` answers the string named (`unit_positions.rex`
and `variable_reference_forwarded_string.rex`; oracle and base in brackets where they differ):

| position | rexxref | head | oracle / base |
|---|---|---|---|
| right operand of a comparison with a string receiver | provide.xml:751-752 "Rexx dyadic operators when the receiving object ... is a string", :798-800 for String's comparison methods | `'kmake' = (>k)` and `==` are `1` | `0` / `0` |
| right operand of arithmetic | the same | `1 + (>m)` with `5` is `6` | `6` / 41.1 |
| String method arguments | provide.xml:807-810 "For all other methods: ... request("STRING") ... If request returns .nil, the method raises an error" | `pos` `2`, `changestr` `x-x`, `append` `abkmake`, `compareTo` `0`, `startsWith` `1`, `match` `1`; with no string value 88.909 | 88.909 on `pos`/`changestr`/`compareTo`/`startsWith`/`match` / `0 xkmakex aba KMAKE 1 0 0`, and 97.1 with no string value |
| DO count | provide.xml:727-729 "DO statements containing exprr or exprf" | `do (>m)` runs 5 times | 26.2 / 26.2 |
| INTERPRET | :739 "Instruction strings on INTERPRET instructions" | runs `say 'interpreted'` | runs `a KCODE` |
| ADDRESS command | :731 "Commands to external environments" | `exit 3`, rc 3 | rc 127 |
| PUSH | :743 "Data queue strings on PUSH and QUEUE instructions" | `push KMAKE` | `push A KMAKE` |
| compound tail | :730 "Substituted values in compound variable names" | `X.kmake` | `X.a KMAKE` |
| NUMERIC DIGITS | :740 "DIGITS, FORM, and FUZZ values on NUMERIC instructions" | `digits 5` | `5` / 26.5 |
| TRACE VALUE | :746 "Trace settings on TRACE VALUE instructions" | `trace R` | `trace A` |
| SIGNAL VALUE | :745 "Label names on SIGNAL VALUE instructions" | reaches `KMAKE:` | label `a KLABEL` not found |
| explicit concatenation message to the reference | fundclasses.xml:2683 "the receiver object's string value", the reference's being its value's | `(>e)~'||'('x')` is `x`, `(>k)~' '('y')` is `kmake y` | `an Arrayx`, `a KMAKE y` |

No position's rexxref text gives it a rule other than `request('STRING')`. Compound tails and TRACE
VALUE are in provide.xml's list (:730, :746); TRACE's own text reads only an option's first letter
(instrc.xml:3511), `R` here, which is what head answers. A TRACE VALUE answer that is a number is 24.901 outside
interactive debug, on head as on the oracle for a literal. The ruled positions of fix round 1 (either
operand of `||`, abuttal and blank, SAY, builtin arguments, PARSE VALUE and VAR) are unchanged.

Witnesses: the oracle agrees with head on arithmetic, NUMERIC DIGITS, and NOSTRING in arithmetic and
in `=`'s right operand, so those are the new corpus program `lang/variable_reference_forwarded_string.rex`
(in `phase-9.txt`, with its `sourceline_oracle` file). At base it differs on stdout and stderr (41.1,
`r2/witnesses/variable_reference_forwarded_string.base.txt`); at head it matches on all descriptors.
The others are `tests::a_references_string_value_reaches_every_required_string_position`, which fails
at base (`r2/witnesses/unit-tests-base-6e0bf5e5b.txt`, with fix round 1's tests).

**2. Important 2: block D's cause, and NOSTRING.** NOSTRING now names the referenced value
(controller ruling): a reference whose value has no string value stands for that value once the
conversion falls back (`conversion_subject`, `dispatch/reqstr.rs`), so the condition's object is the
value and its description the value's `STRING` answer, and an untrapped conversion reads that answer.
rexxref condtra.xml gives NOSTRING's ADDITIONAL as "The object that caused the NOSTRING condition"
and DESCRIPTION as "The readable string representation of the object causing the NOSTRING condition";
neither names the reference. Measured: `ns2.rex` head `n say:   NOSTRING nstr NSTR` and `d say:
NOSTRING a Directory Directory` in every position, the oracle's answer for `addR` and `cmpR`.
`a_reference_raises_nostring_only_where_its_value_has_no_string_value` now expects `NOSTRING a
Directory Directory`, and `a_references_string_value_is_its_referenced_values_in_every_position` the
value's `STRING` answer (`nstr`) for an object with no string value. Block D below replaces fix round
1's.

**Deviation candidate D (replaces fix round 1's). A variable reference's string value is its
referenced value's.** Rule: a string value is what `request("STRING")` answers (rexxref provide.xml:758,
"Required String Values", whose list :727-752 names each position below); a concatenation joins "the
receiver object's string value" (fundclasses.xml:2683); a String method's string argument is
`request("STRING")`, an error where that is `.nil` (provide.xml:807-810); a VariableReference's
`request` "Forwards to the request method of the referenced variable" (utilityclasses.xml:12631-12632).
So in every required-string position a reference's string value is the referenced value's
`request('STRING')`, NOSTRING is raised only where that answers none, and then names the referenced
value as its object and that value's `STRING` answer as its description (condtra.xml, NOSTRING).
ORACLE BUG: `VariableReference::stringValue` answers the referenced value's C++ `stringValue()`
(`VariableReference.cpp:258-261`), its default name, while `VariableReference::request` forwards to the
value's `request` (`:364-370`). The oracle takes the first, with NOSTRING raised for every reference and
naming the reference, in SAY, builtin arguments, either operand of concatenation, abuttal and blank,
PARSE VALUE and VAR, a comparison's right operand where the value has a string value, String method
arguments (where it raises 88.909), INTERPRET, a DO count, ADDRESS, PUSH, compound tails, SIGNAL VALUE,
TRACE VALUE and an explicit concatenation message. It already forwards to the value, as this crate
does, in an arithmetic operand, NUMERIC DIGITS, and a comparison's right operand where the value has
no string value (NOSTRING naming the value, `ns2.rex` `addR` and `cmpR`); a strict comparison's right
operand there names the reference (`"x" == (>d)`, `unit_positions.rex`). Measured:
`t8.5/r1/probes/varref_string_positions.rex`, `t8.5/r1/probes/varref_nostring.rex`,
`t8.5/r2/probes/pos.rex`, `ns2.rex`, `untrapped.rex`, `meth.rex`, `instr.rex`, `tr.rex`,
`unit_positions.rex`, and `t8.5/probes/varref_concat_matrix.rex`. Witnesses (`rust/crates/rexx-exec/src/tests.rs`):
`tests::a_references_string_value_is_its_referenced_values_in_every_position`,
`tests::a_references_string_value_reaches_every_required_string_position`,
`tests::a_reference_raises_nostring_only_where_its_value_has_no_string_value`; and
`rust/corpus/lang/variable_reference_forwarded_string.rex` for the positions the oracle forwards.

**3. Minor 1.** `local_novalue_hook.rex`'s header is re-wrapped (no line over 76 columns,
`awk 'length > 76'` over it finds none); its line count is unchanged and its `sourceline_oracle` file is
regenerated.

**Checks** at `b21493ec6`, from `rust/`, `CARGO_INCREMENTAL=0`,
`CARGO_TARGET_DIR=/tmp/claude-1000/t85-target`, records `r2/final/`:

| command | result |
|---|---|
| `cargo fmt --all --check` (`checks-fmt.txt`) | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test signals --test dispatch_seam --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions` (`checks-touched.txt`) | 0: lib 1194, signals 42, dispatch_seam 6, every table binary passed |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress` (`checks-collect-stress.txt`), after the perf run | 0, 74 passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 101: 1201 of 1202, the known `rexxutil_file_date_time.rex` |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-parse --test sourceline_oracle --test errors` (`checks-parse.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test coverage` (`checks-coverage.txt`) | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t8.5/r2/final/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`suite/`) | 0; the four tables identical to `t8.5/r1/final/suite/` |

**Perf.** `r2/final/callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -j 4 -o
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.5/r2final base=.../t8.5/rexx-base-6e0bf5e5b
head=.../t8.5/rexx-head-b21493ec6`, every program, 3 rounds, no SPREAD or OVER flag. Libc subtracted,
the largest rises are fibfunc +0.0213% and fibcall +0.0209%, as at `074ada1da`; every other program is
at most +0.0000% or falls (intcall -0.1258%). The new fallback runs only where a conversion found no
string value.
