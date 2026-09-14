# Task 9 report: a native `UNINIT`

Committed at `0b586d0c7b8ee7f5d3564a4c366861b5b9607b2a`, one commit, on top of
`75291495f`. The report itself is not committed: `.gitignore:30` ignores
`.superpowers/`, as Tasks 1 to 8 found.

Files added: `rust/corpus/lang/library_uninit_collected.rex` and `.env`,
`rust/corpus/lang/library_uninit_termination.rex` and `.env`,
`rust/corpus/lang/library_uninit_twice.rex` and `.env`, and the three matching
`rust/crates/rexx-parse/tests/sourceline_oracle/library_uninit_*.txt`.
Files modified: `rust/corpus/phase-8.txt`,
`rust/crates/rexx-exec/src/lib.rs`,
`rust/crates/rexx-exec/src/dispatch/library.rs`.

---

## 0. The headline: the dispatch was wrong about what was missing

**A native `UNINIT` already ran at finalisation before this task touched
anything.** Measured at `75291495f` with `target/release/rexx-run`, against
the oracle under the standard wrapper from a fresh empty directory, three
descriptors kept apart: an object of a class whose `::METHOD uninit EXTERNAL`
names `librxregexp.so` has its native finaliser run when the object is
collected, when it is still reachable at program end, and a second time when
one has already been sent by hand. All three agree with the oracle byte for
byte, and did so before the commit.

The reason there was nothing to build is that the finaliser is not a special
path. `::method uninit external "LIBRARY rxregexp RegExp_Uninit"` installs
into the class dictionary like any other library-backed method, so
`ClassGraph::check_uninit`'s `dict.has_method("UNINIT")` sets the class flag,
the collector resurrects the instance, and `Interp::run_one_uninit`'s
`send_message(object, UNINIT, ..)` resolves to `Invocable::Library` and reaches
`Interp::run_library_method`. Task 8 built the last of those, and Task 8's own
open list said "`RegExp_Uninit` never runs" on the strength of the leak being
invisible rather than on a measurement.

So what this task actually owes is Steps 1, 2 and 3 as *evidence and
construction*, not as new dispatch: witnesses that pin the behaviour so it
cannot regress silently, the ordering constraint made structural, and the
allocation question answered.

---

## 1. Can the corpus see the finaliser? Yes, and here is how that was settled

The dispatch's question was the right one to ask first.
`extensions/rxregexp/rxregexp.cpp:93` is `RexxMethod1(int, RegExp_Uninit,
CSELF, self)` and its body deletes the automaton and then, at `:101`, calls
`context->DropObjectVariable("CSELF")` under the comment `// ensure we don't do
this twice` at `:100`. It writes to no descriptor. **But the drop is not
invisible**: `CSELF` is an object variable in the pool of the scope the native
method belongs to, and a written method in that same scope can read it back.

Measured against the oracle, rc 0, stderr empty:

```rexx
r = .Re~new('a*b')
say 'before' r~probe~class~id
r~uninit
say 'after' r~probe~class~id
::class Re subclass Object
::method init external "LIBRARY rxregexp RegExp_Init"
::method uninit external "LIBRARY rxregexp RegExp_Uninit"
::method probe
  expose CSELF
  return CSELF
```

```
before Pointer
after String
```

`String` is the uninitialised read answering the variable's own name, so the
drop happened. That makes the extension's own call observable from Rexx, and a
corpus differential can therefore see it.

**The first attempt could not, and the reason is worth recording.** Task 7's
scope measurement reads the base scope with
`.RegularExpression~define('BASECSELF', .Method~new(..))`, and `~define` is
`method "PROBE" of class "RE" is not implemented (Phase 5)` here. A written
`::method probe` inside the same `::CLASS` has the same scope and needs
nothing unimplemented.

**Collection-driven, not just an explicit send.** A Rexx subclass whose own
`::method uninit` forwards to the native one puts a printing frame either side
of it, so the finaliser's effect is visible at the moment the sweep runs it.
Oracle, rc 0, stderr empty, for the `drop` + `call gc 'force'` shape:

```
live Pointer
sub before Pointer
sub after String
done
```

The three questions the dispatch listed, each answered by running rather than
by assuming:

