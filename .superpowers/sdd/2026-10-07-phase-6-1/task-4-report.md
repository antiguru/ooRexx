# Task 4 report: Method objects and setMethod

Base `a3c2c3c0a`. Commits: `7d233521f` (behaviour, witnesses, refusal-sites.tsv), `899db70ac`
(SOURCELINE expectations), and the commit carrying this report and the gate record section.

## Design

Every `Method` object this crate hands out now carries the method identity (`MethodId`) it runs
under, in `ExecutableRecord::installed` (`lib.rs`, doc rewritten). Before, only objects a class
dictionary reached had one; compiled and `.METHODS` objects had a body in `table_method_bodies`,
and every other object had nothing, which is what b5 refused. The table is gone.

- Where the identity comes from: a directive's own entry (`Class~method`, unchanged); minted at
  compile (`install.rs:1701` `mint_unattached_body`, from `record_compiled_body` and
  `install_executable`); minted at the `.METHODS` table build with the body the directive
  generates (`install.rs:1712` `unattached_method_id`, called at `environment.rs:1031`; the
  `InstallBody` choice is `install.rs:1759` `install_body`, which `install_method` and
  `install_attribute` now share; a floating literal `::CONSTANT` records its value there);
  minted for `loadExternalMethod` answers (`install.rs:1960` library procedure via
  `identities.rs:411` `bind_loaded_method`; `install.rs:1975` `record_rexx_external_method` for
  `LIBRARY REXX`, called from `construct.rs:288`).
- How it is installed: `identities.rs:391` `scoped_method` is `MethodClass::newScope` plus the
  identity. The object itself while it has no scope keeps its own identity; a copy gets one from
  `dispatch.rs:2304` `copy_method_identity`, which copies every row a send reads (native entry,
  Rexx body, generated method, externals, library binding, guard, access scope). Users:
  `setMethod` (`object_protocol.rs:470`, newScope to `.nil` ahead of the checks then to the
  target scope, as `ObjectClass.cpp:1847`/`:2302` do), `run` (`object_protocol.rs:705`, `:737`),
  `enhanced` (`class_protocol.rs:1157`), `define`, `defineMethods`, `defineClassMethod`
  (`identities.rs:372`, `:434`, `define_method_table`) and a subclass's enhancing class methods
  (`class_protocol.rs:819`, which now compiles source text: b6), and `Directory~setMethod`
  (`hash.rs`).
- `define` never recorded a body before (any `define`d method, source text included, refused
  `method "M" of class "K" ... (Phase 9)` when sent). It now runs.
- `.context~executable`: `ObjectMethod` (rexx-core `body.rs:771`) carries the `Method` object
  and traces it, so an own entry keeps its object alive and answers it; `run` puts its object
  in the callee's cold box (`activation.rs:592`, traced) when the body is Rexx;
  `identities.rs:217` `running_method_executable` asks the class entry (checked to run this
  body), then the receiver's own entry, then any live object standing for an identity running
  the body (lowest id), then builds one. A routine compiled by `Routine~new` answers itself
  (`context.rs:454`).
- b14: a class object's own methods sit on the object holding its variable pools
  (`class_variables`): `dispatch.rs:2108` writes there, `dispatch.rs:2091` `class_own_entry`
  reads it behind the existing `object_methods` flag.
