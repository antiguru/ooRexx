# Task 7.1 report: the reached refusals, and the split by disposition

Status: DONE_WITH_CONCERNS. Base `eeab7f128`. Commits: `acd38baee` (code, tables, witnesses),
`a9bec0326` (refusal-sites.tsv re-derived after rustfmt), and the records and report commit
that follows them.

Records: `docs/superpowers/records/2026-10-18-phase-9d/t7.1/`.

## Rows implemented

| row (refusals.tsv) | now | witness |
|---|---|---|
| context.rs StackFrame EXECUTABLE, a library routine's frame | the `Routine` object: the one `~call`/`~callWith` was sent to, else the one `.routines` and `findRoutine` hold for the library code | `lang/stackframe_library_routine_executable.rex`, `lang/stackframe_merged_library_executable.rex` |
| object_protocol.rs OBJECTNAME= on .nil by a scope override | renames .nil for every rendering (`say`, `~string`, `~objectName`, concatenation, an array's `toString`, error text, `TRACE R`) | `lang/object_scope_override_nil.rex` |
| object_protocol.rs COPY on .nil by a scope override | a distinct `Object` with .nil's behaviour (COPY, START, OBJECTNAME= hidden), its current name, and .nil's hash | `lang/object_scope_override_nil.rex` |

Mechanism, StackFrame: `NativeFrame` gains `executable: Option<ObjRef>` (traced in
`object_roots`, reset in `push_native_frame`), set by `run_library_routine`'s new argument
(`Some(receiver)` from `begin_routine`, `None` from every call by name). Both frame builders
answer it, or `library_routine_object(code)`: `build_native_frame_at` for a live frame, and
`blame_native_routine` (now passed the frame's `executable` and `code`, read before the pop) for
the frame a condition carries. The crate already makes `.routines~r`, `loadExternalRoutine` and
`findRoutine` one object per library code (probe `id1.rex`, `1 1 1` on both).

Mechanism, .nil: `Interp::nil_name` (`Option<Box<[u8]>>`), read by `to_text`, `text_len_inner` and
`try_text`'s `Decoded::Nil` arms. The copy is `Body::Instance { class: .Object, behaviour:
nil_behaviour, name: Some(current name) }`; `hash_value` answers `NIL_HASH` for an instance on
`nil_behaviour`, since `RexxNilObject::hashValue` is a member the clone copies.

Semantics (plan Ruled 13): rexxref `oneof.xml:109-112` gives .nil "only the methods of the Object
class", and `fundclasses.xml:2740-2742` and `:3094` define COPY and OBJECTNAME= for any receiver.
The oracle agrees. StackFrame~executable: rexxref and the oracle agree.

Witnesses: red at the base binary (`witnesses/*.base.{out,err,rc}`: rc 120, `method "COPY" of class
"Object"` and `method "EXECUTABLE" of class "StackFrame"`), identical to the oracle on stdout, stderr
and rc at the head (`witnesses/*.{oracle,head}.*`), oracle output identical over 4 runs each. Each
passes under collect-on-every-allocation (521, 208 and 309 collections, output identical).
Mutations (`mutations.txt`), each red: the live-frame arm answering `None` (merged witness, `live`
lines), ignoring `ran` (library witness, `copy` line), `set_nil_name` a no-op and the `NIL_HASH`
arm removed (nil witness).

## Rows not implemented: rexxref and the oracle part

COPY through `.Message~new(x, .array~of('COPY', .Object))~send` on a built-in class, a Rexx-defined
class, a RexxContext and .RexxInfo (seeds `copy_class_message`, `copy_class_rexx_message`,
`copy_context_message`, `copy_rexxinfo`; outputs in `probes/`).

* Oracle: answers a copy. `copied Class 0 Array 2`; `copied Class 0 ZZ m 0`; `copied RexxContext`;
  `copied RexxInfo 0`. A class copy shares the original's method dictionary and instance behaviour
  until a `define` (probe `cls1.rex`: `c~new~class == .zz`, `c~new~isA(c)` 0; after `c~define('N',
  ...)` `.zz~new~hasMethod('N')` 0; a `.zz~define` afterwards does not reach `c`'s instances).
  `RexxClass::copyRexx` (`classes/ClassClass.cpp:161-171`) carries the comment "An override for the
  copy method to keep Class objects from being copied"; the scope override reaches
  `RexxObject::copyRexx` instead.
* rexxref: `fundclasses.xml:63-65`, "Note that the copy method is forbidden for Class and all other
  class objects, and will result in an error." `utilityclasses.xml:7934-7942` (RexxInfo): "Only
  one instance of the RexxInfo class can be obtained via the .REXXINFO environment symbol, other
  instances cannot be created or copied." `utilityclasses.xml:7549` and `:7574` (RexxContext): the source comments "new() and copy() are
  forbidden" and "copy() is forbidden", and `:7545`, "They cannot be directly created by the user."
* Derived answer: 93.970, as the plain sends answer on both sides today (probe `plain.rex`: `COPY
  method is not supported for object The Array class.`, `... a RexxContext.`, `... a RexxInfo.`).
* Held: `Loud::uncopyable_copy`, owner Phase 9, rc 120. No Deviation written. If Moritz rules for
  the derived rule, the change is the three `copy_object` arms returning
  `Raised::copy_not_supported` over the receiver's string value.

## The split (template's last instance)

`grep -rn 'Loud::native_method(' rust/crates/rexx-exec/src` now finds nothing; the constructor is
gone. `awk -F'\t' '!/^#/ && $NF != "disposition" {print $NF}' refusals.tsv | sort | uniq -c` gives `guard`,
`implement` and `oracle-crash` only: no row is ruled Deviation or later phase.

| site | constructor | disposition |
|---|---|---|
| dispatch.rs `invocable` | `unbound_method` | GUARD, the dispatch.rs row |
| hash.rs `refusal_outside` | `hash_collection_without_store` | DEVIATION, oracle-crashes entry 50 |
| context.rs `frame_executable` (what is left: a package hook's frame, one built from a failure site with none) | `frame_without_executable` | GUARD, the context.rs row |
| object_protocol.rs `copy_object`, Directory and StringTable `Body::Native` | `interpreter_table_copy` | GUARD, that row |
| object_protocol.rs `copy_object`, class object, RexxContext, RexxInfo | `uncopyable_copy` | Phase 9, held for the ruling above |
| object_protocol.rs `native_object_name_set` | removed | implemented |

hash.rs: the guard rows (condition object, a subclass whose INIT does not forward, the package
tables) and the crash row reach one site, which cannot tell them apart; the guard receivers answer
before it, so the site keeps one constructor, DEVIATION, and its row says so.
`the_scanner_reads_each_owner_form` now asserts the phase-valued form on `uncopyable_copy`;
`the_scanner_finds_constructors_and_only_constructors` names `unbound_method`.
`src/environment/tests.rs`'s refusing destination (entry 50's shape) expects the new text, and
`oracle-crashes.txt` entry 50's sentence quoting this crate's refusal is corrected.

## Checks

| command (from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t7-target`, `memcap 8G`, `-j 4`) | exit |
|---|---|
| `cargo fmt --all --check` | 1: `crates/rexx-exec/tests/stream_model.rs` only, unformatted at `eeab7f128` and untouched here |
| `cargo clippy -p rexx-exec --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `cargo test --release -p rexx-exec --lib` (`checks-lib.txt`) | 0, 1167 passed |
| `cargo test --release -p rexx-exec --no-fail-fast --test refusal_sites --test refusal_dispositions --test representation_refusals --test owners --test closed_phases --test phase_ten_reasons` (`checks-tables.txt`) | 0 |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 0, 1152 of 1152 matching |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 |
| `cargo test --release -p rexx-exec --no-fail-fast --test method_bodies --test native_entries --test library_routine_memory --test collect_stress --test concurrency_tests --test gate_table_d` (`checks-touched.txt`) | 101: `collect_stress`'s `the_l0_subset_passes_again_under_collect_on_every_allocation` only, red at `eeab7f128` too |

The whole-suite run was not made: the dispatch's check list does not name it.

## Perf

`callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -j 4 -p "startup interpret parse churn
sayloop" -o .../cg-9d/t7.1-run pin=.../rexx-run parent=cg-9d/t7.1/rexx-parent head=cg-9d/t7.1/rexx-head`,
parent built from `eeab7f128`, head from `acd38baee`'s code, exit 0. Own delta, libc subtracted,
head less parent: startup +0.0002, interpret 0.0000, parse 0.0000, churn 0.0000, sayloop +0.0001.
Head against the pin, libc subtracted: startup -0.1206, interpret -4.6330, parse -0.1416, churn +0.1068, sayloop
-0.0639. New refusal constructors are `#[cold]`.

## Concerns

1. **A copied library `Routine` reached by name answers the canonical object.** `probes/addr.rex`
   (`.context~package~addRoutine('X', .routines~r~copy)`) and `probes/wr.rex` (`.routines['X'] =
   copy`): the oracle's frame answers the copy (`ROUTINE X 1 0`), this crate the canonical object
   (`ROUTINE X 0 1`). Both routes end at `Resolved::MergedLibraryRoutine(code)`, which drops the
   object; base refused these loudly. Carrying the object needs it rooted in the call-site
   `Resolved` cache (kept for the body's life, as the oracle's instruction keeps it), or a lookup
   at frame time. Proposed owner: a Task 7.1 fix round.
2. **COPY on a class, RexxContext or RexxInfo by a scope override** waits on Moritz (above).
3. **`collect_stress` is red at base.** `stress.txt`: `package_option_writes.rex` and the
   `raise_*` and `reply_as_the_last_clause.rex` programs panic at `not_in_arena`'s "a live value"
   (`dispatch.rs:1799`) under collect-on-every-allocation, at `eeab7f128` and here alike; the
   panic ends the test at its first such program, so the rest of the subset never runs there.
   Proposed owner: the controller.
4. **`stream_model.rs` fails `cargo fmt --check` at base** (Task 6.15's last commit). Left alone:
   another agent may hold it.
5. **`.Routine~new(...)~call`'s frames differ, unrelated to this task.** `probes/rn.rex`: the oracle
   lists `ROUTINE m` then `METHOD CALL`, this crate `ROUTINE CALL` alone; `sf9.rex`'s `viam` lines
   show it. No library is involved. Proposed owner: the controller.
6. **.nil's name is per interpreter instance**; the oracle's `TheNilObject` is one per process. Only
   a process running several instances can tell.
7. Plain `.nil~copy` and `.nil~objectName=` are 97.1 on both sides (`nil_hidden_methods.rex`),
   which rexxref's "only the methods of the Object class" does not say. Not acted on.