* **A Rexx `::method uninit` on a subclass, and its ordering against the native
  one.** The subclass's method is the only one the send resolves to; the native
  one runs only where the subclass forwards, and `forward class (super)
  continue` is what runs it. Both interpreters agree on the sandwich.
* **Does the oracle run a native `UNINIT` for an object still live at program
  end?** Yes. The same program without the `drop` and the `gc` prints the same
  three lines, and nothing after them.
* **Is double finalisation reachable, and what does the oracle do?** Reachable,
  and the oracle sends the second one. An explicit `r~uninit` does not take the
  object off the collector's list, so `drop r` + `call gc 'force'` sends
  `UNINIT` again; the extension's own guard is what makes that safe, because
  `CSELF` was dropped and arrives as a null pointer. Oracle, rc 0:
  `sub Pointer` / `explicit String` / `sub String` / `done`. **Finalisation
  after library unload is not reachable**, which is section 2's subject.

---

## 2. The ordering constraint, made structural

### What would happen if it did not hold

`libloading::Library` runs `dlclose` when it is dropped. The mapping that goes
away holds the extension's `UNINIT` stub, every entry point
`NativeMethodEntry` addresses, and the heap block a `CSELF` points at. A
library released while an object of a class it contributed a method to is
still reachable therefore leaves `Interp::run_one_uninit` sending `UNINIT` to
that object, `run_library_method` reaching `entry.call`, and the call jumping
to an unmapped address. The termination sweep is the worst case, because it is
the last thing the interpreter does and the objects it finalises are by
definition ones nothing else has released.

### What was already true, and why it was not enough

At `75291495f`, `Interp::libraries` was a `HashMap<Vec<u8>, LibraryLoad>` and
nothing removed from it, so the constraint held. It held by *absence* -- the
weakest kind of guarantee, one that a later `remove`, `clear` or replacing
`insert` silently ends. Task 8's own open list said as much: "holds trivially
and for the wrong reason".

### What the commit does

`Interp::libraries` is now a `Libraries`, a type whose stored map is private
and which exposes `get` and `hold` and nothing else. `hold` is
`entry(name).or_insert(load).clone()`, so it writes only where nothing is held
and answers the held value; there is no `remove`, no `clear`, no `get_mut`, no
`Default`, so `std::mem::take` will not compile either. A name's answer is
written once and afterwards neither removed nor replaced, so an `Rc` handed to
a `LibraryBinding` is never the last one while the interpreter lives, and the
only release is the interpreter's own drop -- at which point nothing sends
messages.

The two halves the borrow checker already carried, and which are Task 2's and
Task 8's rather than this task's, are worth naming because together with the
above they close the constraint:

* `NativeMethodEntry` keeps its address in a private field, is not `Clone`, and
  is only ever borrowed from `&Library`, so an entry point cannot outlive the
  mapping. `rexx-api/src/load.rs` carries two `compile_fail` doctests for that.
* `Interp::run_library_method` clones the `Rc` before it borrows the row, so a
  callback that dropped every other reference mid-call cannot unmap the code it
  is running in.

`a_loaded_library_outlives_the_finaliser_sweep` asserts the runtime half with a
`Weak`: resolve, downgrade, drop the strong clone, run `collect_now` and
`run_termination_uninits`, and the weak still upgrades. Its own control is the
line after, which drops the interpreter and requires the weak to go dead --
that is what says the watch can see a release at all, and that the interpreter
was the only holder in the assertion above it.
`a_held_library_is_not_replaced_by_a_later_answer` asserts the write rule, with
a control on a second name that does take the answer the first refused.

**Superseded by fix round 1.** As committed at `0b586d0c7` this paragraph was
wrong about the boundary: `Libraries` and `Interp::resolve_library` were in the
same module, so the private field was fully visible there and a `remove` on it
compiled. Section 9 has the finding, the move that closes it and the control
that sees it.

**Derived, and the command is committed here.** Everything that touches the
field: `grep -rn 'self\.libraries' crates/rexx-exec/src` answers
`lib.rs:4503` (`get`) and `lib.rs:4513` (`hold`). Everything that touches the
map inside the type: `grep -n 'self\.held' crates/rexx-exec/src/lib.rs`
answers `lib.rs:2129` and `lib.rs:2136`, the bodies of those two methods.

