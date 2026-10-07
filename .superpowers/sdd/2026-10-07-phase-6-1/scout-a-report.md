# Scout A report: disposition of the class (b) and (c) loud refusals

Tree: `be19fd06a`, built from `git worktree add /tmp/claude-1000/p61/sa/wt be19fd06a`,
`CARGO_TARGET_DIR=/tmp/claude-1000/p61/sa/target memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run`.
Ours is `target/release/rexx-run`, unswitched. Every probe ran from the empty directory
`/tmp/claude-1000/p61/sa/empty` through `run.sh` (Appendix A), ours and the oracle back to back, each
probe in its own directory. `results.txt` lines quoted below are Appendix C.

## 1. Census delta

Scripts copied out of the census appendix to `/tmp/claude-1000/p61/sa/{sites,classify,api}.py`, run from
`rust/`:

    python3 -I $S/sites.py crates/rexx-exec/src
    -> 354 sites 47 constructors                      (census: same)
    python3 -I $S/sites.py crates/rexx-exec/src list | python3 -I $S/classify.py
    -> 27 a, 6 a|b, 99 b, 5 c, 1 c|d, 116 d-guard, 6 d-licensed, 1 UNCLASSIFIED, 261 sites

The difference is line drift, not new sites. `git diff 76dd9582e be19fd06a -- rust/crates/rexx-exec/src`
adds 3 lines to `run.rs` at `:1473`, so two `PER_SITE` keys moved: `run.rs:1853` is now `:1856`
(`use_target_name`, class c) and `run.rs:3835` is now `:3838` (`.STREAM`, class b). With only those two
keys edited (`classify-head.py`):

    -> 27 a, 6 a|b, 100 b, 6 c, 1 c|d, 115 d-guard, 6 d-licensed, 261 sites   (census: identical)

C4 (`api.py`): the same refusing members, each `Phase 9`. C5 (`grep -rnoE '"Phase [0-9]+[a-z]?"' ...`):
`"Phase 5"` in `environment.rs` (1), `lib.rs` (12), `redirect.rs` (1), `run.rs` (1), as in the census.
The queued `2026-09-28-phase-5-refusal-labels` line numbers have drifted further: `.STREAM` is
`redirect.rs:652` and `run.rs:3838`.

## 2. Dispositions

Reachable = a probe in Appendix B reaches the refusal on ours. "Oracle" is what the oracle did on that
probe. Sizes: S < 50 lines, M < 300, L more. ooTest rows: C6 is the S5 whole-groups table; "also" lists
groups outside C6 whose source uses the form (grep in `ootest/`, Appendix D).

### Class (c)

