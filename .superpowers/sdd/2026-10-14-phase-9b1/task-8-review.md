# Phase 9b1 Task 8 review: the side-map sweep (d965c149a..d81bacfad)

Reviewer probes, the probe runner and the callgrind tables are in `task-8-review-files/` beside this
file. Raw `.cg` files: `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-8-review/` and
`.../task-8-review-base/`. Oracle runs used the standard wrapper from fresh empty directories under
`/tmp/t8rev/oracle-run*`. Crate runs under collect-every-allocation went through
`task-8-review-files/review_probes.rs` (`run_program_collect_every_alloc`). That runner was built in a
git-archive copy of d81bacfad at `/tmp/t8rev/tree` with `CARGO_TARGET_DIR=/tmp/t8rev/target`, under
`memcap 4G`. The copy and its target dir are deleted.

### Spec Compliance

- ✅ Spec compliant for Steps 1-4. The table matches its committed enumeration. Every per-object row
  is in state 1, 2 or 3, or is reported. Step 3's items are closed with probes. Step 4's rows are
  removed or retagged.
- ⚠️ The three reported leaks wait on a controller ruling. Item 5 below has the oracle figures and a
  size for each fix.

### Strengths

- `side-maps-build.py` fails on an unclassified row or a classification that names no row, so the
  table cannot drift silently from its enumeration.
- The retired-behaviour fix is large: `cm5` went from 1174604 kB to 53012 kB at 80000.
- `prune_freed_classes` is moved cold to address a measured codegen effect (churn +0.655% to
  +0.392%), and the report explains why.
- The `Class~new` charge is witnessed red on base (`class cleared: 0`) and green on head. I
  re-measured it: oracle `1` in 2 of 2 runs, base `0`, head `1`.

### Checks run

1. **side-maps.tsv completeness.** I re-ran `task-8/enumerate-maps.sh` from `rust/`. It gives 136
   rows, the same struct, field and type columns as the table's 136 data rows, and the same sites.
   - Map-like fields: the awk's `(Map|Set|Vec)<` already matches `BTreeMap`, `IndexMap`, `NameMap`
     and `Vec<(..)>`.
   - Wrapped field declarations: a grep for types that wrap onto the next line, over all 16 listed
     files, found none.
   - `Rc<RefCell<..>>` tables: `grep -rnE "Rc<RefCell<.*(Map|Set|Vec)"` over rexx-exec, rexx-core,
     rexx-classes and rexx-api finds nothing.
   - `thread_local!`/`static` tables: all are counters, flags or name tables. None is keyed by an
     object.
   - Interp members the script does not open: `TextNumbers`, `OptionWrites`, `PackageLocalMiss`,
     `Queue`, `Input`, `Pool`, `Pinning`. None holds a per-object map. `TextNumbers.keys` is a
     fixed `[ObjRef; 32]`, which the `(Map|Set|Vec)<` filter cannot see. Its keys are inline-text
     values, so it is not per-object.
   - Other per-object maps outside the listed structs: `BehaviourTable` (rexx-core `behaviour.rs`)
     has no user outside its file. `Body::Native` entries are state 1. Activation `streams`, `extra`
     and `user` are per-frame.
   - Result: complete. Every per-object row is in state 1, 2 or 3, or is reported.
2. **Ownership rule under collect-every-allocation, with a `copies('x', 100000)` x300 filler.**
   Every shape matches the oracle in 3 of 3 runs:
   - `own/p1` (setMethod, then `unsetMethod` of itself mid-call, then reads `.context~executable`,
     `~source`, `.context~name`, `stackFrames[1]~executable`, a trapped 41.1 traceback and
     `~package`);
   - `own/p2` (`define` with a copy, then `.k~define('D')` mid-call);
   - `own/p3` (a private setMethod copy dropped, an OBJECT-scope copy, a `run` copy);
   - `own/p4` (a copy parked in GUARD WHEN on a started activity, its entry unset while parked, then
     woken);
   - `own/p5` (the `define` form of p4).

   The crate made 388-1544 collections per run. In `own3/r1-r3` the copy stays live mid-call on both
   sides (`swept 0`). Head holds it through the class `owned` list, the `ObjectMethod` trace or
   `.context~executable`'s own activation hold (`context.rs:459`). The rule holds on head. Its stated
   justification does not (I1).