---

## 3. Allocation inside a finaliser

**The crate's written `UNINIT` takes no precaution against allocating, and it
needs none, because it never runs inside a collection.** `Heap::collect` only
marks: an object whose class carries the flag and which the sweep found dead is
resurrected and reported in `CollectStats::pending_uninit`
(`rexx-core/src/heap.rs:157`-`:191`), which is `MemoryObject::checkUninit`
(`interpreter/memory/RexxMemory.cpp:274`) doing the same thing --
`setReadyForUninit()` and `pendingUninits++`, no send. `Interp::collect_now`
appends that list to `Interp::uninit_ready` and returns.

The sweep is a separate call afterwards. Derived, and committed here:
`grep -rn 'run_ready_uninits\|run_termination_uninits' crates/*/src` answers,
outside the two definitions in `dispatch.rs`, their doc references and three
test lines, exactly `crates/rexx-exec/src/builtin/state.rs:129` (`GC('force')`)
and `crates/rexx-exec/src/lib.rs:5692` (termination). Neither is inside
`Interp::alloc_with` or `Interp::collect_if_due`, so a finaliser body is on
the same footing as any other method body: it may allocate, and an allocation
inside it may itself collect.

So the answer to Step 2 is not "the existing `UNINIT` does nothing about it,
record that". It is that the question does not arise, and the structural reason
is that the collector marks and the interpreter sweeps. **What I did** was
assert it rather than write it down:
`the_native_finaliser_answers_the_same_under_a_collection_at_every_allocation`
runs `library_uninit_collected.rex` through
`run_program_collect_every_alloc`, which collects before every allocation
including the ones the finaliser makes, and requires the same three
descriptors as the ordinary run plus a non-zero collection count.

`collect_stress` cannot say this: it reads `phase-8.txt` last and
`the_l0_subset_passes_again_under_collect_on_every_allocation` panics at
`dispatch.rs:1506` on a pre-existing defect before it gets there, which is
Task 8's finding and still true.

---

## 4. The witnesses

Three, all in `phase-8.txt`, each carrying the `LD_LIBRARY_PATH={oraclelib}`
sidecar the other library witnesses carry. Every oracle transcript below was
taken under

```
( cd "$D" && ulimit -v 1048576; \
  LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  timeout 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx "$D/$f.rex" ) \
  >"$D/$f.out" 2>"$D/$f.err"
```

from a fresh empty directory, three descriptors kept apart, never `2>&1`.
`corpus/oracle-crashes.txt`'s entry titles were read before running anything;
none of these is one of them.

### `lang/library_uninit_collected.rex` -- rc 0, stderr empty

```
live Pointer
sub before Pointer
sub after String
done
```

The driven sweep: the object is dropped, `GC('force')` collects and finalises
it, and the program runs on afterwards.

### `lang/library_uninit_termination.rex` -- rc 0, stderr empty

```
live Pointer
sub before Pointer
sub after String
```

The termination sweep: the object is reachable at the last clause, so nothing
in the program's own life finalises it.

### `lang/library_uninit_twice.rex` -- rc 0, stderr empty

```
sub Pointer
explicit String
sub String
done
```

The second finalisation, reaching `RegExp_Uninit`'s null-`CSELF` guard.

Each was run five times on the oracle and three times through
`target/release/rexx-run`; all eight runs of each hash the same on stdout.

**A broken implementation does not produce these bytes.** `sub before Pointer`
before `sub after String` is what says the drop was the finaliser's and not the
absence of an `INIT`; a run in which the native `UNINIT` did not fire prints
`Pointer` on both lines, which control A confirms.

**The `sourceline_oracle` expectations** were regenerated with the driver in
that test's own module comment, run against copies of the three files placed
**outside the repository**, because the Global Constraints forbid
`.Package~new` on a file inside it. The bodies are byte-identical to the corpus
sources (`diff <(tail -n +2 ..txt) ..rex` is empty for all three) and the
counts are 24, 18 and 22.