- b4: `class_protocol.rs:250` `SourceTaker` and `:266` `method_source_lines` do
  `processExecutableSource`'s `requestArray` then `makeString` and raise 93.961 (`Method~new`,
  `Routine~new`, `Package~new`), 93.974 (everything that also takes a Method object) or 88.913
  (`loadPackage`'s `arrayArgument`). New `Raised` constructors `error.rs:1077`, `:1083`.
- Calling the program's own `Routine` (`.context~executable` in a main program) re-runs its main
  section as a subroutine: `lib.rs:2327` `call_program_main`, and `run_loaded` split so its
  post-install half is `lib.rs:2363` `run_main`; `executable.rs:619`.
- A `.nil` method scope renders `.NIL` (`dispatch.rs:1720` `class_id_text`, `:2340` `scope_id`
  at every traceback/trace site that rendered a method scope); before, any primitive or trace line
  under a `.nil` scope panicked in `ClassRegistry::id_string` (it did so at base for a traced
  compiled `setMethod` body).
- VariableReference `NEW` row: `dispatch.rs` beside Buffer/Pointer (`native_unsupported_new`).
- `begin_method` no longer reads `method_body_gap` (deleted from `directives.rs`): every
  `InstalledMethodBody` names a directive with a body, and `body_of` already refuses through
  `missing_body` otherwise.

## Refusals deleted

- `Loud::method_body` (constructor and its three texts).
- `Loud::method_from_source` at: `class_protocol.rs` `:274` (b4), `:777`/`:782` (b6/b5),
  `:1115` (b5), `object_protocol.rs` `:488`, `:728` (b5), `executable.rs` `:581`, `:622`. The
  constructor stays for one site this task does not own: `new_file_executable`'s parse failure
  (b3, Task 5). `:581` (no record) is now `receiver_class("a routine object this crate did not
  build")`, which `b2_run_routine_call_on_inst` reaches (Step 3); `:622` answers.
- `Loud::object_method` at `object_protocol.rs:531` (now `receiver_class("a value with no class
  of its own")`, the text `object_protocol.rs:89` already uses) and at `dispatch.rs:2082` for a
  class object (answers).
- `Loud::receiver_class("a method context whose scope no longer defines it")` from
  `context.rs:427`'s path (b7): `.context~executable` no longer reaches `method_executable`'s
  refusal; that function stays for native frames, which map its `Err` to `.nil`.
- `context.rs:430` (b8) is bypassed for a compiled routine.
- Pins deleted: `run/tests/directives.rs` `the_method_source_shapes_this_task_leaves_refuse_loudly`
  (`:264`, `:270`), `dispatch/tests.rs` `a_float_scoped_methods_executable_is_refused`.
  `scheduler/tests/native.rs:301`'s test needed a loud refusal in a started activity and now uses
  `.local['STDQUE']` (Phase 10); oracle 5 of 5 runs print `5` and `after`, rc 0 (comment updated).

Kept with owner `None`, one unreachable site each (doc comments at `lib.rs:601`, `:641`):
`object_method` (`dispatch.rs:2114`, a receiver neither an instance nor a class object) and
`expose_receiver` (`run.rs:1546`). Routes tried, none reaching them: `.object~define` and
`.string~inherit` (98.985 on both), SETMETHOD sent to an Array/String (97.2 on both), Directory,
`.local`, `.environment` and a Directory subclass running `setMethod`, `unsetMethod`,
`setMethod:.object` and `EXPOSE` from their own methods (answers, oracle-identical), a borrowed
Object `RUN` row on an instance (answers), and a borrowed Object `SETMETHOD` row a Directory's
entry method sends to that Directory (88.901 on both; the tracebacks differ, concern 3).
`refusal-sites.tsv` row for `object_method` is now `agrees no` with that reason.

`rust/corpus/refusal-sites.tsv` re-derived with `REXX_REFUSAL_SITES_REFRESH=1 cargo test -p
rexx-exec --test refusal_sites`; hand columns written for `not_a_method_or_source`,
`source_not_string_or_array` (agrees yes) and `object_method` (agrees no); `method_from_source`
is now `body`, `off-send-surface`; `method_body` gone. The test passes (5 passed).

## Corpus programs (`rust/corpus/phase-6-1.txt`, `lang/`)

Each run from an empty directory by
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t4/run.sh`
(ours default, ours `REXX_SWITCH_MODE=every`, oracle; stdout+stderr+rc compared): 53 of 53
`same`, no engine difference. Every stdout was read; the one witness that did not reach its
claimed path (`context_executable_class_method`, which sent an instance method to the class) was
fixed before the commit. Output, ours = oracle:

| program | from | output |
|---|---|---|
| method_new_object_source | b4 `b_method_new_object_source` | `93 93.961` |
| method_new_bad_types | b4 `b7_method_new_bad_types` | `1`, `0`, `93 93.961` |
| routine_new_object_source | b4 `b7_routine_new_object_src` | `93 93.961` |
| setmethod_object_source | b4 `b7_setmethod_object_source` | `93 93.974` |
| run_object_source | b4 `b7_run_object_source` | `93 93.974` |
| define_object_source | b4 `b7_define_object_source` | `93 93.974` |
| method_source_messages | added: each taker's position text | 93.961 `source`, 93.974 `method`/`method source` |
| method_source_conversions | added: requestArray/makeString | `.environment` 69 indexes, Directory 0, MAKESTRING, MAKEARRAY, List, stem, Queue, both (MAKEARRAY wins), MutableBuffer |
| load_package_object_source | added | 88.913, then a string source runs |
| setmethod_from_plain_method | b5 `b10_setmethod_from_plain_method` | `um um` |
| setmethod_from_constant_method | b5 `b9_setmethod_from_directive_method` | `5` |
| run_constant_method | b5 `b10_run_constant_method` | `5` |
| enhanced_constant_method | b5 `b10_enhanced_constant_method` | `5` |
| setmethod_from_attribute_method | b5 `b9_setmethod_from_attr_method` | `A` |
| setmethod_from_delegate | b5 `b9_setmethod_from_delegate` | `1` |
| run_native_items_array_subclass | b5 `b3_send_run_items_array_subclass` | `3` |
| setmethod_native_array_subclass | b5 `b6_setmethod_primitive_array_subclass` | `3` |
| enhanced_native_method | b5 `b6_enhanced_with_primitive` | `1` |
| subclass_enhancing_native | b5 `b7_subclass_enhancing_primitive` | `K` |
| define_native_array_subclass | b5 `b6_define_array_subclass_with_primitive` | `2` |
| define_constant_method | b5 `b10_define_constant_method` | `5` |
| define_attribute_method | b5 `b10_define_attr_method` | `A` |
| define_source_runs | added | `7`, `um` |
| setmethod_unattached_generated | added: `.METHODS` attribute pair, DELEGATE, ABSTRACT | `5 2 syntax 93.965` |
| setmethod_unattached_constant | added: `.METHODS` `::CONSTANT` | `7` |
| setmethod_copy_private | added | `priv`, `syntax 97.2` |
| define_methods_runs | added | `a ub K`, source line, then 97.1 rc 159 (a define after `~new`) |
| setmethod_native_nil_scope_traceback | added | rc 163, `Compiled method "HM" with scope ".NIL".` |
| run_native_traceback | added | rc 163, `*UNNAMED*` with scope `.NIL`, under `RUN` |
| setmethod_native_object_scope_traceback | added | rc 163, scope `T` |
| setmethod_trace_nil_scope | added | `>I> Method "HM" with scope ".NIL"` ... `<I<` |
| setmethod_attribute_scope | added | `U`, `9 9`, `U` |
| directory_setmethod_method_object | added | `um Directory` |
| borrowed_run_row | added | `A` |
| subclass_enhancing_source | b6 `b7_subclass_enhancing_src` | `7` |
| mixinclass_enhancing_source | b6 `b7_mixinclass_enhancing_src` | `7` |
| subclass_enhancing_expose | added | `7 7`, `42` |
| context_executable_setmethod | b7 `b11_ctx_exec_method_new` | `Method` |
| context_executable_enhanced | b7 `b11_ctx_exec_enhanced` | `Method` |
| context_executable_run_floating | b7 `b11_ctx_exec_floating_method` | `Method` |
| context_executable_scope_gone | b7 `b_identities_scope_gone` | `Method` |
| context_executable_setmethod_copy | added: identity | `0` |
| context_executable_run_copy | added: identity | `1`, `0` |
| context_executable_class_method | added | `Method 1`, `1`, `Class` |
| context_executable_routine_new | b8 `b6_context_executable_routine_obj` | `Routine` |
| program_routine_call | added (`executable.rs:622`) | `Routine`, `got x`, `got y` |
| program_routine_call_error | added | four calls (`SUBROUTINE`, caller variable kept), then 42.3 rc 214 under `Compiled method "CALL"` |
| class_setmethod | b14 scout D `cls_setm` | `42`, `42` |
| class_setmethod_object_scope | b14 `cls_setmobj` | `42 5` |
| class_unsetmethod | b14 `cls_unset` | `ok` |
| class_setmethod_hasmethod | added | `42`, `1 0`, `syntax 97.1` |
| variable_reference_new | VariableReference NEW | rc 163, 93.967 `... for the VariableReference class.` |
| variable_reference_subclass_new | scout A `b8_variablereference_setm` | rc 163, 93.967 `... for the K class.` |

## Step 3

Recorded in `docs/superpowers/plans/phase-6-1-gate.md` `## Task 4`: the three `define` probes
answer, so nothing goes under row 9; the b13 probe table (List ITEMS and Routine CALL SIGSEGV 5 of
5 on the oracle, Array ITEMS a different address each of 5 runs; ours rc 120 `receiver_class`, both
engine modes, for those and five more rows).

## Commands and results

- `memcap 8G cargo fmt`: clean.
- `memcap 8G cargo clippy -j 4 --workspace --all-targets -- -D warnings`: exit 0 (at `7d233521f`
  content; the follow-up commit adds data files only).
- `memcap 8G cargo test -j 4 --workspace --no-fail-fast` at `7d233521f`: exit 101, one target,
  `rexx-parse --test sourceline_oracle`: no expectation for the new witnesses. With them it stopped
  at `do_object_compare_array`, Task 3's fix-round-4 witness, which never had one (so this test was
  red at the base). `899db70ac` adds all 54; `cargo test -p rexx-parse --test sourceline_oracle`:
  1 passed. Full rerun at `899db70ac`: exit 0, 144 `test result: ok` lines, none failed.