3. **Internal guards.** I read each site: `environment.rs:1340`, `identities.rs:194`,
   `construct.rs:206`, `:237`, `dispatch.rs:1795`, `object_protocol.rs:398`, `:448`, `:467`, `:670`.
   - The `native_as_instance` callers (`construct.rs:180`, `:268`, `package.rs:1058`) pass only
     constructor answers.
   - `receiver_kind` answers `StackFrame` only for a native body of that class.
   - Probes `guards/g1` (StackFrame `objectName`, `objectName=`, `subclass`, `new`, `copy`) and
     `guards/g2` (subclasses of Method, Routine and Package built from strings, arrays and a
     `::routine` package) match the oracle and reach no guard.
   - The four freed-handle guards and the package guard are reachable only through a rooting defect.
   - No guard was reached.
4. **StackFrame and Deviation 28.** This is the right licence and not a representation leak. On the
   oracle (2 runs each), StackFrame natives `run` on a `::class t` instance give:
   - `NAME`, `EXECUTABLE`, `TYPE`, `TARGET`, and a setMethod-installed `EXECUTABLE`: SIGSEGV,
     rc 139;
   - `LINE`: a pointer (`140708140325504`, then `139666418346624`);
   - `CONTEXT`: `The NIL object`;
   - `ARGUMENTS`: an empty line.

   That is exactly D28's "accident of memory". The crate refuses all of them with `... not a stack
   frame`, as it does for the other primitives. Minor m3 covers the text.
5. **The three leaks**, `/usr/bin/time -f %M` peak kB through the committed `memloop.sh`, oracle
   against head `t8h`:

   | loop | oracle | head |
   |---|---|---|
   | `varref3` 20000/80000/320000 | 20080 / 20628 / 20312 (flat) | 44448 / 112476 / 385656 |
   | `define` 5000/20000/80000 | 20588 / 20888 / 20336 (flat) | 56948 / 170304 / 624112 |
   | `copy` | 12404 / 19532 / 20084 (flat) | 55060 / 55080 / 55600 (flat) |
   | `copypriv` | 11588 / 19836 / 20896 (flat) | 55832 / 59660 / 69860 |
   | `nativecopy` | 13128 / 19560 / 20588 (flat) | 54904 / 56440 / 72456 |
   | `methnew` (`Method~new`) | 20592 / 20396 / 20572 (flat) | 43728 / 45656 / 45668 (flat) |
   | `methnewpriv` (`Method~new` + `setPrivate`) | 20068 / 20600 / 20888 (flat) | 46108 / 48740 / 55224 |

   The oracle is flat on all three leaks. Fix sizes:
   - **`RootSet::cells`.** Make a cell a heap object (a `Body` variant) that `Target::Cell` names by
     handle. It is traced from `VarRef` bodies and from frame alias slots, which is state 1. The other
     option keeps the index vector, marks cells from those holders and free-lists the rest. That keeps
     a numeric-index table and needs a ruling.
     - Risk: high. Every `SlotRef` holder must be traced: VarRef bodies, aliases, a `USE ARG >`
       argument in flight, a REPLY continuation's frame. A missed holder means a reused cell, which
       gives a silent wrong variable, not a crash.
     - It changes rexx-core marking. varlookup and assign need measuring.
   - **Retired behaviours of a live class.** When any class has a retired behaviour, a cold pass
     after the sweep collects the `BehaviourHandle`s that live `Body::Instance` objects hold. It then
     empties each retired handle no instance holds and pushes it to `free_behaviours`.
     - Risk: medium. It rests on the invariant that the slot reuse already relies on (only
       `Body::Instance` holds a handle). The cost is gated to programs that `define`.
   - **MethodId vectors.** Option (a): keep `special_methods`/`natives` as vectors for the bootstrap
     range, and keep an `FxHashMap` beside each one for minted identities, pruned by
     `forget_method_identity`. This is a numeric-id map (`executable_objects` is the precedent), so it
     needs a ruling.
     - Cost: one compare on primitive dispatch. Measure dispatch and sendloop.
     - Risk: low.

     Option (b), MethodId reuse, is a new identity rule. Its risk is high: a stale id held by a
     retired behaviour or a frame would dispatch to a stranger. Prefer (a).

     The leak covers `Method~new` + `setPrivate` too, not only copies (Minor m4).
6. **Perf.** `callgrind.sh -B 0.5 -j 15 -p "startup churn compound alloc hashnew"` exited 0 with
   pin, `t8h` and my own HEAD rebuild:

   | program | t8h | rebuild |
   |---|---|---|
   | startup | +0.3665% | +0.3664% |
   | churn | +0.3919% | +0.3919% |
   | compound | +0.1065% | +0.1065% |
   | alloc | -0.7307% | -0.7307% |
   | hashnew | +0.1166% | +0.1166% |

   Base `t8base` against `t8h`: startup +34132 Ir (+0.0286%), churn -0.0205%. So most of startup's
   +0.3665% comes from earlier tasks. On the function-level self-cost diff, Task 8's startup share is
   all `ClassDef::retired_behaviours`:
   - `copy_instance_behaviour` self +11.5k;
   - `RawVec<BehaviourHandle>` `grow_one`/`finish_grow` about +10k;
   - the malloc family net of the memcpy and trim shifts.

   The library bootstrap calls `copy_instance_behaviour` 893 times, on classes that are never
   collected. Removing that share is Minor m2.
7. **Mutation.** One mutant, M4, in the archive copy: `object_protocol.rs:894` (`run`'s
   `cold_mut().executable = Some(executable)`) replaced by `let _ = executable;`. Probe
   `run/s1_run_inline.rex`, under collect-every-allocation:
   - oracle 3 of 3 and head: `exe Method 0 1 7`;
   - mutant: `exe Method 1 0 7`.

   It goes red. Without the hold, `run`'s newScope copy is swept mid-call. `forget_method_identity`
   drops its rows, and `.context~executable` then rebuilds an object with the template's flags. I
   restored the mutant by an exact edit and checked it with `diff` against `git show d81bacfad:...`.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

- **I1. The ownership rule's justification is false, and the hold it depends on has no witness.**
  - `lib.rs:1803-1806` (`ExecutableRecord::owns_installed`), the same sentence in the report, and
    `task-8/mutations.md` row M4 all say "an activation reads none of its identity's rows once it has
    entered".
  - The M4 mutant above disproves it. `.context~executable` resolves through
    `executable_objects`/identity rows after entry, and with the `run` hold removed the answer
    changes.
  - The committed tests stayed green under M4 only because `go`'s `use arg m` variable also held the
    object.
  - Head is correct only because of `object_protocol.rs:894`, and of `context.rs:459` once
    `.context~executable` has been asked. Task 8's pruning makes that hold load-bearing for the first
    time, and no committed test goes red without it. The global constraint says every load-bearing
    root needs a red-on-deletion test.
  - Fix:
    - delete the false clause, and name the `run` activation (and `.context~executable`'s) hold as
      the holder;
    - add an s1-shaped `collect_stress` test: `self~run(.k~method('RUNNER')~copy~~setPrivate)` with
      no variable holding the copy, a filler before `.context~executable~isPrivate`/`isGuarded`;
    - correct M4's row in `mutations.md`.

#### Minor (Nice to Have)

- **m1. INTERPRET cost.** `heap.rs:539-545` `release_immortal` scans the whole `immortal` vector with
  a SipHash `HashSet` on every INTERPRET whose chunk interned a heap literal.
  - On a 20000-iteration `interpret 'x = "a literal longer than seven"'` loop
    (`task-8-review-files/interp_loop.rex`), whole-process Ir goes from 3829901297 (base) to
    3932908255 (head), +2.69%. Of that, 95.5 M Ir is `release_immortal`, about 4.8 k Ir per
    INTERPRET.
  - No bench program covers it.
  - Fix: `FxHashSet`, or an immortal flag on the slot, so that release is O(released).
- **m2. Startup share.** `class_graph.rs:712` pushes a retired handle for each of the 893 bootstrap
  `define`s on library classes. The class is never collected, so the record is never read. Skipping it
  during the library build removes Task 8's +34132 Ir (0.0286%) of startup.
- **m3. Deviation 28's text.** `docs/superpowers/plans/phase-4-exclusions.txt:1768` lists the
  receivers D28 covers but not StackFrame, which Task 8 moved under it. Add the measured shapes
  (item 4).
- **m4. Leak 3's scope.** The report names copies only. `Method~new` + `setPrivate` grows the same way
  (`methnewpriv`, above).
- **m5. Pre-existing divergences found while probing** (identical on `t8base`, not Task 8's; queue
  them):
  - `.context~executable~isGuarded` inside an `enhanced` method whose Method was copied and
    `setUnguarded` answers `1`. The oracle answers `0` (`e2.rex`, `dir/d1`).
  - `raise syntax 40.1 additional("abc...")` then `condition('O')~additional[1]` answers `a`. The
    oracle answers the whole string (`add.rex`).

### Assessment

**Task quality:** Needs fixes

**Reasoning:** The sweep is complete and its fixes hold on head: every requested shape matches the
oracle under collect-every-allocation. But the stated soundness argument for copy-identity pruning is
false, and the `run` hold it actually depends on has no witness (I1).