**One fragility, recorded rather than fixed.** Two of the three depend on the
forced collection running the finaliser before the next `say`, and that is
collection *ordering*, which is a licensed divergence
(Moritz 2026-09-01: "it just needs to be one of the many correct orderings").
They match today and matched on every one of the eight runs, and the existing
`class_uninit_gc.rex`, `uninit_instance_collected.rex` and
`setmethod_uninit.rex` carry the same dependency with no note, so no new
convention was invented for these. Section 7 records two shapes where the
ordering does already diverge.

---

## 5. Controls

Predictions were written to `scratchpad/predictions-task9.md` at 06:22:55, in
one block, before any control was applied; the file's mtime is that time and
the first control ran after it. Baselines, on the tree with the witnesses and
the type already in it: `cargo test -p rexx-exec --lib` 814 passed 3 failed
(the three pre-existing `ir::drive::tests` cases), corpus `528 of 528
matching`.

Each control was one edit undone by re-editing, never by `git checkout`.
`sha256sum -c` against hashes taken beforehand reports `OK` for both
`crates/rexx-exec/src/lib.rs` and `crates/rexx-exec/src/dispatch/library.rs`
after each of A, B, C and D, and `git status --short` showed only this task's
own files afterwards.

| control | edit | predicted | observed | verdict |
|---|---|---|---|---|
| A | `Host::drop_object_variable` is a no-op | corpus `525 of 528`, the three new witnesses on stdout only; `library_method_external` unchanged; `--lib` 813/4 with the stress test the new failure; `rexx-api` still 0 | the corpus binary **died without printing a count** -- `free(): invalid pointer`, `signal: 6, SIGABRT` in this run; the reviewer's run of the same edit reported SIGSEGV, section 9. Run one at a time through `rexx-run`: `library_uninit_collected` and `library_uninit_termination` print `sub after Pointer`, `library_uninit_twice` is rc 134 with `free(): invalid pointer` on stderr and empty stdout, `library_method_external` unchanged. `--lib` 813/4, the new failure exactly the one named. `rexx-api` 0 | corpus **count falsified**, per-program divergence **confirmed** for all three, the mechanism on the third **falsified**, `--lib` and `rexx-api` **confirmed** |
| B | `Interp::collect_now` stops extending `uninit_ready` from `stats.pending_uninit` | `library_uninit_collected` and `library_uninit_twice` diverge; `library_uninit_termination` does **not**, because it reaches the sweep through `Heap::take_uninit_flagged`; total below 526 with a larger failing set, extent not predicted; `--lib` fails at least the stress test | corpus `520 of 528`; the diverging set is `class_behaviour_snapshot_delete`, `class_uninit_gc`, `library_uninit_collected`, `library_uninit_twice`, `object_copy`, `setmethod_uninit`, `uninit_instance_collected`, `uninit_nested_collection`. `library_uninit_termination` green. `--lib` 813/4, the stress test the new failure | **confirmed**, every part |
| C | `self.libraries.remove(name)` added to `Interp::resolve_library` | `cargo check -p rexx-exec` fails, `E0599`, no method named `remove` | exit 101, `error[E0599]: no method named 'remove' found for struct 'Libraries' in the current scope` | **confirmed and worthless** -- it probed the method table and not the field, and the field was reachable; see section 9 |
| D | `Libraries::hold` uses `insert` instead of `entry(..).or_insert` | `a_held_library_is_not_replaced_by_a_later_answer` fails, `--lib` 813/4; corpus **unchanged** at 528, because `resolve_library` reads `get` first and never calls `hold` twice for one name | exactly that: `--lib` 813 passed 4 failed with that test the new failure, corpus rc 0 and `528 of 528 matching` | **confirmed**, both parts |

**A is the one that found something, and it is the most important result in
this report.** With `DropObjectVariable` inert, `RegExp_Uninit` deletes the same
automaton twice on the second finalisation and glibc aborts the process. That
proves `library_uninit_twice.rex` really does reach the extension's guard --
the guard is *our* `DropObjectVariable` arming it, not something the extension
does on its own -- and it proves the witness's bytes are unreachable to a
broken implementation in the strongest possible way. It also means a future
regression there kills the corpus binary rather than naming a mismatch, which
is the hazard the dispatch warned about, arriving from a direction I had not
predicted: not a panic in a callback, but the *extension's* own heap
bookkeeping reacting after our callback returned the wrong answer. **Which
signal it dies with is not stable**, section 9.