| group | sites | reachable, probe | oracle | disposition | size, files | ooTest rows |
|---|---|---|---|---|---|---|
| c1 DO/LOOP `COUNTER` (every kind), `DO WITH ... OVER`, `DO x OVER stem.` | `run/loops.rs:770` via `loop_header_plan` (`:371-410`), IR fallback `:1333` | y: `c_do_counter_{ctrl,rep,forever,while,until,over}`, `c_loop_counter`, `c_loop_label_counter`, `c_do_with_{over_array,index_only,item_only,over_dir,counter}`, `c_do_over_stem_{one,three}`, all rc 120 `DO/LOOP is not implemented` | runs, rc 0; e.g. `do counter c i = 1 to 3` gives `3 4`, `DO WITH` over a Directory gives `A 1`, stem OVER over tails 1, 2, ABC gives `1 ABC 2` | IMPLEMENT. Stem OVER under deviation 1 (`phase-4-exclusions.txt:844`, order licensed), which the refusal currently contradicts | M: `run/loops.rs` (LoopState for With, counter store per iteration), IR lowering (`ir/`), `lib.rs` `instruction_owner` comment | C6: DateTime TEST_BRUTE_FORCE (`do with` over a StringTable, `DateTime.testGroup:846`), Method whole and derived (`:75`, `:179`), MethodArgs TEST_REQUEST_STRING_* (`:127`), TRACE TEST_TRACE_LABEL_WITH_FORWARD (`:1178`), TRACE_TraceObject TEST_CALLER_STACK_FRAME_REPLY_START (`:151`). Also: DO, DoWith, LOOP, LoopWith, File, Package, Routine, REQUIRES, SysFileTree |
| c2 `USE ARG` into a message or bracket term | `run.rs:1856` (`use_target_name`) | y: `c_use_arg_msg`, `c2_use_arg_bracket`, `c2_use_strict_arg_msg`, `c2_use_arg_msg_default`, `c2_use_arg_msg_method`, all rc 120 `a message send is not implemented` | assigns through `a=` / `[]=`: `x`, `x`, `x`, `dflt`, `v`, rc 0 | IMPLEMENT (queued `2026-09-28-use-arg-message-term`); reuse PARSE's `assign_expr_target` message arm | S-M: `run.rs` | also USE |
| c3 `.context~condition` | `dispatch/context.rs:322` | y: `c_context_condition` (SIGNAL ON SYNTAX), `c_context_condition_call` (CALL ON ERROR) | a Directory copy: `Directory SYNTAX 41.1`, `Directory ERROR 3` | IMPLEMENT via the existing `Interp::condition_copy` (`condition.rs:512`) | S: `dispatch/context.rs` | C6: RexxContext TESTCONDITION01 |
| c4 `CONDITION('D')` for a NOVALUE with no description | `builtin/state.rs:360` | y, only through `RAISE NOVALUE` caught in the caller: `c_condition_d_raise6`, `c_condition_d_raise7`. Every variable-read NOVALUE carries its name and agrees: `c_condition_d_{novalue,stem,var,ref,interp,expose}` | `D=[] NOVALUE`: the empty string | IMPLEMENT: answer `''` for `(None, b"NOVALUE")` like every other condition; the arm's premise ("the variable's own derived name") is already met on the read path | S: `builtin/state.rs` | none found |
| c5 `::ATTRIBUTE` on a stem or compound name | `dispatch.rs:3078` | y: `c_attr_compound` (`::attribute "A.B"`), `c_attr_stem` (`::attribute s.`) | `5`, `Stem`, rc 0 | IMPLEMENT | S-M: `dispatch.rs`, object variable pool for stem and tail | none in C6 |
| c6 `DELEGATE` to a stem or compound | `dispatch.rs:3059` | y: `c_delegate_compound`, `c_delegate_stem` | sends to the variable's value: 97.1 on `"x"` for `a.b`, `3` for `a.~length` | IMPLEMENT with c5 (same storage) | S: `dispatch.rs` | none in C6 |
| c7 `Refused::Raised` in a native conversion | `dispatch/library.rs:857` | n: `c_native_raised` (an object whose `string` raises 93.900, passed to `RxCalcSqrt`) agrees, rc 163 both. Both callers of `refusal()` match `Err(Refused::Raised)` first (`library.rs:471`, `:541`), and `grep -rn '\.refusal(' crates/rexx-exec/src` finds only `:476` and `:550` | raises the callee's condition | GUARD (d-guard) | none | none |
| c8 main program that does not parse | `lib.rs:3223` | y: `c_parse_error_main`, `c_do_counter_simple` (27.905): rc 120 `rexx-exec: 35.1: ...` | the standard error report, rc 221 / rc 229 | IMPLEMENT with b3 (one ParseError-to-SYNTAX report with the failing file's line); the obstacle is written at `lib.rs:3196-3222` (rexx-parse drops the `ProgramSource`) | M with b3: `rexx-parse` (hand the source back with the error), `lib.rs` | indirect: any group whose test expects a translation error at top level |

### Class (b)

| group | sites | reachable, probe | oracle | disposition | size, files | ooTest rows |
|---|---|---|---|---|---|---|
| b1 `OPTIONS` | `run.rs:1197`, `lib.rs:1014` (`instruction_owner`) | y: `b_options`, `b_options_expr` | evaluates and traces the expression, does nothing else, rc 0 (`OptionsInstruction.cpp`, `execute`) | IMPLEMENT | S: `run.rs`, IR op, `lib.rs`; `owners.rs` tag | also TRACE, ASSIGNMENT, DATE, SysFileTree |
| b2 `USE LOCAL` as a method's first instruction | `run.rs:1620` | y: `b_use_local_method`, `b_use_local_method_expose` | listed names local, every other name exposed: `5`, `5 9 9` | IMPLEMENT | M: `run.rs` (auto-expose on first touch) | C6: GUARD TEST_WHEN_USE_LOCAL_NO_WAIT (derived). Also VarRef, SecurityManager |
| b3 a loaded source that does not parse | `install.rs:657` (`::REQUIRES`), `install.rs:953` (`Package~new`), `lib.rs:2208` (external call), `install.rs:1778` (`Method/Routine~newFile`) | y: `b_requires_bad`, `b12_requires_trapped`, `b_package_new_bad`, `b12_package_new_bad_untrapped`, `b_external_call_bad`, `b12_method_newfile_bad_abs`, `b12_routine_newfile_bad_abs`, `b12_routine_newfile_bad_untrapped` | SYNTAX in the callee's file at its line: untrapped rc 221 with the callee line first in the traceback; trapped `35 35.1` (position `1` for newFile) | IMPLEMENT with c8; `Method~new`/`INTERPRET` already raise (`b12_method_new_bad`, `b12_interpret_bad` agree). Exclusions row `phase-4-exclusions.txt:1895-1925` and `:530` ("OWNER: Phase 5 for the rest of ::REQUIRES") are this group | M: `install.rs`, `lib.rs`, `rexx-parse` | C6: ATTRIBUTE TESTABSTRACTTWICE, CONSTANT TEST_BAD_NEGATIVE, METHOD TESTABSTRACTEXTERNAL, CALL TEST_INVALID, GUARD TEST_INVALID_OPTION_ONOFF |
| b4 a method source neither string nor array | `dispatch/class_protocol.rs:274` | y: `b_method_new_object_source`, `b7_method_new_bad_types`, `b7_routine_new_object_src`, `b7_setmethod_object_source`, `b7_run_object_source`, `b7_define_object_source` | 93.961 for `Method~new`/`Routine~new`; 93.974 for `setMethod`/`run`/`define` | IMPLEMENT | S: `dispatch/class_protocol.rs` (caller passes which of the two) | none in C6 |
| b5 a Method object whose body this crate does not hold | `dispatch/object_protocol.rs:488` (setMethod), `:728` (run), `dispatch/class_protocol.rs:1115` (enhanced), `:782` (subclass enhancing class method), plus `directives.rs:437/444/449` (`method_body`) and `dispatch/executable.rs:581/622` behind them | y: any Method object from `~method(name)`: plain `::method` (`b10_setmethod_from_plain_method`), `::constant` (`b9_setmethod_from_directive_method`, `b10_run_constant_method`, `b10_enhanced_constant_method`), `::attribute` (`b9_setmethod_from_attr_method`), `DELEGATE` (`b9_setmethod_from_delegate`), a native row on its own type (`b3_send_run_items_array_subclass`, `b6_setmethod_primitive_array_subclass`, `b6_enhanced_with_primitive`), a class method (`b7_subclass_enhancing_primitive`). `method_body` and `executable.rs:581/622` were not reached: every route to them stops at `:488`/`:728` first | runs them: `um um`, `5`, `5`, `A`, `1`, `3`, `3`, `1`, `K` | IMPLEMENT: a body for every Method object this crate hands out (directive bodies, generated accessors, constants, delegates, native rows). Once it lands, a native row on a receiver of the wrong type reaches the receiver_class guards (b12), which are the oracle-crash stand-ins | M-L: `dispatch/object_protocol.rs`, `dispatch/class_protocol.rs`, `environment/identities.rs` (`ExecutableRecord`), `directives.rs` | C6 indirect: Class TEST_CLASS_DEFINE refuses as `method "TEST1" of class "TESTDEFINE1" ... (Phase 9)`; `b6_define_array_subclass_with_primitive` and `b10_define_constant_method` show `define` with such an object ends in that same (a) refusal, so the root is likely this group (not confirmed against the test source) |
| b6 a class method built from source text | `dispatch/class_protocol.rs:777` | y: `b7_subclass_enhancing_src`, `b7_mixinclass_enhancing_src` | `7` | IMPLEMENT: compile as `:1115`'s instance path already does (`b7_subclass_enhancing_method` with a `Method~new` object agrees) | S: `dispatch/class_protocol.rs` | none in C6 |
| b7 `.context~executable` in a one-off method | `environment/identities.rs:201` | y: `b11_ctx_exec_method_new` (setMethod), `b11_ctx_exec_enhanced`, `b11_ctx_exec_floating_method` (`run(.methods[...])`), `b_identities_scope_gone` (after `define('M')`) | the Method object (`Method`) | IMPLEMENT (queued `2026-10-02-context-executable-setmethod`) | S-M: `environment/identities.rs` | TRACE_TraceObject `test_object_and_scope` (per the queued item) |
| b8 `.context~executable` in a `Routine~new` body | `dispatch/context.rs:427` | y: `b6_context_executable_routine_obj` | `Routine` | IMPLEMENT: answer the Routine object itself for a package-less routine | S: `dispatch/context.rs` | none in C6 |
| b9 `context.rs:431`, `:432`, `:449`, `:452` | same function | n: `::requires`d routine, `Package~new` routine, nested `::routine` in `Routine~new`, external call, INTERPRET in a routine, duplicate name (`b11_*`), all agree `Routine` | | GUARD | none | none |
| b10 objects in a DO header, FORWARD ARGUMENTS, RAISE ADDITIONAL | `run/loops.rs:716` (TO/BY/FOR value), `:2233` (control variable), `run.rs:2281` (FORWARD), `run/condition.rs:733` (RAISE) | y: `b_do_to_class`, `b2_do_to_class`, `b3_do_to_array`, `b2_do_by_inst`, `b3_do_by_ctx`; `b_do_ctrl_class`, `b_do_ctrl_env`, `b3_do_ctrl_inst`, `b3_do_ctrl_array`; `b3_forward_args_env`, `b3_forward_args_inst`; `b_raise_additional_env`, `b3_raise_additional_{inst,env,class,class_trace}` | DO: 97.1 from sending `+` (or `>` to the answer, `.local~"+"` answers .nil), trappable; FORWARD: MAKEARRAY's array (`10`, `2 1`); RAISE: MAKEARRAY's array for `.environment` (93.900 raised), 98.939 for a class object or an instance without MAKEARRAY | IMPLEMENT: route through the send the operators already make (`operator_message_receiver`) and `requestArray` | S-M: `run/loops.rs`, `run.rs`, `run/condition.rs` | none in C6 |
| b11 DO OVER an interpreter directory | `run/loops.rs:553` | n: `b_do_over_env`, `b_do_over_env2` (`.local`, `.environment`) agree | | GUARD | none | none |
| b12 operators on objects | `eval.rs:671`, `:855`, `:1067`, `:1163` | n: 198 probes, 9 left operands (`.array`, `.local`, `.context`, `.methods`, an instance, an array, `>zz`, `s. = .array`, `.stdout`) by 19 dyadic and 3 prefix operators (`spec-op.txt`): none refused; 196 agree | | GUARD. The 2 differing probes are concatenation with a variable reference, silent (section 4) | none | none |
| b13 receiver type guards (`receiver_class`, the constructor's 63 sites) | `dispatch/{array,array/*,collection,collection/*,hash,class_protocol,context,executable,introspection,object_protocol,package}.rs`, `dispatch.rs`, `environment.rs:1180/1183`, `environment/identities.rs:454/462` | today: only `dispatch/collection/supplier.rs:78`, through a Supplier subclass whose INIT does not forward (`b4_subclass_supplier`, `b4_subclass_supplier_noinit_avail`). The rest: borrowed native rows stop at b5 (`b2_run_*_on_inst`); `enhanced` and `copy` of every collection agree (`b5_*`, `b4_copy_collections`); WeakReference sends agree (`b_weakref_*`, `b2_run_hasmethod_on_weakref`); `package`, `scope`, `annotations`, stack frames agree (`b6_*`) | supplier subclass: SIGSEGV rc 139, 5 of 5 runs. Borrowed native rows on an instance: SIGSEGV rc 139, 5 of 5 each, for List ITEMS, Supplier ITEM, Package NAME, VariableReference NAME, StackFrame NAME, Routine CALL; Array ITEMS and Queue ITEMS answer a number that varies by run, Table ITEMS `0`, Method SCOPE empty, Class ID 91.999 | DEVIATION, owner none: an oracle-crash stand-in, recorded in `corpus/oracle-crashes.txt` (wrong-type receiver for a native row). The `Err(kind)` sites from `receiver_kind` (`no longer live`, unknown `Body::Native`) and "this crate did not build" sites: GUARD | S (relabel, record): `lib.rs` `receiver_class` owner to none, `oracle-crashes.txt` | none |
| b14 `setMethod`/`unsetMethod` OBJECT scope and EXPOSE on a receiver that is not an instance | `dispatch.rs:2082/2083`, `dispatch/object_protocol.rs:531`, `run.rs:1536` | n today: `SETMETHOD`/`RUN` on an Array or String are private and 97.2 agrees (`b_setmethod_*`, `b3_send_*`); subclasses of Directory, StringTable, WeakReference, Properties, MutableBuffer agree (`b8_*`); subclasses of String, Stem, Method, Routine, Message refuse earlier at `NEW` with `method "NEW" of class "K" is not implemented (Phase 9)` (`b8_{string,stem,method,routine,message}_*`) | those subclasses answer `42`, `42`, `3`, `ok` | REHOME Phase 9, with the `NEW` rows that block them. Row 9: "**L3-core on the host**: the full suite green against existing baselines *excluding* the groups enumerated below". If 6.1 takes subclass `NEW` instead, these become IMPLEMENT | S once NEW exists: `dispatch.rs`, `run.rs` | none in C6 |
| b15 `.Class` Setup methods | `dispatch/class_protocol.rs:849`, `:853`, `:859`, `:879`, `:883` | n: `b_define_class_method_setup`, `b_inherit_instance_methods` agree, 97.1 (removed before a program runs) | | GUARD | none | none |
| b16 `.STREAM` before `StreamClasses.orx` | `redirect.rs:652`, `run.rs:3838` | n: `c2_stream_first_clause` (LINEOUT as the first clause), `c2_address_with_stream_first` agree | | GUARD (bootstrap-only) | none | none |
| b17 embedded `.orx` that does not parse | `lib.rs:2254` | n: `rexx-lib` pins each file's sha256 (`lib.rs:2250-2252`); every probe in Appendix C ran past the library bootstrap | | GUARD | none | none |
| b18 owed `Phase 5` placeholder (the Phase 5 half of `a\|b`) | `environment.rs:363` (`owed[0]`), read at `environment.rs:527`, `dispatch/hash.rs:441`, `environment/identities.rs:536`, `dispatch/class_protocol.rs:454`, `:745`, `:1052` | n: every one of the 79 `.environment`/`.local` indexes read in its own program (`envx_*`, generated, Appendix B): only `.local['STDQUE']` refuses, naming Phase 10; also from a started activity and a REPLY continuation (`env_new_activity`, `env_reply_activity`), and through `~supplier` (`env_supplier`, STDQUE again) | | GUARD for the Phase 5 half: relabel `owed[0]` as an internal-inconsistency guard with no phase | S: `environment.rs` | none |

## 3. `closed_phases.rs` and the pins

Run in the scratch copy only (reverted after, `git checkout -- rust/crates/rexx-exec/tests/closed_phases.rs`):
`CLOSED` edited to `["Phase 5", "Phase 6", "Phase 7", "Phase 8"]`, then
`CARGO_TARGET_DIR=/tmp/claude-1000/p61/sa/target memcap 8G cargo test -j 4 -p rexx-exec --test closed_phases`:
5 passed, `no_refusal_names_a_closed_phase` failed naming
`src/dispatch/tests.rs:497`, `:1734`; `src/environment.rs:363`; `src/eval/object_operand_tests.rs:188`,
`:240`, `:337`; `src/lib.rs:482`, `:503`, `:514`, `:600`, `:608`, `:617`, `:626`, `:636`, `:646`, `:654`,
`:662`, `:1014`; `src/redirect.rs:652`; `src/run/tests/directives.rs:264`, `:270`; `src/run.rs:3838`;
`src/scheduler/tests/native.rs:301`.

What policing Phase 5 needs:

1. `"Phase 5"` in `CLOSED`, and the debt paragraph at `closed_phases.rs:31-38` deleted.
2. Every `lib.rs` constructor owner above changed per section 2 (implemented constructors deleted; GUARD
   and DEVIATION ones to `None`), `instruction_owner`'s `Options` arm (`lib.rs:1014`) removed, `owed[0]`
   relabelled (`environment.rs:363`), and both `.STREAM` sites relabelled.
3. `no_open_exclusions_row_names_a_closed_phase` passes today with Phase 5 in `CLOSED`, but
   `phase-4-exclusions.txt:530` ("OWNER: Phase 5 for the rest of ::REQUIRES. EXTERNAL was DELIVERED ...")
   passes only through the queued NC-h gap (`2026-09-29-closed-phases-owner-gaps`): "DELIVERED" there
   resolves EXTERNAL, not the Phase 5 rest, which is b3. Rewrite that row when b3 lands, and close NC-h
   before relying on the check.
4. The scan covers `src/` only. Integration tests still name Phase 5 as an owner and are not scanned:
   `tests/assertions.rs` EXEMPT rows with `unblocked_by: "Phase 5"` (12, `:192`-`:280`);
   `tests/owners.rs:154`, `:213`, `:324`, `:325` (Options and `LoopKind::With` tagged Phase 5), `:331`
   (`SPLIT_TABLE_PHASES`), and the count assertion `:468-473` (`Phase 5` tags == 1);
   `tests/bif_assertions.rs:781` and `tests/keyword_assertions.rs:668` (`PHASES` vocabulary; no row in
   `corpus/bif-exempt.txt` or `corpus/keyword-exempt.txt` uses it, `awk -F'\t' '!/^#/ {print $NF}'` gives
   MISMATCH, Phase 10, RAISED). Decide whether `closed_phases` widens to `tests/` or these are fixed by hand.

Pins of the refusal text that change with the dispositions (`git grep -nI 'not implemented (Phase 5)\|"Phase 5"\|(Phase 5)' -- rust`, plus the class (c) texts):

- `src/dispatch/tests.rs:487`, `:491` (accessor), `:497` (USE LOCAL), `:1734` (OPTIONS)
- `src/eval/object_operand_tests.rs:188` (DO header value), `:240` (DO OVER target), `:337` (control variable)
- `src/run/tests/directives.rs:264` (b4), `:270` (b6)
- `src/run/tests/loops.rs:196-228` (`DO is not implemented`, unsuffixed, for WITH and COUNTER OVER)
- `src/scheduler/tests/native.rs:301` (b4)
- `tests/spike.rs:78` (OPTIONS)
- `tests/concurrency_tests.rs`: whole-group expectations, 14 lines with `(Phase 5)` (`:2718`-`:2893`:
  the five "does not parse here" groups and USE LOCAL), 16 `DO is not implemented`, 2 CONDITION "O"
  (`:2690`, `:2697`)
- `tests/loud.rs` witnesses OPTIONS through the owners table; it loses its only row when b1 lands
- `corpus/refusal-sites.tsv`: derived from the `Loud` constructors (`tests/refusal_sites.rs`); deleting
  constructors changes it, regenerate. `corpus/introspection-arity.tsv` holds no Phase 5 refusal text
  (its two crate refusals, `:196`, `:203`, name Phase 9)
- Stale comments only, not scanned: `bench-programs/alloc4c.rex:6`, `bench-control/alloc4c-101.rex:6`
  quote `method "OF" of class "Array" is not implemented (Phase 5)`

## 4. Silent divergences met on the way (not loud, not this scout's)

- `(>zz) || 'x'` and `(>zz) 'x'` with `zz = .array~new(2)`: ours `x`, oracle `an Arrayx` / `an Array x`
  (op probes, `out-op.txt`).
- A `RAISE NOVALUE` reflected into a caller with `CALL ON ANY` enters the handler on ours; the oracle
  prints only `back` (`c_condition_d_raise7`).
- `.context~name` in a `Routine~new` body run by `~call`: ours `CALL`, oracle `r`
  (`b11_ctx_exec_routine_new_call_ctx`; queued `2026-10-01-routine-call-context-name`).
- `::class k subclass package` then `.k~new(...)~go`: ours 97.1 on `a Package` (the subclass is not
  built), oracle runs `go` (`b8_package_*`).
- `::class k subclass rexxinfo`: ours 98.909 rc 158, oracle 99.949 rc 157 (`b8_rexxinfo_*`).
- Class (a) refusals that are the same root as b5 or b14 rather than Phase 9 work: `define` with a
  Method object from `~method` (`b6_define_array_subclass_with_primitive`, `b10_define_constant_method`,
  `b10_define_attr_method`), and `NEW` on subclasses of String, Stem, Method, Routine, Message,
  VariableReference (`b8_*`; for VariableReference the oracle answers 93.967, so that one is wrong, not
  missing).

## Appendix A: harness

`run.sh`, `gen.py` (spec file of `### name` sections, `#### file: x` for extra files, to
`probes/<name>/`), as run:

```bash
#!/usr/bin/env bash
B=/tmp/claude-1000/p61/sa
OURS=$B/target/release/rexx-run
mkdir -p $B/empty
cd $B/empty || exit 1
for n in "$@"; do
  f=$B/probes/$n/p.rex
  echo "##### $n"
  if [ -n "$MODE" ]; then export REXX_SWITCH_MODE=$MODE; fi
  ( ulimit -v 4194304; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 $OURS $f >$B/o.out 2>$B/o.err ); rc=$?
  unset REXX_SWITCH_MODE
  echo "-- ours rc=$rc"; head -c 600 $B/o.out | sed 's/^/  out| /'; head -c 400 $B/o.err | sed 's/^/  err| /'
  ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx $f >$B/r.out 2>$B/r.err ); rc=$?
  echo "-- oracle rc=$rc"; head -c 600 $B/r.out | sed 's/^/  out| /'; head -c 400 $B/r.err | sed 's/^/  err| /'
done
```

`LD_LIBRARY_PATH` on ours was added after the first `c_native_raised` run, which failed on ours with
98.903 "Unable to load library rxmath"; every result in Appendix C is from the final run with it.

Oracle crash repeats (Appendix C rows marked rc 139), 5 runs each, oracle only:

    for i in 1 2 3 4 5; do ( ulimit -v 1048576; LD_LIBRARY_PATH=... rexx ../probes/$n/p.rex ); done
    b2_run_{list_items,supplier_item,package_name,varref_name,stackframe_name,routine_call}_on_inst,
    b4_subclass_supplier: 139 x5 each; b2_run_array_items_on_inst, b2_run_queue_items_on_inst: 0 x5

Final run: `./run.sh $(cat spec-c.txt spec-c2.txt spec-b.txt spec-b2.txt ... spec-env.txt | grep '^### ' | cut -c5-) env_list > out-all.txt`,
346 probes, summarised to `results.txt` (Appendix C). The op matrix ran separately into `out-op.txt`.

## Appendix B: probes

Generated sets, as run:

    # spec-b8.txt
    for pair in "string:.k~new('abc')" "stem:.k~new" "directory:.k~new" "stringtable:.k~new" \
      "method:.k~new('m', 'return 1')" "routine:.k~new('r', 'return 1')" "package:.k~new('p', 'say 1')" \
      "weakreference:.k~new(.nil)" "message:.k~new(.nil, 'X')" "properties:.k~new" "mutablebuffer:.k~new('a')" \
      "pointer:.k~new" "buffer:.k~new(1)" "variablereference:.k~new" "stackframe:.k~new" "rexxcontext:.k~new" \
      "rexxinfo:.k~new" "exception:.k~new" "regularexpression:.k~new"; do
      cls=${pair%%:*}; mk=${pair#*:}
      for v in setm setmobj expose unset; do   # body of ::method go:
        # setm:    self~setMethod('m', 'return 42'); return self~m
        # setmobj: self~setMethod('m', 'return 42', 'OBJECT'); return self~m
        # expose:  expose x; x = 3; return x
        # unset:   self~unsetMethod('m'); return 'ok'
        # program: say $mk~go / ::class k subclass $cls / ::method go / <body>
      done; done

    # spec-op.txt, op_1 .. op_198
    for l in ".array" ".local" ".context" ".methods" "(.t~new)" "(.array~of(1))" "(>zz)" "s." ".stdout"; do
      for op in "+ 1" "- 1" "* 2" "/ 2" "% 2" "// 2" "** 2" "= 1" "== 1" "\= 1" "< 1" "<= 1" ">> 1" \
                "<<= 1" "& 1" "| 0" "&& 1" "|| 'x'" "'x'"; do   # and prefixes "-" "+" "\"
        # zz = .array~new(2); s. = .array
        # signal on syntax
        # say 'R' ($l $op)
        # exit
        # syntax: say 'S' rc condition('O')~code
        # ::class t

    # envx_<E|L>_<NAME>, one per line of env_list's output (79):
    #   x = <.environment|.local>['NAME']
    #   say 'E NAME' x~class~id
    # env_list:
    #   do i over .environment~allIndexes~sort; say 'E' i; end
    #   do i over .local~allIndexes~sort; say 'L' i; end
    # env_list: 79 lines on both, `diff` empty.

Hand-written specs follow verbatim.

### spec-c.txt

```rexx
### c_do_counter_ctrl
do counter c i = 1 to 3; end; say c i
### c_do_counter_rep
do counter c 3; end; say c
### c_do_counter_simple
do counter c; say 'in' c; end; say c
### c_do_counter_forever
do counter c forever; if c = 2 then leave; end; say c
### c_do_counter_while
n = 0; do counter c while n < 2; n = n + 1; end; say c n
### c_do_counter_until
n = 0; do counter c until n = 2; n = n + 1; end; say c n
### c_loop_counter
loop counter c 2; end; say c
### c_loop_label_counter
loop label l counter c i = 1 to 5; if i = 3 then leave l; end; say c i
### c_do_counter_over
do counter c x over .array~of('a', 'b'); end; say c x
### c_do_with_over_array
do with index i item v over .array~of('a', 'b'); say i v; end
### c_do_with_index_only
do with index i over .array~of('a', 'b'); say i; end
### c_do_with_item_only
do with item v over .array~of('a', 'b'); say v; end
### c_do_with_over_dir
d = .directory~new; d~a = 1
do with index i item v over d; say i v; end
### c_do_with_counter
do counter c with index i item v over .array~of('a', 'b'); end; say c i v
### c_do_over_stem_one
s.7 = 'x'
do t over s.; say t; end
### c_do_over_stem_three
s.1 = 'x'; s.2 = 'y'; s.abc = 'z'
do t over s.; say t; end
### c_use_arg_msg
o = .t~new
call sub 'x'
exit
sub: use arg o~a; say o~a
::class t
::attribute a
### c_context_condition
signal on syntax
x = 1 + 'a'
exit
syntax:
c = .context~condition
say c~class~id c~condition c~code
### c_context_condition_call
call on error
'exit 3'
say 'after'
exit
error:
c = .context~condition
say c~class~id c~condition c~rc
return
### c_condition_d_novalue
signal on novalue
say zunsetvar
exit
novalue: say 'D=' condition('D')
### c_attr_compound
o = .t~new
o~a.b = 5
say o~a.b
::class t
::attribute "A.B"
### c_attr_stem
o = .t~new
say o~s.~class~id
::class t
::attribute s.
### c_delegate_compound
o = .t~new
say o~m
::class t
::method init
expose a.
a.b = 'x'
::method m delegate a.b
### c_delegate_stem
o = .t~new
say o~length
::class t
::method init
expose a.
a. = 'xyz'
::method length delegate a.
### c_parse_error_main
say 'before'
say (
### c_native_raised
o = .t~new
say rxcalcsqrt(o)
::class t
::method string
  raise syntax 93.900 array('boom')
::routine rxcalcsqrt external "LIBRARY rxmath RxCalcSqrt"
### c_condition_d_raise
signal on novalue
raise novalue
exit
novalue: say 'D=' condition('D') '|' condition('C')
### c_condition_d_stem
signal on novalue
say s.zz
exit
novalue: say 'D=' condition('D')
### c_condition_d_value
signal on novalue
say value('ZQ')
exit
novalue: say 'D=' condition('D')
### c_condition_d_var
signal on novalue
x = zunset + 1
exit
novalue: say 'D=' condition('D')
### c_condition_d_ref
signal on novalue
call r
exit
r: procedure; say q
novalue: say 'D=' condition('D')
### c_condition_d_interp
signal on novalue
interpret 'say zz'
exit
novalue: say 'D=' condition('D')
### c_condition_d_expose
o = .t~new; o~m
::class t
::method m
expose zz
signal on novalue
say zz
exit
novalue: say 'D=' condition('D')
### c_condition_d_raise2
signal on novalue name h
say 'start'
raise novalue
say 'not trapped'
exit
h: say 'D=['condition('D')']' condition('C')
### c_condition_d_raise3
signal on any name h
raise novalue
say 'not trapped'
exit
h: say 'D=['condition('D')']' condition('C')
### c_condition_d_raise4
call sub
say 'back'
exit
sub:
signal on novalue name h
raise novalue
say 'not trapped'
return
h: say 'D=['condition('D')']' condition('C')
return
### c_condition_d_raise5
signal on novalue name h
call sub
say 'back'
exit
sub: raise novalue
h: say 'D=['condition('D')']' condition('C')
### c_condition_d_raise6
signal on novalue name h
call sub
say 'back'
exit
sub: raise novalue return
h: say 'D=['condition('D')']' condition('C')
### c_condition_d_raise7
call on any name h
call sub
say 'back'
exit
sub: raise novalue return
h: say 'D=['condition('D')']' condition('C'); return
```

### spec-c2.txt

```rexx
### c2_use_arg_bracket
o = .array~new
call sub 'x'
say o[1]
exit
sub: use arg o[1]; return
### c2_use_strict_arg_msg
o = .t~new
call sub 'x'
say o~a
exit
sub: use strict arg o~a; return
::class t
::attribute a
### c2_use_arg_msg_default
o = .t~new
call sub
say o~a
exit
sub: use arg o~a = 'dflt'; return
::class t
::attribute a
### c2_use_arg_msg_method
say .t~new~m('v')
::class t
::attribute a
::method m
use arg self~a
return self~a
### c2_stream_first_clause
call lineout 'out.txt', 'x'
call lineout 'out.txt'
say linein('out.txt')
call sysfiledelete 'out.txt'
### c2_address_with_stream_first
address command 'echo hi' with output stream 'out2.txt'
say linein('out2.txt')
call sysfiledelete 'out2.txt'
```

### spec-b.txt

```rexx
### b_options
options 'ETMODE'
say 'ok'
### b_options_expr
x = 'NOEXMODE'
options x
say 'ok'
### b_use_local_method
o = .t~new
say o~m
::class t
::method m
use local a
a = 5
return a
### b_use_local_method_expose
o = .t~new
o~init2
say o~m o~v
::class t
::method init2
expose v
v = 1
::method v
expose v
return v
::method m
use local a
a = 5
v = 9
return a v
### b_expose_array_receiver
a = .array~new
a~setMethod('m', 'expose x; x = 3; return x')
say a~m
### b_expose_string_run
say 'abc'~run(.method~new('m', 'expose x; x = 3; return x'))
### b_expose_dir_run
d = .directory~new
say d~run(.method~new('m', 'expose x; x = 3; return x'))
### b_setmethod_array
a = .array~new
a~setMethod('m', 'return 42')
say a~m
### b_setmethod_string
s = 'abc'
s~setMethod('m', 'return 42')
say s~m
### b_setmethod_dir
d = .directory~new
d~setMethod('m', 'return 42')
say d~m
### b_setmethod_object_scope
a = .array~new
a~setMethod('m', 'return 42', 'OBJECT')
say a~m
### b_unsetmethod_array
a = .array~new
a~setMethod('m', 'return 42')
a~unsetMethod('m')
say a~hasMethod('m')
### b_weakref_value
o = .object~new
w = .WeakReference~new(o)
say w~value == o
### b_weakref_string
w = .WeakReference~new(.object~new)
say w~class~id
say w
### b_operator_class_eq
say .array = .array
### b_operator_class_plus
say .array + 1
### b_operator_env_eq
say .environment == .local
### b_operator_context_not
say \.context
### b_operator_class_and
say .array & 1
### b_operator_methods_concat
say .methods || 'x'
### b_do_header_env
do .environment; say 'x'; end
### b_do_to_class
do i = 1 to .array; end
### b_do_over_env
n = 0
do x over .local; n = n + 1; end
say n > 0
### b_do_over_env2
do x over .environment; if x == 'ARRAY' then say 'found'; end
### b_do_ctrl_class
do i = 1 to 3; say i; i = .array; end
### b_do_ctrl_env
do i = 1 to 3; say i; i = .local; end
### b_forward_args_env
call r
exit
r: forward to s arguments (.local)
s: say arg()
### b_forward_args_inst
call r
exit
r: forward to s arguments (.t~new)
s: say arg() arg(1)
::class t
::method makearray
return .array~of(1, 2)
### b_raise_additional_env
signal on syntax
raise syntax 93.900 additional (.environment)
exit
syntax: say condition('C') rc; say condition('O')~additional~class~id
### b_raise_additional_class
signal on syntax
raise syntax 93.900 additional (.array)
exit
syntax: say condition('C') rc
### b_requires_bad
#### file: bad.rex
say (
#### file: p.rex
say 'main'
::requires 'bad.rex'
### b_package_new_bad
signal on syntax
p = .package~new('x', 'say (')
say 'no'
exit
syntax: say rc condition('O')~code
### b_external_call_bad
#### file: badx.rex
say (
#### file: p.rex
call badx
say 'back'
### b_method_newfile_bad
#### file: badm.rex
say (
#### file: p.rex
signal on syntax
m = .method~newFile('badm.rex')
say 'no'
exit
syntax: say rc condition('O')~code
### b_routine_newfile_bad
#### file: badm.rex
say (
#### file: p.rex
signal on syntax
m = .routine~newFile('badm.rex')
say 'no'
exit
syntax: say rc condition('O')~code
### b_method_new_object_source
signal on syntax
m = .method~new('m', .object~new)
say 'no'
exit
syntax: say rc condition('O')~code
### b_setmethod_object_source
signal on syntax
o = .object~new
o~setMethod('m', .object~new)
say 'no'
exit
syntax: say rc condition('O')~code
### b_run_object_source
signal on syntax
say .object~new~run(.object~new)
exit
syntax: say rc condition('O')~code
### b_define_class_method_src
c = .object~subclass('K')
c~setMethod('xx', 'return 1')
say 'ok'
### b_class_method_enhanced
c = .object~subclass('K')
o = c~enhanced(.directory~of(('M', 'return 7')))
say o~m
### b_define_methods_src
c = .object~subclass('K')
c~defineMethods(.directory~of(('M', 'return 7')))
say c~new~m
### b_define_method_src
c = .object~subclass('K')
c~define('M', 'return 7')
say c~new~m
### b_define_class_method_setup
signal on syntax
.object~subclass('K')~defineClassMethod('m', .method~new('m', 'return 1'))
say 'no'
exit
syntax: say rc condition('O')~code
### b_inherit_instance_methods
signal on syntax
.object~subclass('K')~inheritInstanceMethods(.array)
say 'no'
exit
syntax: say rc condition('O')~code
### b_method_abstract
o = .t~new
signal on syntax
o~m
exit
syntax: say rc condition('O')~code
::class t
::method m abstract
### b_method_external_missing
o = .t~new
signal on syntax
o~m
exit
syntax: say rc condition('O')~code
::class t
::method m external 'LIBRARY nosuchlib nosuch'
### b_attr_abstract
o = .t~new
signal on syntax
say o~a
exit
syntax: say rc condition('O')~code
::class t
::attribute a abstract
### b_method_body_nobody
o = .t~new
say o~m
::class t
::method m
### b_stem_default_send
a. = 'dflt'
say a.~length
### b_stem_default_send2
a. = 'dflt'
say a.~items
### b_run_array_items
o = .object~new
say o~run(.array~method('ITEMS'))
### b_run_string_on_array
a = .array~of(1, 2)
say a~run(.string~method('LENGTH'))
### b_setmethod_array_put
o = .object~new
o~setMethod('PUT', .array~method('PUT'))
o~put('x', 1)
say 'survived'
### b_run_list_items
o = .object~new
say o~run(.list~method('ITEMS'))
### b_run_queue_items
o = .object~new
say o~run(.queue~method('ITEMS'))
### b_run_supplier_item
o = .object~new
say o~run(.supplier~method('ITEM'))
### b_run_table_items
o = .object~new
say o~run(.table~method('ITEMS'))
### b_run_package_name
o = .object~new
say o~run(.package~method('NAME'))
### b_run_method_source
o = .object~new
say o~run(.method~method('SOURCE'))~items
### b_run_class_id
o = .object~new
say o~run(.class~method('ID'))
### b_run_varref_name
o = .object~new
say o~run(.VariableReference~method('NAME'))
### b_run_stackframe_name
o = .object~new
say o~run(.StackFrame~method('NAME'))
### b_context_executable_setmethod
o = .t~new
say o~m
::class t
::method m
.t~setMethod('M', 'return 2', 'OBJECT')
return .context~executable~class~id
### b_identities_scope_gone
o = .t~new
say o~m
::class t
::method m
.t~define('M')
return .context~executable~class~id
### b_method_scope_on_object
o = .object~new
say o~run(.method~method('SCOPE'))
### b_annotations_on_object
o = .object~new
say o~run(.method~method('ANNOTATIONS'))
### b_stream_env
say .stream~new('x')~class~id
```

### spec-b2.txt

```rexx
### b2_ext_setmethod_array
a = .array~new
say a~go
::extension array
::method go
self~setMethod('m', 'return 42')
return self~m
### b2_ext_setmethod_string
say 'abc'~go
::extension string
::method go
self~setMethod('m', 'return 42')
return self~m
### b2_ext_setmethod_object_scope
say .array~new~go
::extension array
::method go
self~setMethod('m', 'return 42', 'OBJECT')
return self~m
### b2_ext_unsetmethod_string
say 'abc'~go
::extension string
::method go
self~unsetMethod('m')
return 'ok'
### b2_ext_expose_array
say .array~new~go
::extension array
::method go
self~setMethod('m', 'expose x; x = 3; return x')
return self~m
### b2_ext_expose_string_run
say 'abc'~go
::extension string
::method go
return self~run(.method~new('m', 'expose x; x = 3; return x'))
### b2_ext_expose_array_ext
say .array~new~go
::extension array
::method go
expose x
x = 3
return x
### b2_run_array_items_on_inst
say .t~new~go(.array~method('ITEMS'))
::class t
::method go
use arg m
return self~run(m)
### b2_run_array_put_on_inst
say .t~new~go(.array~method('PUT'), 'x', 1)
::class t
::method go
use arg m, v, i
self~run(m, 'a', v, i)
return 'survived'
### b2_run_string_length_on_array
say .array~of(1, 2)~go
::extension array
::method go
return self~run(.string~method('LENGTH'))
### b2_run_list_items_on_inst
say .t~new~go(.list~method('ITEMS'))
::class t
::method go
use arg m
return self~run(m)
### b2_run_queue_items_on_inst
say .t~new~go(.queue~method('ITEMS'))
::class t
::method go
use arg m
return self~run(m)
### b2_run_supplier_item_on_inst
say .t~new~go(.supplier~method('ITEM'))
::class t
::method go
use arg m
return self~run(m)
### b2_run_table_items_on_inst
say .t~new~go(.table~method('ITEMS'))
::class t
::method go
use arg m
return self~run(m)
### b2_run_package_name_on_inst
say .t~new~go(.package~method('NAME'))
::class t
::method go
use arg m
return self~run(m)
### b2_run_class_id_on_inst
say .t~new~go(.class~method('ID'))
::class t
::method go
use arg m
return self~run(m)
### b2_run_varref_name_on_inst
say .t~new~go(.VariableReference~method('NAME'))
::class t
::method go
use arg m
return self~run(m)
### b2_run_stackframe_name_on_inst
say .t~new~go(.StackFrame~method('NAME'))
::class t
::method go
use arg m
return self~run(m)
### b2_run_method_scope_on_inst
say .t~new~go(.method~method('SCOPE'))
::class t
::method go
use arg m
return self~run(m)
### b2_run_routine_call_on_inst
say .t~new~go(.routine~method('CALL'))
::class t
::method go
use arg m
return self~run(m)
### b2_run_hasmethod_on_weakref
w = .WeakReference~new(.object~new)
say w~hasMethod('VALUE')
say w~isA(.WeakReference) w~defaultName w~objectName
say w~instanceMethods~class~id
### b2_weakref_copy
w = .WeakReference~new(.object~new)
say w~copy~class~id
### b2_weakref_objectname_set
w = .WeakReference~new(.object~new)
w~objectName = 'x'
say w~objectName
### b2_do_to_class
signal on syntax
do i = 1 to .array; end
exit
syntax: say rc condition('O')~code
### b2_do_by_inst
do i = 1 to 3 by .t~new; end
::class t
### b2_do_for_env
do i = 1 for .environment; end
### b2_do_rep_inst
do .t~new; end
::class t
### b2_forward_args_env
call r
exit
r: forward arguments (.local) to s
s: say arg()
### b2_forward_args_inst
call r
exit
r: forward arguments (.t~new) to s
s: say arg() arg(1)
::class t
::method makearray
return .array~of(1, 2)
### b2_forward_args_weak
call r
exit
r: forward arguments (.WeakReference~new(.nil)) to s
s: say arg()
### b2_identities_scope_gone2
o = .t~new
say o~m
::class t
::method m
.t~define('M')
return .context~executable~source~items
### b2_ext_method_new_object_src
signal on syntax
m = .method~new('m', .object~new)
say 'no'
exit
syntax: say rc condition('O')~code
### b2_define_methods_src
c = .object~subclass('K')
c~defineMethods(.directory~of(('M', 'return 7')))
say c~new~m
### b2_operator_dir_plus
say .directory~new + 1
### b2_operator_env_plus
say .environment + 1
### b2_operator_inst_lt
say .t~new < 1
::class t
### b2_class_define_on_array
.array~define('ZZ', 'return 1')
say .array~new~zz
```

### spec-b3.txt

```rexx
### b3_send_setmethod_array
a = .array~new
a~send('SETMETHOD', 'm', 'return 42')
say a~m
### b3_send_setmethod_string
s = 'abc'
s~send('SETMETHOD', 'm', 'return 42')
say s~m
### b3_send_setmethod_scope_object_array
a = .array~new
a~send('SETMETHOD', 'm', 'return 42', 'OBJECT')
say a~m
### b3_send_setmethod_weak
w = .WeakReference~new(.nil)
w~send('SETMETHOD', 'm', 'return 42')
say w~m
### b3_send_setmethod_scope_object_weak
w = .WeakReference~new(.nil)
w~send('SETMETHOD', 'm', 'return 42', 'OBJECT')
say w~m
### b3_send_unsetmethod_array
a = .array~new
a~send('UNSETMETHOD', 'm')
say 'ok'
### b3_send_run_expose_array
a = .array~new
say a~send('RUN', .method~new('m', 'expose x; x = 3; return x'))
### b3_send_run_expose_string
say 'abc'~send('RUN', .method~new('m', 'expose x; x = 3; return x'))
### b3_send_run_expose_weak
say .WeakReference~new(.nil)~send('RUN', .method~new('m', 'expose x; x = 3; return x'))
### b3_send_run_expose_dir
say .directory~new~send('RUN', .method~new('m', 'expose x; x = 3; return x'))
### b3_send_run_expose_package
say .context~package~send('RUN', .method~new('m', 'expose x; x = 3; return x'))
### b3_send_run_items_array_subclass
say .a~of(1, 2, 3)~go
::class a subclass array
::method go
return self~run(.array~method('ITEMS'))
### b3_send_run_items_array
say .array~of(1, 2, 3)~send('RUN', .array~method('ITEMS'))
### b3_send_run_length_on_array
say .array~of(1, 2, 3)~send('RUN', .string~method('LENGTH'))
### b3_setmethod_primitive_same_type
a = .array~of(1, 2, 3)
a~send('SETMETHOD', 'cnt', .array~method('ITEMS'))
say a~cnt
### b3_setmethod_primitive_on_object
o = .t~new
say o~go
::class t
::method go
self~setMethod('cnt', .array~method('ITEMS'))
return self~cnt
### b3_forward_args_env
say .t~new~go
::class t
::method go
forward message 'M' arguments (.local)
::method m
return arg()
### b3_forward_args_inst
say .t~new~go
::class t
::method go
forward message 'M' arguments (.u~new)
::method m
return arg() arg(1)
::class u
::method makearray
return .array~of(1, 2)
### b3_forward_args_weak
say .t~new~go
::class t
::method go
forward message 'M' arguments (.WeakReference~new(.nil))
::method m
return arg()
### b3_do_ctrl_inst
do i = 1 to 3; say i; i = .t~new; end
::class t
### b3_do_ctrl_array
do i = 1 to 3; say i; i = .array~of(1); end
### b3_do_to_array
signal on syntax
do i = 1 to .array~of(1); end
exit
syntax: say rc condition('O')~code
### b3_do_by_ctx
signal on syntax
do i = 1 to 3 by .context; end
exit
syntax: say rc condition('O')~code
### b3_raise_additional_inst
signal on syntax
raise syntax 93.900 additional (.t~new)
exit
syntax: say condition('C') rc condition('O')~code
::class t
### b3_raise_additional_env
signal on syntax
raise syntax 93.900 additional (.environment)
exit
syntax: say condition('C') rc condition('O')~code; say condition('O')~additional~class~id
### b3_raise_additional_class
signal on syntax
raise syntax 93.900 additional (.array)
exit
syntax: say condition('C') rc condition('O')~code
### b3_raise_additional_class_trace
raise syntax 93.900 additional (.array)
```

### spec-b4.txt

```rexx
### b4_enhanced_collections
m = .directory~of(('ZZ', 'return 1'))
a = .array~of(1, 2)~enhanced(m); a~append(3); say a~items a~zz a[3] a~makearray~items
q = .queue~of(1, 2)~enhanced(m); q~queue(3); say q~items q~zz q~pull
l = .list~of(1, 2)~enhanced(m); l~append(3); say l~items l~zz l~firstItem
t = .table~new~enhanced(m); t['a'] = 1; say t~items t~zz t['a']
s = .stringtable~new~enhanced(m); s['a'] = 1; say s~items s~zz
d = .directory~new~enhanced(m); d['a'] = 1; say d~items d~zz
r = .relation~new~enhanced(m); r['a'] = 1; say r~items r~zz
b = .bag~new~enhanced(m); b~put(1); say b~items b~zz
st = .set~new~enhanced(m); st~put(1); say st~items st~zz
ss = .supplier~new(.array~of(1), .array~of(1))~enhanced(m); say ss~item ss~zz
cq = .circularqueue~new(3)~enhanced(m); cq~queue(1); say cq~items cq~zz
p = .properties~new~enhanced(m); p['a'] = 1; say p~items p~zz
### b4_copy_collections
a = .array~of(1, 2)~copy; a~append(3); say a~items
q = .queue~of(1, 2)~copy; q~queue(3); say q~items
l = .list~of(1, 2)~copy; l~append(3); say l~items
t = .table~new; t['a'] = 1; t = t~copy; t['b'] = 2; say t~items
s = .supplier~new(.array~of(1), .array~of(1))~copy; say s~item
r = .relation~new; r['a'] = 1; r = r~copy; say r~items
cq = .circularqueue~new(3); cq~queue(1); cq = cq~copy; say cq~items
p = .properties~new; p['a'] = 1; p = p~copy; say p~items
w = .WeakReference~new(.nil)~copy; say w~value
### b4_subclass_no_init
say .a~new~append(1) .q~new~queue(1) .l~new~append(1)
x = .a~new; x~append(5); say x~items
y = .q~new; y~queue(5); say y~items
z = .l~new; z~append(5); say z~items
u = .t~new; u['a'] = 1; say u~items
::class a subclass array
::method init
::class q subclass queue
::method init
::class l subclass list
::method init
::class t subclass table
::method init
### b4_subclass_supplier
s = .s~new(.array~of(1), .array~of('i'))
say s~item s~index
::class s subclass supplier
::method init
### b4_subclass_supplier_noinit_avail
s = .s~new(.array~of(1), .array~of('i'))
say s~available
::class s subclass supplier
::method init
### b4_package_new_methods
p = .package~new('x', 'say 1')
say p~name p~local~class~id
### b4_mixin_array
o = .k~new
o~append(1)
say o~items
::class k subclass object inherit arraymix
::class arraymix mixinclass array
### b4_mixin_list
o = .k~new
o~append(1)
say o~items
::class k subclass object inherit lm
::class lm mixinclass list
### b4_varref_on_stem_default
a. = .array~new
say a.~items
### b4_reference_methods
x = 5
r = >x
say r~name r~value
r~value = 6
say x
```

### spec-b5.txt

```rexx
### b5_enhanced_array
m = .directory~of(('ZZ', 'return 1'))
a = .array~enhanced(m); a~append(3); say a~items a~zz a[1]
### b5_enhanced_queue
m = .directory~of(('ZZ', 'return 1'))
q = .queue~enhanced(m); q~queue(3); say q~items q~zz q~pull
### b5_enhanced_list
m = .directory~of(('ZZ', 'return 1'))
l = .list~enhanced(m); l~append(3); say l~items l~zz l~firstItem
### b5_enhanced_table
m = .directory~of(('ZZ', 'return 1'))
t = .table~enhanced(m); t['a'] = 1; say t~items t~zz t['a']
### b5_enhanced_supplier
m = .directory~of(('ZZ', 'return 1'))
s = .supplier~enhanced(m, .array~of(1), .array~of(2)); say s~item s~index s~zz
### b5_enhanced_directory
m = .directory~of(('ZZ', 'return 1'))
d = .directory~enhanced(m); d['a'] = 1; say d~items d~zz
### b5_enhanced_stringtable
m = .directory~of(('ZZ', 'return 1'))
d = .stringtable~enhanced(m); d['a'] = 1; say d~items d~zz
### b5_enhanced_circ
m = .directory~of(('ZZ', 'return 1'))
d = .circularqueue~enhanced(m, 2); d~queue(1); say d~items d~zz
```

### spec-b6.txt

```rexx
### b6_routine_sources_call
r1 = .routine~new('r', 'return 1')
say r1~call r1~source~items r1~package~class~id
r2 = .context~package~findRoutine('PR')
say r2~call r2~source~items r2~package~name~right(10)
r3 = .routines['PR']
say r3 = .nil
r4 = .context~package~routines['PR']
say r4~call
r5 = .context~package~publicRoutines['PR']
say r5~call
::routine pr public
return 'pr'
### b6_routine_borrow_call
r = .routine~new('r', 'return 1')
say .t~new~go(.routine~method('CALL'), r)
::class t
::method go
use arg m, r
return r~run(m)
### b6_context_executable_in_routine
say rr()
call ir
say .context~executable~class~id
interpret 'say .context~executable~class~id'
exit
ir: say .context~executable~class~id; return
::routine rr
return .context~executable~class~id .context~executable~name
### b6_context_executable_routine_obj
r = .routine~new('r', 'return .context~executable~class~id')
say r~call
### b6_context_executable_newfile
#### file: rf.rex
return .context~executable~class~id
#### file: p.rex
r = .routine~newFile('rf.rex')
say r~call
call rf
say result
### b6_method_scope_annotations
m = .method~new('m', 'return 1')
say m~scope m~annotations~class~id
a = .array~method('ITEMS')
say a~scope a~annotations~class~id
say .t~method('X')~scope .t~method('X')~annotations~class~id
r = .routine~new('r', 'return 1')
say r~annotations~class~id
::class t
::method x
### b6_package_addclass
p1 = .context~package; p1~addClass('K1', .object~subclass('K1')); say 'p1'
p2 = .package~new('x', 'say 1'); p2~addClass('K2', .object~subclass('K2')); say 'p2'
p3 = .routine~new('r', 'return 1')~package; say p3~class~id; p3~addClass('K3', .object~subclass('K3')); say 'p3'
p4 = .method~new('m', 'return 1')~package; say p4~class~id
p5 = .object~package; say p5~name
signal on syntax
p5~addClass('K5', .object~subclass('K5'))
exit
syntax: say rc condition('O')~code
### b6_package_local_name
say .method~new('m', 'return 1')~package~local~class~id
say .routine~new('r', 'return 1')~package~name
say .object~package~name .object~package~local~class~id
say .array~method('ITEMS')~package
### b6_hash_native_puts
.methods~put(1, 'A'); say .methods~items
### b6_hash_native_routines_put
.routines~put(1, 'A'); say .routines~items
### b6_hash_native_classes_put
.context~package~classes~put(1, 'A'); say .context~package~classes~items
### b6_hash_native_publicclasses_put
.context~package~publicClasses~put(1, 'A'); say 'ok'
### b6_hash_native_resources
.resources~put(1, 'A'); say .resources~items
### b6_hash_native_remove_empty
.methods~empty; say .methods~items
### b6_hash_native_copy
c = .methods~copy; c~put(1, 'A'); say c~items
### b6_subclass_array_setmethod
say .a~of(1, 2)~go
::class a subclass array
::method go
self~setMethod('m', 'return 42')
return self~m
### b6_subclass_array_setmethod_object
say .a~of(1, 2)~go
::class a subclass array
::method go
self~setMethod('m', 'return 42', 'OBJECT')
return self~m
### b6_subclass_array_expose
say .a~of(1, 2)~go
::class a subclass array
::method go
expose x
x = 3
return x
### b6_define_with_primitive
c = .object~subclass('K')
c~define('CNT', .array~method('ITEMS'))
say 'defined'
### b6_define_methods_with_primitive
c = .object~subclass('K')
c~defineMethods(.directory~of(('CNT', .array~method('ITEMS'))))
say 'defined'
### b6_define_array_subclass_with_primitive
c = .array~subclass('K')
c~define('CNT', .array~method('ITEMS'))
o = c~of(1, 2)
say o~cnt
### b6_enhanced_with_primitive
c = .array~subclass('K')
o = c~enhanced(.directory~of(('CNT', .array~method('ITEMS'))))
o~append(1)
say o~cnt
### b6_setmethod_primitive_array_subclass
say .a~of(1, 2, 3)~go
::class a subclass array
::method go
self~setMethod('CNT', .array~method('ITEMS'))
return self~cnt
### b6_define_class_method_primitive
c = .object~subclass('K')
c~class~define('ZZ', .class~method('ID'))
say 'x'
### b6_stackframes
call r
exit
r:
f = .context~stackFrames
do x over f; say x~name x~type x~line x~objectName; end
```

### spec-b7.txt

```rexx
### b7_subclass_enhancing_src
c = .object~subclass('K', .class, .directory~of(('M', 'return 7')))
say c~m
### b7_subclass_enhancing_method
c = .object~subclass('K', .class, .directory~of(('M', .method~new('m', 'return 7'))))
say c~m
### b7_subclass_enhancing_primitive
c = .object~subclass('K', .class, .directory~of(('M', .class~method('ID'))))
say c~m
### b7_mixinclass_enhancing_src
c = .object~mixinClass('K', .class, .directory~of(('M', 'return 7')))
say c~m
### b7_method_new_bad_types
signal on syntax
m = .method~new('m', 5)
say m~source~items
m = .method~new('m', .array~new)
say m~source~items
m = .routine~new('r', .object~new)
exit
syntax: say rc condition('O')~code
### b7_routine_new_object_src
signal on syntax
m = .routine~new('r', .object~new)
say 'no'
exit
syntax: say rc condition('O')~code
### b7_setmethod_object_source
say .t~new~go
::class t
::method go
signal on syntax
self~setMethod('m', .object~new)
return 'no'
syntax: return rc condition('O')~code
### b7_run_object_source
say .t~new~go
::class t
::method go
signal on syntax
return self~run(.object~new)
syntax: return rc condition('O')~code
### b7_define_object_source
signal on syntax
.object~subclass('K')~define('M', .object~new)
say 'no'
exit
syntax: say rc condition('O')~code
### b7_setmethod_scope_object_dir
d = .directory~new
d~setMethod('m', 'return 42', 'OBJECT')
say d~m
### b7_setmethod_scope_object_inst
say .t~new~go
::class t
::method go
self~setMethod('m', 'return 42', 'OBJECT')
return self~m
```

### spec-b9.txt

```rexx
### b9_constant_send
say .t~c .t~new~c
::class t
::constant c 5
### b9_constant_expr
say .t~c .t~new~c
::class t
::constant c (2 + 3)
### b9_attribute_get_set
o = .t~new; o~a = 4; say o~a
::class t
::attribute a get
::attribute a set
### b9_attribute_get_code
o = .t~new; say o~a
::class t
::attribute a get
return 9
### b9_method_attribute
o = .t~new; o~a = 4; say o~a
::class t
::method a attribute
### b9_attribute_class
.t~a = 3; say .t~a
::class t
::attribute a class
### b9_constant_no_value
say .t~c
::class t
::constant c
### b9_floating_constant
say .methods['C'] 
::constant c 5
### b9_floating_attribute
::attribute a
### b9_method_from_methods
m = .methods['M']
say .t~new~go(m)
::method m
return 'floated'
::class t
::method go
use arg m
return self~run(m)
### b9_setmethod_from_directive_method
o = .t~new
say o~go(.u~method('C'))
::class t
::method go
use arg m
self~setMethod('cc', m)
return self~cc
::class u
::constant c 5
### b9_setmethod_from_attr_method
o = .t~new
say o~go(.u~method('A'))
::class t
::method go
use arg m
self~setMethod('aa', m)
return self~aa
::class u
::attribute a
### b9_setmethod_from_delegate
o = .t~new
say o~go(.u~method('LENGTH'))
::class t
::method init
expose a
a = 'xyz'
::method go
use arg m
self~setMethod('length', m)
return self~length
::class u
::method length delegate a
```

### spec-b10.txt

```rexx
### b10_setmethod_from_plain_method
o = .t~new
say o~go(.u~method('M'))
::class t
::method go
use arg m
self~setMethod('mm', m)
return self~mm self~run(m)
::class u
::method m
return 'um'
### b10_run_constant_method
say .t~new~go(.u~method('C'))
::class t
::method go
use arg m
return self~run(m)
::class u
::constant c 5
### b10_enhanced_constant_method
o = .object~enhanced(.directory~of(('CC', .u~method('C'))))
say o~cc
::class u
::constant c 5
### b10_define_constant_method
c = .object~subclass('K')
c~define('CC', .u~method('C'))
say c~new~cc
::class u
::constant c 5
### b10_define_attr_method
c = .object~subclass('K')
c~define('AA', .u~method('A'))
say c~new~aa
::class u
::attribute a
```

### spec-b11.txt

```rexx
### b11_ctx_exec_requires
#### file: lib.rex
::routine lr public
return .context~executable~class~id
::routine lpriv
return .context~executable~class~id
#### file: p.rex
say lr()
::requires 'lib.rex'
### b11_ctx_exec_package_new
p = .package~new('x', .array~of('::routine r public', 'return .context~executable~class~id'))
say p~findRoutine('R')~call
### b11_ctx_exec_routine_new_nested
r = .routine~new('r', .array~of('return inner()', '::routine inner', 'return .context~executable~class~id'))
say r~call
### b11_ctx_exec_external_call
#### file: ext.rex
return .context~executable~class~id
#### file: p.rex
call ext
say result
### b11_ctx_exec_interpret_in_routine
say rr()
::routine rr
interpret 'x = .context~executable~class~id'
return x
### b11_ctx_exec_method_new
say .t~new~go
::class t
::method go
self~setMethod('mm', .method~new('mm', 'return .context~executable~class~id'))
return self~mm
### b11_ctx_exec_enhanced
o = .object~enhanced(.directory~of(('MM', 'return .context~executable~class~id')))
say o~mm
### b11_ctx_exec_floating_method
say .t~new~go
::method fm
return .context~executable~class~id
::class t
::method go
return self~run(.methods['FM'])
### b11_ctx_exec_dup_routine
say rr()
::routine rr
return .context~executable~class~id
### b11_ctx_exec_routine_new_call_ctx
r = .routine~new('r', 'return .context~package~name .context~name')
say r~call
```

### spec-b12.txt

```rexx
### b12_method_newfile_bad_abs
#### file: badm.rex
say (
#### file: p.rex
signal on syntax
parse source . . me
m = .method~newFile(me~left(me~lastpos('/'))'badm.rex')
say 'no'
exit
syntax: say rc condition('O')~code condition('O')~position
### b12_routine_newfile_bad_abs
#### file: badm.rex
say (
#### file: p.rex
signal on syntax
parse source . . me
m = .routine~newFile(me~left(me~lastpos('/'))'badm.rex')
say 'no'
exit
syntax: say rc condition('O')~code condition('O')~position
### b12_routine_newfile_bad_untrapped
#### file: badm.rex
say 1
say (
#### file: p.rex
parse source . . me
m = .routine~newFile(me~left(me~lastpos('/'))'badm.rex')
### b12_package_new_bad_untrapped
p = .package~new('x', .array~of('say 1', 'say ('))
### b12_requires_trapped
#### file: bad.rex
say (
#### file: p.rex
say 'main'
::requires 'bad.rex'
### b12_interpret_bad
signal on syntax
interpret 'say ('
exit
syntax: say rc condition('O')~code
### b12_method_new_bad
signal on syntax
m = .method~new('m', 'say (')
exit
syntax: say rc condition('O')~code
```

### spec-env.txt

```rexx
### env_new_activity
m = .t~new~start('go')
say m~result
::class t
::method go
s = ''
do i over .local~allIndexes~sort
  if i \= 'STDQUE' then s = s i':'.local[i]~class~id
end
return s
### env_reply_activity
say .t~new~go
call syssleep 0.2
::class t
::method go
reply 'r'
s = ''
do i over .local~allIndexes~sort
  if i \= 'STDQUE' then s = s i':'.local[i]~class~id
end
say s
### env_supplier
s = .environment~supplier
n = 0
do while s~available; x = s~item; n = n + 1; s~next; end
say n > 0
s = .local~supplier
do while s~available; if s~index \= 'STDQUE' then x = s~item; s~next; end
say 'ok'
### env_items_makearray
say .environment~items > 0 .environment~makearray~items > 0
say .local~items
```

## Appendix C: results, final run

One line per probe: name, `same` when ours and the oracle print identical stdout, stderr and rc, else `DIFF`, then each side's rc and its last error line (or its stdout, cut).

```
c_do_counter_ctrl	DIFF	ours rc 120: rexx-exec: DO is not implemented	oracle rc 0: 3 4
c_do_counter_rep	DIFF	ours rc 120: rexx-exec: DO is not implemented	oracle rc 0: 3
c_do_counter_simple	DIFF	ours rc 120: rexx-exec: 27.905: COUNTER keyword not allowed on a simple DO instruction.	oracle rc 229: Error 27.905:  COUNTER keyword not allowed on a simple DO instruction.
c_do_counter_forever	DIFF	ours rc 120: rexx-exec: DO is not implemented	oracle rc 0: 2
c_do_counter_while	DIFF	ours rc 120: rexx-exec: DO is not implemented	oracle rc 0: 2 2
c_do_counter_until	DIFF	ours rc 120: rexx-exec: DO is not implemented	oracle rc 0: 2 2
c_loop_counter	DIFF	ours rc 120: rexx-exec: LOOP is not implemented	oracle rc 0: 2
c_loop_label_counter	DIFF	ours rc 120: rexx-exec: LOOP is not implemented	oracle rc 0: 3 3
c_do_counter_over	DIFF	ours rc 120: rexx-exec: DO is not implemented	oracle rc 0: 2 b
c_do_with_over_array	DIFF	ours rc 120: rexx-exec: DO is not implemented	oracle rc 0: 1 a / 2 b
c_do_with_index_only	DIFF	ours rc 120: rexx-exec: DO is not implemented	oracle rc 0: 1 / 2
c_do_with_item_only	DIFF	ours rc 120: rexx-exec: DO is not implemented	oracle rc 0: a / b
c_do_with_over_dir	DIFF	ours rc 120: rexx-exec: DO is not implemented	oracle rc 0: A 1
c_do_with_counter	DIFF	ours rc 120: rexx-exec: DO is not implemented	oracle rc 0: 2 2 b
c_do_over_stem_one	DIFF	ours rc 120: rexx-exec: DO is not implemented	oracle rc 0: 7
c_do_over_stem_three	DIFF	ours rc 120: rexx-exec: DO is not implemented	oracle rc 0: 1 / ABC / 2
c_use_arg_msg	DIFF	ours rc 120: rexx-exec: a message send is not implemented	oracle rc 0: x
c_context_condition	DIFF	ours rc 120: rexx-exec: CONDITION option "O" answers a Directory, which is not implemented	oracle rc 0: Directory SYNTAX 41.1
c_context_condition_call	DIFF	ours rc 120: rexx-exec: CONDITION option "O" answers a Directory, which is not implemented	oracle rc 0: Directory ERROR 3 / after
c_condition_d_novalue	same	ours rc 0: D= ZUNSETVAR	oracle rc 0: D= ZUNSETVAR
c_attr_compound	DIFF	ours rc 120: rexx-exec: a generated accessor for the attribute "A.B" is not implemented	oracle rc 0: 5
c_attr_stem	DIFF	ours rc 120: rexx-exec: a generated accessor for the attribute "S." is not implemented	oracle rc 0: Stem
c_delegate_compound	DIFF	ours rc 120: rexx-exec: a DELEGATE to the variable "A.B" is not implemented	oracle rc 159: Error 97.1:  Object "x" does not understand message "M".
c_delegate_stem	DIFF	ours rc 120: rexx-exec: a DELEGATE to the variable "A." is not implemented	oracle rc 0: 3
c_parse_error_main	DIFF	ours rc 120: rexx-exec: 35.1: Invalid expression.	oracle rc 221: Error 35.1:  Incorrect expression detected at "(".
c_native_raised	same	ours rc 163: Error 93.900:  boom.	oracle rc 163: Error 93.900:  boom.
c_condition_d_raise	same	ours rc 0: 	oracle rc 0: 
c_condition_d_stem	same	ours rc 0: D= S.ZZ	oracle rc 0: D= S.ZZ
c_condition_d_value	same	ours rc 0: ZQ	oracle rc 0: ZQ
c_condition_d_var	same	ours rc 0: D= ZUNSET	oracle rc 0: D= ZUNSET
c_condition_d_ref	same	ours rc 0: D= Q	oracle rc 0: D= Q
c_condition_d_interp	same	ours rc 0: D= ZZ	oracle rc 0: D= ZZ
c_condition_d_expose	same	ours rc 0: D= ZZ	oracle rc 0: D= ZZ
c_condition_d_raise2	same	ours rc 0: start	oracle rc 0: start
c_condition_d_raise3	same	ours rc 0: 	oracle rc 0: 
c_condition_d_raise4	same	ours rc 0: 	oracle rc 0: 
c_condition_d_raise5	same	ours rc 0: 	oracle rc 0: 
c_condition_d_raise6	DIFF	ours rc 120: rexx-exec: CONDITION option "D" answers the NOVALUE variable's name, which is not implemented	oracle rc 0: D=[] NOVALUE
c_condition_d_raise7	DIFF	ours rc 120: rexx-exec: CONDITION option "D" answers the NOVALUE variable's name, which is not implemented	oracle rc 0: back
c2_use_arg_bracket	DIFF	ours rc 120: rexx-exec: a message send is not implemented	oracle rc 0: x
c2_use_strict_arg_msg	DIFF	ours rc 120: rexx-exec: a message send is not implemented	oracle rc 0: x
c2_use_arg_msg_default	DIFF	ours rc 120: rexx-exec: a message send is not implemented	oracle rc 0: dflt
c2_use_arg_msg_method	DIFF	ours rc 120: rexx-exec: a message send is not implemented	oracle rc 0: v
c2_stream_first_clause	same	ours rc 0: x	oracle rc 0: x
c2_address_with_stream_first	same	ours rc 0: hi	oracle rc 0: hi
b_options	DIFF	ours rc 120: rexx-exec: OPTIONS is not implemented (Phase 5)	oracle rc 0: ok
b_options_expr	DIFF	ours rc 120: rexx-exec: OPTIONS is not implemented (Phase 5)	oracle rc 0: ok
b_use_local_method	DIFF	ours rc 120: rexx-exec: USE LOCAL in a ::METHOD body is not implemented (Phase 5)	oracle rc 0: 5
b_use_local_method_expose	DIFF	ours rc 120: rexx-exec: USE LOCAL in a ::METHOD body is not implemented (Phase 5)	oracle rc 0: 5 9 9
b_expose_array_receiver	same	ours rc 159: Error 97.2:  Object "an Array" cannot accept private message "SETMETHOD" from this context.	oracle rc 159: Error 97.2:  Object "an Array" cannot accept private message "SETMETHOD" from this context.
b_expose_string_run	same	ours rc 159: Error 97.2:  Object "abc" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "abc" cannot accept private message "RUN" from this context.
b_expose_dir_run	same	ours rc 0: The NIL object	oracle rc 0: The NIL object
b_setmethod_array	same	ours rc 159: Error 97.2:  Object "an Array" cannot accept private message "SETMETHOD" from this context.	oracle rc 159: Error 97.2:  Object "an Array" cannot accept private message "SETMETHOD" from this context.
b_setmethod_string	same	ours rc 159: Error 97.2:  Object "abc" cannot accept private message "SETMETHOD" from this context.	oracle rc 159: Error 97.2:  Object "abc" cannot accept private message "SETMETHOD" from this context.
b_setmethod_dir	same	ours rc 0: 42	oracle rc 0: 42
b_setmethod_object_scope	same	ours rc 159: Error 97.2:  Object "an Array" cannot accept private message "SETMETHOD" from this context.	oracle rc 159: Error 97.2:  Object "an Array" cannot accept private message "SETMETHOD" from this context.
b_unsetmethod_array	same	ours rc 159: Error 97.2:  Object "an Array" cannot accept private message "SETMETHOD" from this context.	oracle rc 159: Error 97.2:  Object "an Array" cannot accept private message "SETMETHOD" from this context.
b_weakref_value	same	ours rc 0: 1	oracle rc 0: 1
b_weakref_string	same	ours rc 0: WeakReference / a WeakReference	oracle rc 0: WeakReference / a WeakReference
b_operator_class_eq	same	ours rc 0: 1	oracle rc 0: 1
b_operator_class_plus	same	ours rc 159: Error 97.1:  Object "The Array class" does not understand message "+".	oracle rc 159: Error 97.1:  Object "The Array class" does not understand message "+".
b_operator_env_eq	same	ours rc 0: 0	oracle rc 0: 0
b_operator_context_not	same	ours rc 159: Error 97.1:  Object "a RexxContext" does not understand message "\".	oracle rc 159: Error 97.1:  Object "a RexxContext" does not understand message "\".
b_operator_class_and	same	ours rc 159: Error 97.1:  Object "The Array class" does not understand message "&".	oracle rc 159: Error 97.1:  Object "The Array class" does not understand message "&".
b_operator_methods_concat	same	ours rc 0: .METHODSx	oracle rc 0: .METHODSx
b_do_header_env	same	ours rc 230: Error 26.2:  Value of repetition count expression in DO or LOOP instruction must be zero or a positive whole number; found "The Environment Directory".	oracle rc 230: Error 26.2:  Value of repetition count expression in DO or LOOP instruction must be zero or a positive whole number; found "The Environment Directory".
b_do_to_class	DIFF	ours rc 120: rexx-exec: a class object as a DO header's TO value is not implemented (Phase 5)	oracle rc 159: Error 97.1:  Object "The Array class" does not understand message "+".
b_do_over_env	same	ours rc 0: 1	oracle rc 0: 1
b_do_over_env2	same	ours rc 0: found	oracle rc 0: found
b_do_ctrl_class	DIFF	ours rc 120: rexx-exec: a class object as a controlled DO's control variable is not implemented (Phase 5)	oracle rc 159: Error 97.1:  Object "The Array class" does not understand message "+".
b_do_ctrl_env	DIFF	ours rc 120: rexx-exec: an instance of a user class as a controlled DO's control variable is not implemented (Phase 5)	oracle rc 159: Error 97.1:  Object "The NIL object" does not understand message ">".
b_forward_args_env	DIFF	ours rc 120: rexx-exec: 35.1: Invalid expression.	oracle rc 221: Error 35.1:  Incorrect expression detected at "S".
b_forward_args_inst	DIFF	ours rc 120: rexx-exec: 35.1: Invalid expression.	oracle rc 221: Error 35.1:  Incorrect expression detected at "S".
b_raise_additional_env	DIFF	ours rc 120: rexx-exec: an instance of a user class as a RAISE ADDITIONAL value is not implemented (Phase 5)	oracle rc 0: SYNTAX 93 / Array
b_raise_additional_class	DIFF	ours rc 120: rexx-exec: a class object as a RAISE ADDITIONAL value is not implemented (Phase 5)	oracle rc 0: SYNTAX 98
b_requires_bad	DIFF	ours rc 120: rexx-exec: /tmp/claude-1000/p61/sa/probes/b_requires_bad/bad.rex does not parse here: 35.1: Invalid expression. is not implemented (Phase 5)	oracle rc 221: Error 35.1:  Incorrect expression detected at "(".
b_package_new_bad	DIFF	ours rc 120: rexx-exec: x does not parse here: 35.1: Invalid expression. is not implemented (Phase 5)	oracle rc 0: 35 35.1
b_external_call_bad	DIFF	ours rc 120: rexx-exec: /tmp/claude-1000/p61/sa/probes/b_external_call_bad/badx.rex does not parse here: 35.1: Invalid expression. is not implemented (Phase 5)	oracle rc 221: Error 35.1:  Incorrect expression detected at "(".
b_method_newfile_bad	same	ours rc 0: 3 3.1	oracle rc 0: 3 3.1
b_routine_newfile_bad	same	ours rc 0: 3 3.1	oracle rc 0: 3 3.1
b_method_new_object_source	DIFF	ours rc 120: rexx-exec: a method source that is neither a string nor an array is not implemented (Phase 5)	oracle rc 0: 93 93.961
b_setmethod_object_source	same	ours rc 0: 97 97.2	oracle rc 0: 97 97.2
b_run_object_source	same	ours rc 0: 97 97.2	oracle rc 0: 97 97.2
b_define_class_method_src	same	ours rc 159: Error 97.2:  Object "The K class" cannot accept private message "SETMETHOD" from this context.	oracle rc 159: Error 97.2:  Object "The K class" cannot accept private message "SETMETHOD" from this context.
b_class_method_enhanced	same	ours rc 0: 7	oracle rc 0: 7
b_define_methods_src	DIFF	ours rc 120: rexx-exec: method "M" of class "K" is not implemented (Phase 9)	oracle rc 0: 7
b_define_method_src	DIFF	ours rc 120: rexx-exec: method "M" of class "K" is not implemented (Phase 9)	oracle rc 0: 7
b_define_class_method_setup	same	ours rc 0: 97 97.1	oracle rc 0: 97 97.1
b_inherit_instance_methods	same	ours rc 0: 97 97.1	oracle rc 0: 97 97.1
b_method_abstract	same	ours rc 0: 93 93.965	oracle rc 0: 93 93.965
b_method_external_missing	same	ours rc 158: Error 98.903:  Unable to load library "nosuchlib".	oracle rc 158: Error 98.903:  Unable to load library "nosuchlib".
b_attr_abstract	same	ours rc 0: 93 93.965	oracle rc 0: 93 93.965
b_method_body_nobody	same	ours rc 165: Error 91.999:  Message "M" did not return a result.	oracle rc 165: Error 91.999:  Message "M" did not return a result.
b_stem_default_send	same	ours rc 0: 4	oracle rc 0: 4
b_stem_default_send2	same	ours rc 0: 0	oracle rc 0: 0
b_run_array_items	same	ours rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.
b_run_string_on_array	same	ours rc 159: Error 97.2:  Object "an Array" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Array" cannot accept private message "RUN" from this context.
b_setmethod_array_put	same	ours rc 159: Error 97.2:  Object "an Object" cannot accept private message "SETMETHOD" from this context.	oracle rc 159: Error 97.2:  Object "an Object" cannot accept private message "SETMETHOD" from this context.
b_run_list_items	same	ours rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.
b_run_queue_items	same	ours rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.
b_run_supplier_item	same	ours rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.
b_run_table_items	same	ours rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.
b_run_package_name	same	ours rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.
b_run_method_source	same	ours rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.
b_run_class_id	same	ours rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.
b_run_varref_name	same	ours rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.
b_run_stackframe_name	same	ours rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.
b_context_executable_setmethod	same	ours rc 158: Error 98.991:  Method SETMETHOD may only be invoked from a method of the same object or one of its classes.	oracle rc 158: Error 98.991:  Method SETMETHOD may only be invoked from a method of the same object or one of its classes.
b_identities_scope_gone	DIFF	ours rc 120: rexx-exec: a message send to a method context whose scope no longer defines it is not implemented (Phase 5)	oracle rc 0: Method
b_method_scope_on_object	same	ours rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.
b_annotations_on_object	same	ours rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Object" cannot accept private message "RUN" from this context.
b_stream_env	same	ours rc 0: Stream	oracle rc 0: Stream
b2_ext_setmethod_array	DIFF	ours rc 120: rexx-exec: 99.916: Unrecognized directive instruction.	oracle rc 157: Error 99.916:  Unrecognized directive instruction.
b2_ext_setmethod_string	DIFF	ours rc 120: rexx-exec: 99.916: Unrecognized directive instruction.	oracle rc 157: Error 99.916:  Unrecognized directive instruction.
b2_ext_setmethod_object_scope	DIFF	ours rc 120: rexx-exec: 99.916: Unrecognized directive instruction.	oracle rc 157: Error 99.916:  Unrecognized directive instruction.
b2_ext_unsetmethod_string	DIFF	ours rc 120: rexx-exec: 99.916: Unrecognized directive instruction.	oracle rc 157: Error 99.916:  Unrecognized directive instruction.
b2_ext_expose_array	DIFF	ours rc 120: rexx-exec: 99.916: Unrecognized directive instruction.	oracle rc 157: Error 99.916:  Unrecognized directive instruction.
b2_ext_expose_string_run	DIFF	ours rc 120: rexx-exec: 99.916: Unrecognized directive instruction.	oracle rc 157: Error 99.916:  Unrecognized directive instruction.
b2_ext_expose_array_ext	DIFF	ours rc 120: rexx-exec: 99.916: Unrecognized directive instruction.	oracle rc 157: Error 99.916:  Unrecognized directive instruction.
b2_run_array_items_on_inst	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 0: 140509746667456
b2_run_array_put_on_inst	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 163: Error 93.902:  Too many arguments in invocation of method; 3 expected.
b2_run_string_length_on_array	DIFF	ours rc 120: rexx-exec: 99.916: Unrecognized directive instruction.	oracle rc 157: Error 99.916:  Unrecognized directive instruction.
b2_run_list_items_on_inst	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 139: 
b2_run_queue_items_on_inst	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 0: 140270845403072
b2_run_supplier_item_on_inst	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 139: 
b2_run_table_items_on_inst	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 0: 0
b2_run_package_name_on_inst	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 139: 
b2_run_class_id_on_inst	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 165: Error 91.999:  Message "RUN" did not return a result.
b2_run_varref_name_on_inst	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 139: 
b2_run_stackframe_name_on_inst	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 139: 
b2_run_method_scope_on_inst	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 0: 
b2_run_routine_call_on_inst	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 139: 
b2_run_hasmethod_on_weakref	same	ours rc 0: 1 / 1 a WeakReference a WeakReference / Supplier	oracle rc 0: 1 / 1 a WeakReference a WeakReference / Supplier
b2_weakref_copy	same	ours rc 0: WeakReference	oracle rc 0: WeakReference
b2_weakref_objectname_set	same	ours rc 0: x	oracle rc 0: x
b2_do_to_class	DIFF	ours rc 120: rexx-exec: a class object as a DO header's TO value is not implemented (Phase 5)	oracle rc 0: 97 97.1
b2_do_by_inst	DIFF	ours rc 120: rexx-exec: an instance of a user class as a DO header's BY value is not implemented (Phase 5)	oracle rc 159: Error 97.1:  Object "a T" does not understand message "+".
b2_do_for_env	same	ours rc 230: Error 26.3:  Value of FOR expression in DO or LOOP instruction must be zero or a positive whole number; found "The Environment Directory".	oracle rc 230: Error 26.3:  Value of FOR expression in DO or LOOP instruction must be zero or a positive whole number; found "The Environment Directory".
b2_do_rep_inst	same	ours rc 230: Error 26.2:  Value of repetition count expression in DO or LOOP instruction must be zero or a positive whole number; found "a T".	oracle rc 230: Error 26.2:  Value of repetition count expression in DO or LOOP instruction must be zero or a positive whole number; found "a T".
b2_forward_args_env	DIFF	ours rc 120: rexx-exec: 35.1: Invalid expression.	oracle rc 221: Error 35.1:  Incorrect expression detected at "S".
b2_forward_args_inst	DIFF	ours rc 120: rexx-exec: 35.1: Invalid expression.	oracle rc 221: Error 35.1:  Incorrect expression detected at "S".
b2_forward_args_weak	DIFF	ours rc 120: rexx-exec: 35.1: Invalid expression.	oracle rc 221: Error 35.1:  Incorrect expression detected at "S".
b2_identities_scope_gone2	DIFF	ours rc 120: rexx-exec: a message send to a method context whose scope no longer defines it is not implemented (Phase 5)	oracle rc 0: 2
b2_ext_method_new_object_src	DIFF	ours rc 120: rexx-exec: a method source that is neither a string nor an array is not implemented (Phase 5)	oracle rc 0: 93 93.961
b2_define_methods_src	DIFF	ours rc 120: rexx-exec: method "M" of class "K" is not implemented (Phase 9)	oracle rc 0: 7
b2_operator_dir_plus	same	ours rc 0: The NIL object	oracle rc 0: The NIL object
b2_operator_env_plus	same	ours rc 0: The NIL object	oracle rc 0: The NIL object
b2_operator_inst_lt	same	ours rc 159: Error 97.1:  Object "a T" does not understand message "<".	oracle rc 159: Error 97.1:  Object "a T" does not understand message "<".
b2_class_define_on_array	same	ours rc 158: Error 98.985:  User additions are not allowed to the REXX language classes.	oracle rc 158: Error 98.985:  User additions are not allowed to the REXX language classes.
b3_send_setmethod_array	same	ours rc 159: Error 97.2:  Object "an Array" cannot accept private message "SETMETHOD" from this context.	oracle rc 159: Error 97.2:  Object "an Array" cannot accept private message "SETMETHOD" from this context.
b3_send_setmethod_string	same	ours rc 159: Error 97.2:  Object "abc" cannot accept private message "SETMETHOD" from this context.	oracle rc 159: Error 97.2:  Object "abc" cannot accept private message "SETMETHOD" from this context.
b3_send_setmethod_scope_object_array	same	ours rc 159: Error 97.2:  Object "an Array" cannot accept private message "SETMETHOD" from this context.	oracle rc 159: Error 97.2:  Object "an Array" cannot accept private message "SETMETHOD" from this context.
b3_send_setmethod_weak	same	ours rc 159: Error 97.2:  Object "a WeakReference" cannot accept private message "SETMETHOD" from this context.	oracle rc 159: Error 97.2:  Object "a WeakReference" cannot accept private message "SETMETHOD" from this context.
b3_send_setmethod_scope_object_weak	same	ours rc 159: Error 97.2:  Object "a WeakReference" cannot accept private message "SETMETHOD" from this context.	oracle rc 159: Error 97.2:  Object "a WeakReference" cannot accept private message "SETMETHOD" from this context.
b3_send_unsetmethod_array	same	ours rc 159: Error 97.2:  Object "an Array" cannot accept private message "UNSETMETHOD" from this context.	oracle rc 159: Error 97.2:  Object "an Array" cannot accept private message "UNSETMETHOD" from this context.
b3_send_run_expose_array	same	ours rc 159: Error 97.2:  Object "an Array" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Array" cannot accept private message "RUN" from this context.
b3_send_run_expose_string	same	ours rc 159: Error 97.2:  Object "abc" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "abc" cannot accept private message "RUN" from this context.
b3_send_run_expose_weak	same	ours rc 159: Error 97.2:  Object "a WeakReference" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "a WeakReference" cannot accept private message "RUN" from this context.
b3_send_run_expose_dir	same	ours rc 0: The NIL object	oracle rc 0: The NIL object
b3_send_run_expose_package	same	ours rc 159: Error 97.2:  Object "a Package" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "a Package" cannot accept private message "RUN" from this context.
b3_send_run_items_array_subclass	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 0: 3
b3_send_run_items_array	same	ours rc 159: Error 97.2:  Object "an Array" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Array" cannot accept private message "RUN" from this context.
b3_send_run_length_on_array	same	ours rc 159: Error 97.2:  Object "an Array" cannot accept private message "RUN" from this context.	oracle rc 159: Error 97.2:  Object "an Array" cannot accept private message "RUN" from this context.
b3_setmethod_primitive_same_type	same	ours rc 159: Error 97.2:  Object "an Array" cannot accept private message "SETMETHOD" from this context.	oracle rc 159: Error 97.2:  Object "an Array" cannot accept private message "SETMETHOD" from this context.
b3_setmethod_primitive_on_object	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 0: 139920990111360
b3_forward_args_env	DIFF	ours rc 120: rexx-exec: an instance of a user class as FORWARD ARGUMENTS is not implemented (Phase 5)	oracle rc 0: 10
b3_forward_args_inst	DIFF	ours rc 120: rexx-exec: an instance of a user class as FORWARD ARGUMENTS is not implemented (Phase 5)	oracle rc 0: 2 1
b3_forward_args_weak	same	ours rc 158: Error 98.946:  FORWARD arguments must be a single-dimensional array of values.	oracle rc 158: Error 98.946:  FORWARD arguments must be a single-dimensional array of values.
b3_do_ctrl_inst	DIFF	ours rc 120: rexx-exec: an instance of a user class as a controlled DO's control variable is not implemented (Phase 5)	oracle rc 159: Error 97.1:  Object "a T" does not understand message "+".
b3_do_ctrl_array	DIFF	ours rc 120: rexx-exec: an array as a controlled DO's control variable is not implemented (Phase 5)	oracle rc 159: Error 97.1:  Object "an Array" does not understand message "+".
b3_do_to_array	DIFF	ours rc 120: rexx-exec: an array as a DO header's TO value is not implemented (Phase 5)	oracle rc 0: 97 97.1
b3_do_by_ctx	DIFF	ours rc 120: rexx-exec: one of the interpreter's own objects as a DO header's BY value is not implemented (Phase 5)	oracle rc 0: 97 97.1
b3_raise_additional_inst	DIFF	ours rc 120: rexx-exec: an instance of a user class as a RAISE ADDITIONAL value is not implemented (Phase 5)	oracle rc 0: SYNTAX 98 98.939
b3_raise_additional_env	DIFF	ours rc 120: rexx-exec: an instance of a user class as a RAISE ADDITIONAL value is not implemented (Phase 5)	oracle rc 0: SYNTAX 93 93.900 / Array
b3_raise_additional_class	DIFF	ours rc 120: rexx-exec: a class object as a RAISE ADDITIONAL value is not implemented (Phase 5)	oracle rc 0: SYNTAX 98 98.939
b3_raise_additional_class_trace	DIFF	ours rc 120: rexx-exec: a class object as a RAISE ADDITIONAL value is not implemented (Phase 5)	oracle rc 158: Error 98.939:  Additional information for SYNTAX errors must be a single-dimensional array of values.
b4_enhanced_collections	same	ours rc 159: Error 97.1:  Object "an Array" does not understand message "ENHANCED".	oracle rc 159: Error 97.1:  Object "an Array" does not understand message "ENHANCED".
b4_copy_collections	same	ours rc 0: 3 / 3 / 3 / 2 / 1 / 1 / 1 / 1 / The NIL object	oracle rc 0: 3 / 3 / 3 / 2 / 1 / 1 / 1 / 1 / The NIL object
b4_subclass_no_init	same	ours rc 165: Error 91.999:  Message "QUEUE" did not return a result.	oracle rc 165: Error 91.999:  Message "QUEUE" did not return a result.
b4_subclass_supplier	DIFF	ours rc 120: rexx-exec: a message send to a value that is not a supplier is not implemented (Phase 5)	oracle rc 139: 
b4_subclass_supplier_noinit_avail	DIFF	ours rc 120: rexx-exec: a message send to a value that is not a supplier is not implemented (Phase 5)	oracle rc 139: 
b4_package_new_methods	same	ours rc 0: 1 / x Directory	oracle rc 0: 1 / x Directory
b4_mixin_array	same	ours rc 158: Error 98.943:  Class "The K class" is not a subclass of "The ARRAYMIX class" base class "The Array class".	oracle rc 158: Error 98.943:  Class "The K class" is not a subclass of "The ARRAYMIX class" base class "The Array class".
b4_mixin_list	same	ours rc 158: Error 98.943:  Class "The K class" is not a subclass of "The LM class" base class "The List class".	oracle rc 158: Error 98.943:  Class "The K class" is not a subclass of "The LM class" base class "The List class".
b4_varref_on_stem_default	same	ours rc 0: 0	oracle rc 0: 0
b4_reference_methods	same	ours rc 0: X 5 / 6	oracle rc 0: X 5 / 6
b5_enhanced_array	same	ours rc 0: 1 1 3	oracle rc 0: 1 1 3
b5_enhanced_queue	same	ours rc 0: 1 1 3	oracle rc 0: 1 1 3
b5_enhanced_list	same	ours rc 0: 1 1 3	oracle rc 0: 1 1 3
b5_enhanced_table	same	ours rc 0: 1 1 1	oracle rc 0: 1 1 1
b5_enhanced_supplier	same	ours rc 0: 1 2 1	oracle rc 0: 1 2 1
b5_enhanced_directory	same	ours rc 0: 1 1	oracle rc 0: 1 1
b5_enhanced_stringtable	same	ours rc 0: 1 1	oracle rc 0: 1 1
b5_enhanced_circ	same	ours rc 0: 1 1	oracle rc 0: 1 1
b6_routine_sources_call	same	ours rc 0: 1 1 Package / pr 1 call/p.rex / 0 / pr / pr	oracle rc 0: 1 1 Package / pr 1 call/p.rex / 0 / pr / pr
b6_routine_borrow_call	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 158: Error 98.991:  Method RUN may only be invoked from a method of the same object or one of its classes.
b6_context_executable_in_routine	same	ours rc 159: Error 97.1:  Object "a Routine" does not understand message "NAME".	oracle rc 159: Error 97.1:  Object "a Routine" does not understand message "NAME".
b6_context_executable_routine_obj	DIFF	ours rc 120: rexx-exec: a message send to a routine whose package has no table is not implemented (Phase 5)	oracle rc 0: Routine
b6_context_executable_newfile	same	ours rc 253: Error 3.1:  Failure during initialization: File "rf.rex" is unreadable.	oracle rc 253: Error 3.1:  Failure during initialization: File "rf.rex" is unreadable.
b6_method_scope_annotations	same	ours rc 0: The NIL object StringTable / The Array class StringTable / The T class StringTable / Strin	oracle rc 0: The NIL object StringTable / The Array class StringTable / The T class StringTable / Strin
b6_package_addclass	same	ours rc 0: p1 / 1 / p2 / Package / p3 / Package / REXX / 98 98.984	oracle rc 0: p1 / 1 / p2 / Package / p3 / Package / REXX / 98 98.984
b6_package_local_name	same	ours rc 0: Directory / r / REXX Directory / The REXX Package	oracle rc 0: Directory / r / REXX Directory / The REXX Package
b6_hash_native_puts	same	ours rc 159: Error 97.1:  Object ".METHODS" does not understand message "PUT".	oracle rc 159: Error 97.1:  Object ".METHODS" does not understand message "PUT".
b6_hash_native_routines_put	same	ours rc 159: Error 97.1:  Object ".ROUTINES" does not understand message "PUT".	oracle rc 159: Error 97.1:  Object ".ROUTINES" does not understand message "PUT".
b6_hash_native_classes_put	same	ours rc 0: 0	oracle rc 0: 0
b6_hash_native_publicclasses_put	same	ours rc 0: ok	oracle rc 0: ok
b6_hash_native_resources	same	ours rc 159: Error 97.1:  Object ".RESOURCES" does not understand message "PUT".	oracle rc 159: Error 97.1:  Object ".RESOURCES" does not understand message "PUT".
b6_hash_native_remove_empty	same	ours rc 159: Error 97.1:  Object ".METHODS" does not understand message "EMPTY".	oracle rc 159: Error 97.1:  Object ".METHODS" does not understand message "EMPTY".
b6_hash_native_copy	same	ours rc 159: Error 97.1:  Object ".METHODS" does not understand message "PUT".	oracle rc 159: Error 97.1:  Object ".METHODS" does not understand message "PUT".
b6_subclass_array_setmethod	same	ours rc 0: 42	oracle rc 0: 42
b6_subclass_array_setmethod_object	same	ours rc 0: 42	oracle rc 0: 42
b6_subclass_array_expose	same	ours rc 0: 3	oracle rc 0: 3
b6_define_with_primitive	same	ours rc 0: defined	oracle rc 0: defined
b6_define_methods_with_primitive	same	ours rc 0: defined	oracle rc 0: defined
b6_define_array_subclass_with_primitive	DIFF	ours rc 120: rexx-exec: method "CNT" of class "K" is not implemented (Phase 9)	oracle rc 0: 2
b6_enhanced_with_primitive	DIFF	ours rc 120: rexx-exec: an enhancing method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 0: 1
b6_setmethod_primitive_array_subclass	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 0: 3
b6_define_class_method_primitive	same	ours rc 158: Error 98.985:  User additions are not allowed to the REXX language classes.	oracle rc 158: Error 98.985:  User additions are not allowed to the REXX language classes.
b6_stackframes	same	ours rc 0: R INTERNALCALL 4 a StackFrame / /tmp/claude-1000/p61/sa/probes/b6_stackframes/p.rex PROGRA	oracle rc 0: R INTERNALCALL 4 a StackFrame / /tmp/claude-1000/p61/sa/probes/b6_stackframes/p.rex PROGRA
b7_subclass_enhancing_src	DIFF	ours rc 120: rexx-exec: a class method built from source text is not implemented (Phase 5)	oracle rc 0: 7
b7_subclass_enhancing_method	same	ours rc 0: 7	oracle rc 0: 7
b7_subclass_enhancing_primitive	DIFF	ours rc 120: rexx-exec: a class method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 0: K
b7_mixinclass_enhancing_src	DIFF	ours rc 120: rexx-exec: a class method built from source text is not implemented (Phase 5)	oracle rc 0: 7
b7_method_new_bad_types	DIFF	ours rc 120: rexx-exec: a method source that is neither a string nor an array is not implemented (Phase 5)	oracle rc 0: 1 / 0 / 93 93.961
b7_routine_new_object_src	DIFF	ours rc 120: rexx-exec: a method source that is neither a string nor an array is not implemented (Phase 5)	oracle rc 0: 93 93.961
b7_setmethod_object_source	DIFF	ours rc 120: rexx-exec: a method source that is neither a string nor an array is not implemented (Phase 5)	oracle rc 0: 93 93.974
b7_run_object_source	DIFF	ours rc 120: rexx-exec: a method source that is neither a string nor an array is not implemented (Phase 5)	oracle rc 0: 93 93.974
b7_define_object_source	DIFF	ours rc 120: rexx-exec: a method source that is neither a string nor an array is not implemented (Phase 5)	oracle rc 0: 93 93.974
b7_setmethod_scope_object_dir	same	ours rc 163: Error 93.902:  Too many arguments in invocation of method; 2 expected.	oracle rc 163: Error 93.902:  Too many arguments in invocation of method; 2 expected.
b7_setmethod_scope_object_inst	same	ours rc 0: 42	oracle rc 0: 42
b8_string_setm	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: 42
b8_string_setmobj	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: 42
b8_string_expose	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: 3
b8_string_unset	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: ok
b8_stem_setm	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: 42
b8_stem_setmobj	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: 42
b8_stem_expose	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: 3
b8_stem_unset	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: ok
b8_directory_setm	same	ours rc 0: 42	oracle rc 0: 42
b8_directory_setmobj	same	ours rc 163: Error 93.902:  Too many arguments in invocation of method; 2 expected.	oracle rc 163: Error 93.902:  Too many arguments in invocation of method; 2 expected.
b8_directory_expose	same	ours rc 0: 3	oracle rc 0: 3
b8_directory_unset	same	ours rc 0: ok	oracle rc 0: ok
b8_stringtable_setm	same	ours rc 0: 42	oracle rc 0: 42
b8_stringtable_setmobj	same	ours rc 0: 42	oracle rc 0: 42
b8_stringtable_expose	same	ours rc 0: 3	oracle rc 0: 3
b8_stringtable_unset	same	ours rc 0: ok	oracle rc 0: ok
b8_method_setm	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: 42
b8_method_setmobj	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: 42
b8_method_expose	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: 3
b8_method_unset	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: ok
b8_routine_setm	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: 42
b8_routine_setmobj	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: 42
b8_routine_expose	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: 3
b8_routine_unset	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: ok
b8_package_setm	DIFF	ours rc 159: Error 97.1:  Object "a Package" does not understand message "GO".	oracle rc 0: 1 / 42
b8_package_setmobj	DIFF	ours rc 159: Error 97.1:  Object "a Package" does not understand message "GO".	oracle rc 0: 1 / 42
b8_package_expose	DIFF	ours rc 159: Error 97.1:  Object "a Package" does not understand message "GO".	oracle rc 0: 1 / 3
b8_package_unset	DIFF	ours rc 159: Error 97.1:  Object "a Package" does not understand message "GO".	oracle rc 0: 1 / ok
b8_weakreference_setm	same	ours rc 0: 42	oracle rc 0: 42
b8_weakreference_setmobj	same	ours rc 0: 42	oracle rc 0: 42
b8_weakreference_expose	same	ours rc 0: 3	oracle rc 0: 3
b8_weakreference_unset	same	ours rc 0: ok	oracle rc 0: ok
b8_message_setm	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: 42
b8_message_setmobj	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: 42
b8_message_expose	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: 3
b8_message_unset	DIFF	ours rc 120: rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)	oracle rc 0: ok
b8_properties_setm	same	ours rc 0: 42	oracle rc 0: 42
b8_properties_setmobj	same	ours rc 163: Error 93.902:  Too many arguments in invocation of method; 2 expected.	oracle rc 163: Error 93.902:  Too many arguments in invocation of method; 2 expected.
b8_properties_expose	same	ours rc 0: 3	oracle rc 0: 3
b8_properties_unset	same	ours rc 0: ok	oracle rc 0: ok
b8_mutablebuffer_setm	same	ours rc 0: 42	oracle rc 0: 42
b8_mutablebuffer_setmobj	same	ours rc 0: 42	oracle rc 0: 42
b8_mutablebuffer_expose	same	ours rc 0: 3	oracle rc 0: 3
b8_mutablebuffer_unset	same	ours rc 0: ok	oracle rc 0: ok
b8_pointer_setm	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_pointer_setmobj	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_pointer_expose	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_pointer_unset	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_buffer_setm	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_buffer_setmobj	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_buffer_expose	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_buffer_unset	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_variablereference_setm	DIFF	ours rc 120: rexx-exec: method "NEW" of class "VariableReference" is not implemented (Phase 9)	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_variablereference_setmobj	DIFF	ours rc 120: rexx-exec: method "NEW" of class "VariableReference" is not implemented (Phase 9)	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_variablereference_expose	DIFF	ours rc 120: rexx-exec: method "NEW" of class "VariableReference" is not implemented (Phase 9)	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_variablereference_unset	DIFF	ours rc 120: rexx-exec: method "NEW" of class "VariableReference" is not implemented (Phase 9)	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_stackframe_setm	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_stackframe_setmobj	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_stackframe_expose	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_stackframe_unset	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_rexxcontext_setm	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_rexxcontext_setmobj	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_rexxcontext_expose	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_rexxcontext_unset	same	ours rc 163: Error 93.967:  NEW method is not supported for the K class.	oracle rc 163: Error 93.967:  NEW method is not supported for the K class.
b8_rexxinfo_setm	DIFF	ours rc 158: Error 98.909:  Class "REXXINFO" not found.	oracle rc 157: Error 99.949:  "REXXINFO" is not a valid class.
b8_rexxinfo_setmobj	DIFF	ours rc 158: Error 98.909:  Class "REXXINFO" not found.	oracle rc 157: Error 99.949:  "REXXINFO" is not a valid class.
b8_rexxinfo_expose	DIFF	ours rc 158: Error 98.909:  Class "REXXINFO" not found.	oracle rc 157: Error 99.949:  "REXXINFO" is not a valid class.
b8_rexxinfo_unset	DIFF	ours rc 158: Error 98.909:  Class "REXXINFO" not found.	oracle rc 157: Error 99.949:  "REXXINFO" is not a valid class.
b8_exception_setm	same	ours rc 158: Error 98.909:  Class "EXCEPTION" not found.	oracle rc 158: Error 98.909:  Class "EXCEPTION" not found.
b8_exception_setmobj	same	ours rc 158: Error 98.909:  Class "EXCEPTION" not found.	oracle rc 158: Error 98.909:  Class "EXCEPTION" not found.
b8_exception_expose	same	ours rc 158: Error 98.909:  Class "EXCEPTION" not found.	oracle rc 158: Error 98.909:  Class "EXCEPTION" not found.
b8_exception_unset	same	ours rc 158: Error 98.909:  Class "EXCEPTION" not found.	oracle rc 158: Error 98.909:  Class "EXCEPTION" not found.
b8_regularexpression_setm	same	ours rc 158: Error 98.909:  Class "REGULAREXPRESSION" not found.	oracle rc 158: Error 98.909:  Class "REGULAREXPRESSION" not found.
b8_regularexpression_setmobj	same	ours rc 158: Error 98.909:  Class "REGULAREXPRESSION" not found.	oracle rc 158: Error 98.909:  Class "REGULAREXPRESSION" not found.
b8_regularexpression_expose	same	ours rc 158: Error 98.909:  Class "REGULAREXPRESSION" not found.	oracle rc 158: Error 98.909:  Class "REGULAREXPRESSION" not found.
b8_regularexpression_unset	same	ours rc 158: Error 98.909:  Class "REGULAREXPRESSION" not found.	oracle rc 158: Error 98.909:  Class "REGULAREXPRESSION" not found.
b9_constant_send	same	ours rc 0: 5 5	oracle rc 0: 5 5
b9_constant_expr	same	ours rc 0: 5 5	oracle rc 0: 5 5
b9_attribute_get_set	same	ours rc 0: 4	oracle rc 0: 4
b9_attribute_get_code	same	ours rc 0: 9	oracle rc 0: 9
b9_method_attribute	same	ours rc 0: 4	oracle rc 0: 4
b9_attribute_class	same	ours rc 0: 3	oracle rc 0: 3
b9_constant_no_value	same	ours rc 0: C	oracle rc 0: C
b9_floating_constant	same	ours rc 0: a Method	oracle rc 0: a Method
b9_floating_attribute	same	ours rc 0: 	oracle rc 0: 
b9_method_from_methods	same	ours rc 0: floated	oracle rc 0: floated
b9_setmethod_from_directive_method	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 0: 5
b9_setmethod_from_attr_method	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 0: A
b9_setmethod_from_delegate	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 0: 1
b10_setmethod_from_plain_method	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 0: um um
b10_run_constant_method	DIFF	ours rc 120: rexx-exec: a one-off method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 0: 5
b10_enhanced_constant_method	DIFF	ours rc 120: rexx-exec: an enhancing method whose body this crate does not hold is not implemented (Phase 5)	oracle rc 0: 5
b10_define_constant_method	DIFF	ours rc 120: rexx-exec: method "CC" of class "K" is not implemented (Phase 9)	oracle rc 0: 5
b10_define_attr_method	DIFF	ours rc 120: rexx-exec: method "AA" of class "K" is not implemented (Phase 9)	oracle rc 0: A
b11_ctx_exec_requires	same	ours rc 0: Routine	oracle rc 0: Routine
b11_ctx_exec_package_new	same	ours rc 0: Routine	oracle rc 0: Routine
b11_ctx_exec_routine_new_nested	same	ours rc 0: Routine	oracle rc 0: Routine
b11_ctx_exec_external_call	same	ours rc 0: Routine	oracle rc 0: Routine
b11_ctx_exec_interpret_in_routine	same	ours rc 0: Routine	oracle rc 0: Routine
b11_ctx_exec_method_new	DIFF	ours rc 120: rexx-exec: a message send to a method context whose scope no longer defines it is not implemented (Phase 5)	oracle rc 0: Method
b11_ctx_exec_enhanced	DIFF	ours rc 120: rexx-exec: a message send to a method context whose scope no longer defines it is not implemented (Phase 5)	oracle rc 0: Method
b11_ctx_exec_floating_method	DIFF	ours rc 120: rexx-exec: a message send to a method context whose scope no longer defines it is not implemented (Phase 5)	oracle rc 0: Method
b11_ctx_exec_dup_routine	same	ours rc 0: Routine	oracle rc 0: Routine
b11_ctx_exec_routine_new_call_ctx	DIFF	ours rc 0: r CALL	oracle rc 0: r r
b12_method_newfile_bad_abs	DIFF	ours rc 120: rexx-exec: reporting a file that does not parse (/tmp/claude-1000/p61/sa/probes/b12_method_newfile_bad_abs/badm.rex, 35.1: Invalid expression.) is not implemented (Phase 5)	oracle rc 0: 35 35.1 1
b12_routine_newfile_bad_abs	DIFF	ours rc 120: rexx-exec: reporting a file that does not parse (/tmp/claude-1000/p61/sa/probes/b12_routine_newfile_bad_abs/badm.rex, 35.1: Invalid expression.) is not implemented (Phase 5)	oracle rc 0: 35 35.1 1
b12_routine_newfile_bad_untrapped	DIFF	ours rc 120: rexx-exec: reporting a file that does not parse (/tmp/claude-1000/p61/sa/probes/b12_routine_newfile_bad_untrapped/badm.rex, 35.1: Invalid expression.) is not implemented (Phase 5)	oracle rc 221: Error 35.1:  Incorrect expression detected at "(".
b12_package_new_bad_untrapped	DIFF	ours rc 120: rexx-exec: x does not parse here: 35.1: Invalid expression. is not implemented (Phase 5)	oracle rc 221: Error 35.1:  Incorrect expression detected at "(".
b12_requires_trapped	DIFF	ours rc 120: rexx-exec: /tmp/claude-1000/p61/sa/probes/b12_requires_trapped/bad.rex does not parse here: 35.1: Invalid expression. is not implemented (Phase 5)	oracle rc 221: Error 35.1:  Incorrect expression detected at "(".
b12_interpret_bad	same	ours rc 0: 35 35.1	oracle rc 0: 35 35.1
b12_method_new_bad	same	ours rc 0: 35 35.1	oracle rc 0: 35 35.1
env_new_activity	same	ours rc 0: DEBUGINPUT:Monitor ERROR:Monitor INPUT:Monitor OUTPUT:Monitor STDERR:Stream STDIN:Stream S	oracle rc 0: DEBUGINPUT:Monitor ERROR:Monitor INPUT:Monitor OUTPUT:Monitor STDERR:Stream STDIN:Stream S
env_reply_activity	same	ours rc 0: r / DEBUGINPUT:Monitor ERROR:Monitor INPUT:Monitor OUTPUT:Monitor STDERR:Stream STDIN:Stre	oracle rc 0: r / DEBUGINPUT:Monitor ERROR:Monitor INPUT:Monitor OUTPUT:Monitor STDERR:Stream STDIN:Stre
env_supplier	DIFF	ours rc 120: rexx-exec: directory entry "STDQUE" is not implemented (Phase 10)	oracle rc 0: 1 / ok
env_items_makearray	same	ours rc 0: 1 / 10	oracle rc 0: 1 / 10
env_list	same	ours rc 0: E ALARM / E ALARMNOTIFICATION / E ARGUTIL / E ARRAY / E BAG / E BUFFER / E CASELESSCOLUMNC	oracle rc 0: E ALARM / E ALARMNOTIFICATION / E ARGUTIL / E ARRAY / E BAG / E BUFFER / E CASELESSCOLUMNC
```

## Appendix D: ooTest form search

```
grep -rliE '^\s*(do|loop)\b[^;]*\b(counter|with)\b' --include=*.testGroup --include=*.cls --include=*.rex --include=*.frm .
ooRexx/base/class/File.testGroup
ooRexx/base/class/Method.testGroup
ooRexx/base/class/Package.testGroup
ooRexx/base/class/Routine.testGroup
ooRexx/base/directives/REQUIRES.testGroup
ooRexx/base/keyword/DO.testGroup
ooRexx/base/keyword/DoWith.testGroup
ooRexx/base/keyword/LOOP.testGroup
ooRexx/base/keyword/LoopWith.testGroup
ooRexx/base/keyword/TRACE.testGroup
ooRexx/base/keyword/TRACE_TraceObject.testGroup
ooRexx/base/rexxutil/SysFileTree.testGroup

grep -rliE '^\s*use local\b' --include=*.testGroup .
ooRexx/base/keyword/VarRef.testGroup
ooRexx/base/keyword/GUARD.testGroup
ooRexx/base/security.manager/SecurityManager.testGroup

grep -rliE '^\s*options\b' --include=*.testGroup .
ooRexx/base/rexxutil/SysFileTree.testGroup
ooRexx/base/keyword/TRACE.testGroup
ooRexx/base/keyword/ASSIGNMENT.testGroup
ooRexx/base/bif/DATE.testGroup

grep -rli 'use arg [a-z_]*~' --include=*.testGroup .
ooRexx/base/keyword/USE.testGroup
```