- `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 -p rexx-exec --test corpus --test
  ir_recorded_oracle`: exit 0 (29 passed 1 ignored; 21 passed).
- `REXX_REFUSAL_SITES_REFRESH=1 memcap 8G cargo test -j 4 -p rexx-exec --test refusal_sites`, then
  the hand columns, then the plain test: 5 passed.
- Callgrind against `a3c2c3c0a` (gate record): fibcall +0.0002%, fibfunc +0.0003%, dispatch
  -1.13%, dispatchclass -1.19%, sendloop -1.68%, rexxcps +0.0001%, emptyloop +0.0003%.
- `whole_groups` not run (not scheduled at Task 4).

## Concerns

1. `object_method` and `expose_receiver` are not deleted: each keeps one site no program reaches,
   now with owner `None`. Task 7's disposition table needs GUARD rows for them, citing the routes
   above.
2. `whole_groups` expectation lines pin text this task removed: TRACE_TraceObject `whole`, both
   modes, expect `a message send to a method context whose scope no longer defines it ...
   TEST_OBJECT_AND_SCOPE` (`concurrency_tests.rs:2926`-`:2934`). Class TEST_CLASS_DEFINE and the
   Method groups, whose refusals were `define`'s, will move too. Task 5's run rewrites them.
3. Divergences met and not fixed, none introduced here (base binary or the same path refused
   before): `Class~enhanced`'s instance is `an Object` in 97.1/97.2 messages where the oracle says
   `enhanced Object`; a method compiled from source text reports the caller's file as its package
   in `>I>` lines (oracle: the method name); `StackFrame~executable` refuses (Phase 9); a
   `Directory~setMethod` entry method adds `Compiled method "UNKNOWN"` traceback lines and reports
   its compiled name as `RUN`; code compiled by `setMethod`/`run` does not see the caller's
   package classes (measured: `self~setMethod('x', 'return .u~new~class~id')` with a `::class u`,
   ours 97.1 on `.U`, oracle `U`); `.Class~enhanced` sends no `NEW` (scout D);
   `.context~package~local~setMethod` refuses (Phase 9). In a method's internal routine,
   `.context~executable` answers the program's Routine where the oracle answers the method
   (measured: `call sub` from a `::METHOD`, ours `Routine`, oracle `Method`).
4. Identity limits: a `Directory~setMethod` copy is not held by anything, so after a collection its
   `.context~executable` is a rebuilt object; when a class takes a name back while its method runs,
   the answer is the live object for the lowest identity running that body.
5. `ObjectMethod` grew by one `ObjRef` (rexx-core), and the `.METHODS` table build now mints
   identities, which shifts `MethodId` numbering during the library bootstrap.