**C and D are the pair that makes the construction's claim honest.** C says the
forbidden state does not compile. D says the test that pins the write rule is
measuring the type and not the interpreter's current call pattern, and that the
corpus cannot see the difference -- which is exactly right, because the state
the type forbids was already unreached before this commit. Neither control on
its own would have said that.

**No control reddened only by abort where the answer mattered.** B, C and D are
read from a corpus count, a compiler error and a test assertion after the call
returned. A's abort was itself the finding, and its per-program divergences
were read from three separate one-program runs.

---

## 6. Commands and exit statuses

Every status read unpiped, from `rust/`, at the committed tree
`0b586d0c7` unless the row says otherwise.

| command | exit |
|---|---|
| `cargo fmt --all` | 0 |
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test -p rexx-api` | 0 |
| `cargo test -p rexx-core --test unsafe_sites` | 0 |
| `cargo test -p rexx-parse` | 0 |
| `cargo test -p rexx-exec --lib` | 101 |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 |
| `cargo test -p rexx-exec --no-fail-fast` | 101 |

* `cargo test -p rexx-api` reports **106** `ok` result lines, the same as Task
  8's. Nothing in that crate changed.
* `cargo test -p rexx-core --test unsafe_sites` reports 2 passed. **No `unsafe`
  was added**: `git diff 75291495f..0b586d0c7 -- rust/crates | grep '^+' | grep
  unsafe` answers nothing.
* `cargo test -p rexx-exec --lib` reports **814 passed, 3 failed**, the three
  pre-existing `ir::drive::tests` cases -- the dispatch's expected 811 plus the
  three added here (`a_held_library_is_not_replaced_by_a_later_answer`,
  `a_loaded_library_outlives_the_finaliser_sweep`,
  `the_native_finaliser_answers_the_same_under_a_collection_at_every_allocation`).
* The corpus gate reports **528 of 528 matching**, 29 passed 0 failed 1
  ignored. The dispatch's expected 525 was the subset before this task, and
  three programs were added.
* `cargo test -p rexx-exec --no-fail-fast` fails in three binaries and **the
  failing set is exactly Task 8's**: `--lib`'s three, `collect_stress`'s
  `a_loops_per_pass_roots_outlive_the_pass_and_not_the_loop` and
  `the_l0_subset_passes_again_under_collect_on_every_allocation`, and
  `refusal_sites`'s `the_table_holds_every_constructor_the_source_defines`.
  1480 `ok` result lines against Task 8's 1476.
* `cargo test -p rexx-parse` covers
  `sourceline_matches_the_interpreter_for_every_corpus_program`, which is what
  reads the three new expectation files.

The gates run in this task are the ones the dispatch named. The phase's four
Global-Constraint gates -- including the debug `REXX_CORPUS_GATE=1 cargo test
-j 4 --workspace --no-fail-fast` -- were **not** run here and are still owed at
the phase's closing commit.

---

## 7. What the brief and the dispatch got wrong

1. **"This task makes a native `UNINIT` run at finalisation."** It already ran,
   at `75291495f`, and the commit adds no dispatch. Section 0.

2. **The brief's Step 2 anticipated the wrong answer.** "Find out what this
   crate's existing Rexx `UNINIT` does about [allocating]; if it does nothing,
   that is a finding to record rather than a licence." It does nothing, and it
   is not a finding of that shape: a finaliser never runs inside a collection
   here, so there is nothing for it to do. Section 3.

3. **The dispatch's doubt about the corpus was reasonable and wrong.**
   `RegExp_Uninit` does write something a Rexx program can read -- its
   `DropObjectVariable("CSELF")` -- and three corpus witnesses see it. Section
   1. The fallback the dispatch offered (a crate-level test recording the call
   on our own side) was not needed and was not built.

4. **The brief's file list is wrong in both directions.**
   `rexx-api/src/invoke.rs` was not touched; nothing in `rexx-api` was. The
   collector's uninit path in `rexx-exec/src` was not touched either. What
   changed is `rexx-exec/src/lib.rs` (the `Libraries` type),
   `rexx-exec/src/dispatch/library.rs` (three tests), and the corpus.

5. **Two gc-ordering divergences were found while probing, both licensed and
   neither fixed.** A program finalising several objects at one `GC('force')`
   prints one of them after the following clause on the oracle and all of them
   before it here; a finaliser that itself sends into the library defers the
   whole sweep past the following clause on the oracle. Both are ordering only:
   every line is present on both sides, in a different order, at rc 0 with
   empty stderr. `oorexx-gc-ordering-divergence-licensed` covers exactly this.
   **The mechanism**, which explains why the shapes that match do match: the
   oracle drains the pending-uninit queue at more safe points than this crate
   does, including after every native activation returns
   (`interpreter/execution/NativeActivation.cpp:1361`) and at the end of a
   Rexx activation (`interpreter/execution/RexxActivation.cpp:705`), where this
   crate reaches the sweep only from `GC('force')` and from termination. The
   three committed witnesses were chosen from shapes where the two agree.

6. **Task 8's open item 4 was stated as fact on no measurement.**
   "`RegExp_Uninit` never runs. `library_method_external.rex` matches the
   oracle because the extension prints nothing when it frees an automaton, so
   the leak is invisible." The second sentence is true of that program and the
   first does not follow from it; the finaliser was running the whole time.
   This is the "unreachability claims need running" shape one step over: a
   claim of absence inferred from an absence of evidence.

---

## 8. Open, for whoever comes next

1. **A regression in `Host::drop_object_variable` kills the corpus binary, and
   the signal it dies with is not stable.** Control A: with the drop inert,
   `library_uninit_twice.rex` deletes the same automaton twice inside
   `librxregexp.so`. Three observations of that one edit: the corpus binary
   under `REXX_CORPUS_GATE=1` gave `free(): invalid pointer` and `signal: 6,
   SIGABRT` in my run and SIGSEGV in the reviewer's, and the standalone
   `rexx-run` of the witness gave rc 134 with `free(): invalid pointer` on
   stderr for both of us. **The harness classifies none of these**: the corpus
   comparison never runs, so there is no mismatch line and no program name.
   The witness is right and worth keeping -- what kills the process is the
   extension's allocator reacting to our wrong answer -- but whoever meets a
   corpus binary dying with no count should read it as "an object variable the
   extension dropped is still set", not as a harness fault, and should not
   expect a particular signal.

2. **`Interp::libraries` is insert-once by type, and `library_externals` is
   not.** The bindings map is a plain `HashMap<MethodId, LibraryBinding>` that
   nothing prunes today. If a later phase prunes it -- when a class is expunged,
   say -- the `Rc` it drops is not the last one, because `Libraries` still holds
   one, so the constraint survives. That is the whole reason the type is where
   it is rather than in the bindings map.

3. **Nothing unloads a library, deliberately.** The interpreter's own drop is
   the only release, and no `UNINIT` is sent then. A phase that wants
   `dlclose` before the interpreter ends owes a reachability argument this
   type does not currently need.

4. **Task 8's open items 1, 2, 5, 6, 7 and 8 are untouched** -- the routine
   half of the two-call protocol, the unwitnessed 98.982, `Host::string_value`
   narrowing a condition, the stale `refusal-sites.tsv`, and the
   `Failure::Signature` spelling. None is this task's.

5. **The phase's four Global-Constraint gates are still owed**, section 6.

---

## 9. Fix round 1

Committed at `d39dc3194a3bd30d75b7668586a9c80c19870dfa`, on top of `0b586d0c7`. Files added:
`rust/crates/rexx-exec/src/libraries.rs`. Files modified:
`rust/crates/rexx-exec/src/lib.rs`,
`rust/crates/rexx-exec/src/dispatch/library.rs`.

### 9.1 Finding 1: the guarantee was documented at the site that matters

The reviewer inserted `self.libraries.held.remove(name);` into
`Interp::resolve_library` at `0b586d0c7` and `cargo check -p rexx-exec` exited
0. **Reproduced here independently**, in a detached worktree at `0b586d0c7`
under the session scratchpad, same edit, same result: exit 0, the workspace
compiles. `Libraries` was declared in `lib.rs` and `resolve_library` is a
method on `Interp` in the same file, so the field's privacy bought nothing
against the one module that resolves libraries. `held.clear()` and
`std::mem::take(&mut ..held)` were open the same way.

The fix is a module. `Libraries` now lives in
`rust/crates/rexx-exec/src/libraries.rs` with `new`, `get` and `hold`
`pub(crate)` and the map private to that file; `lib.rs` declares `mod
libraries;` and `use libraries::Libraries;`. Nothing else moved:
`LibraryLoad` stays in `lib.rs`, and the new module reaches it through
`use crate::LibraryLoad`.

**Why control C could not see this, which is the lesson.** C wrote
`self.libraries.remove(name)` -- a *method* call, from outside. Rust resolved
that to `E0599` and the control read green. But rustc's own note on that error
says "one of the expressions' fields has a method of the same name", and that
note was the hole: the method table was empty and the field was not. A probe
written from where the privacy already held could not distinguish a boundary
that holds everywhere from one that holds only away from the site that matters.
**Write a privacy control from inside the module that would abuse the field**,
not from a module that already cannot.

### 9.2 The redone control

Predictions were written to `scratchpad/predictions-task9-fix1.md` at 06:54:04,
before any of the five ran. Each is one edit to
`Interp::resolve_library` -- except E4 -- checked with
`cargo check -p rexx-exec` and undone by re-editing.

| part | edit | predicted | observed | verdict |
|---|---|---|---|---|
| E1 | `self.libraries.held.remove(name);` | exit 101, `E0616` | exit 101, ``error[E0616]: field `held` of struct `Libraries` is private`` | **confirmed** |
| E2 | `self.libraries.held.clear();` | exit 101, `E0616` | the same | **confirmed** |
| E3 | `std::mem::take(&mut self.libraries.held);` | exit 101, `E0616` | the same | **confirmed** |
| E4 | `self.held.remove(name);` at the top of `Libraries::hold`, inside the module | exit **0** | exit 0 | **confirmed** |
| E5 | `let _ = std::mem::take(&mut self.libraries);` | exit 101, and not `E0616` | exit 101, ``error[E0277]: the trait bound `Libraries: std::default::Default` is not satisfied`` | **confirmed** |

E4 is the part that makes E1 to E3 mean anything: without it they are equally
consistent with a probe that fails to compile for a reason other than privacy,
and the hole this round closes is exactly the case where the same-module edit
compiles. E5 covers the whole-value replacement the field's privacy does not
reach, and it is refused for a different reason -- no `Default`, so no
`mem::take`. `mem::replace` with a freshly built `Libraries` is still open and
is not closed by anything here.

`sha256sum -c` against hashes taken beforehand reports `OK` for
`crates/rexx-exec/src/lib.rs`, `crates/rexx-exec/src/libraries.rs` and
`crates/rexx-exec/src/dispatch/library.rs` after all five, and the temporary
worktree used for 9.1 was removed with `git worktree remove --force`.

### 9.3 Finding 2: control A's regression mode

Corrected in place at open item 8.1 and in the control A row of section 5. The
short version: three observations of one edit gave `free(): invalid pointer`
with `signal: 6, SIGABRT` (my corpus run), SIGSEGV (the reviewer's corpus run)
and rc 134 with `free(): invalid pointer` (both standalone runs). My original
text named SIGABRT as if it were the mode. The harness classifies none of them
-- the comparison never runs, so no program is named.

### 9.4 Finding 3: the doc at `dispatch/library.rs`

`**A native `UNINIT` allocates**` was false about `RegExp_Uninit`, which
allocates nothing. The doc now opens `**A finalizer allocates**` and says which
part of the witness does it: the Rexx subclass finalizer around the native one,
which sends and says.

### 9.5 Commands and exit statuses, fix round

Read unpiped, from `rust/`, at the committed tree.

| command | exit |
|---|---|
| `cargo fmt --all` | 0 |
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test -p rexx-api` | 0 |
| `cargo test -p rexx-core --test unsafe_sites` | 0 |
| `cargo test -p rexx-exec --lib` | 101 |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 |

`rexx-api` reports 106 `ok` result lines, `unsafe_sites` 2 passed, `--lib`
**814 passed 3 failed** with the three pre-existing `ir::drive::tests` cases
and nothing else, and the corpus gate **528 of 528 matching**, 29 passed 0
failed 1 ignored. `git diff 0b586d0c7..HEAD -- rust/crates | grep '^+' | grep
unsafe` answers nothing.
