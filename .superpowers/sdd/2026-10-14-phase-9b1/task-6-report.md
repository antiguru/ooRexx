# Phase 9b1 Task 6 report: program lifetime

Status: DONE_WITH_CONCERNS. Step 4's 2x memory bar is not met (see Step 4). Commits:
`1f0b8382f` (the change), `40a65669f` (lookup cost), `e186ebe0c` (test fix), `c27f3b938`
(refusal-sites.tsv). Records commit: see the reply.

## What changed

**The table (Step 1).** `Interp::programs` is `plan::Programs`. A `ProgramId` is one `u64`: slot in
the low half, generation in the high half. The hot table holds each slot's current id beside its
program, so `Programs::get` is a bounds check, one compare and one load. A freed slot holds the next
generation's id, not yet issued, and goes on a free list. A slot whose next generation would be
`u32::MAX` is retired. `program_packages` is gone: each slot's package object and holds live in a
second table, `Programs::packages`.

**What holds a program (Step 2).** A package object holds its program: `PackageData::hold` is a
`ProgramHold` (an `Rc`), set by `package_object` when it builds the package, and cloned by every
package copy. A `Method` or `Routine` object holds its package (Task 5), a class holds its package
(Task 5), and a running frame's package is rooted by `package_roots` (Task 5). `collect_now` calls
`Programs::free_unheld` after the sweep: a program whose package was built and whose holds are all
dropped is freed. A program no package was ever built for is never freed (the main program before
anything asks for its package, for one).

`package_object` refuses a freed program's id (`Loud::unknown_receiver("a package whose program was
freed")`, with a `debug_assert` beside it, the same shape as Task 5's collected-package guard).
Every `self.programs[...]` index became a `get`: the send and call paths refuse with
`Loud::missing_body()`, now `#[cold] #[inline(never)]`.

**A compiled method's annotation table** (`Annotated::Compiled`) is held by its object alone: no
global root and no `annotations` row. Each was a fresh global root per `Method~new`/`Routine~new`
that nothing ever asked for by its site again.

`RootSet::remove_global` is new in `rexx-core` (for the annotation roots below).

## Step 2: each table and its pruning site

Enumerated from `git grep -n 'ProgramId' rust/crates/rexx-exec/src/lib.rs` (field declarations),
plus the ProgramId-valued maps that grep shows through `InstalledRoutine`, `BodyKey`,
`InstalledMethodBody`, `GeneratedMethod`, `ExecutableRecord` and `Annotated`. Pruning is
`Interp::prune_program_rows` (`lib.rs`), called from `collect_now` when `free_unheld` freed
anything.

| table | key / value | pruned where |
|---|---|---|
| `programs` | the table | `Programs::free_unheld` |
| `program_packages` | removed | the slot's `PackageSlot` is reset in `free_unheld` |
| `plans` | `BodyKey` | `prune_program_rows` |
| `last_plan` | `BodyKey` | `prune_program_rows` (cleared) |
| `chunks` | `(BodyKey, ChunkTrace)` | `prune_program_rows` |
| `compiled_method_names` | `ProgramId` | `prune_program_rows` |
| `required_paths` | `ProgramId` | `prune_program_rows` |
| `untranslated` | `ProgramId` | `prune_program_rows` |
| `required_packages` | value `ProgramId` | `prune_program_rows`, and `collect_now`'s dead-package retain (Task 5) |
| `method_bodies` | value `InstalledMethodBody` | `prune_program_rows`, collecting the MethodIds |
| `generated_methods` | value `GeneratedMethod` | same |
| `external_packages` | value `ProgramId` | same |
| `executable_objects`, `native_externals`, `library_externals`, `guarded_externals` | the MethodIds above | `prune_program_rows` |
| `routine_objects` | `InstalledRoutine` | `prune_program_rows` |
| `library_routine_codes` | `InstalledRoutine` | `prune_program_rows` |
| `rexx_routine_rows` | `InstalledRoutine` | `prune_program_rows` |
| `executable_sources` | value `ExecutableRecord` naming the program | `prune_program_rows` (`ExecutableRecord::names_program`) |
| `annotations` | `Annotated::Package(Program)`, `Unattached`, `Routine` | `prune_program_rows`, and each one's global root via `unroot_annotation_table` |
| `library_programs` | `Vec<ProgramId>` | not pruned: `package_roots` roots these packages, so they are never freed |
| `library_codes` | `Vec<Option<ProgramId>>` | not pruned: `package_roots` roots these packages (Task 5, the rxmath measurement) |
| `option_writes` | not keyed by program (`OptionWrites { defaults, count }`) | nothing to prune |
| `class_packages` | value `ClassPackage::Program` | not pruned: a class holds its package, and classes are never collected (D59) |
| `package_local_miss` | a one-entry cache naming a program | not pruned: ids are never reused, so a stale entry cannot match |

## Step 3 and Review Focus 4

`rust/crates/rexx-exec/tests/collect_stress.rs`:

- `a_held_stack_frame_keeps_its_program`: three `Method~new` run through `self~run`, each answering
  `.context~stackFrames`, every method dropped, only the frames array held. Reads the frame's
  `name`, `line`, `traceline`, its executable's `source` and `package~sourceline`, and runs the
  executable again. Oracle rc 0 with a 300-pass `copies('x', 100000)` filler:
  `frame *UNNAMED* 2      2 *-* return .context~stackFrames` / `executable Method 2 x =  3 x =  3` /
  `runs      2 *-* return .context~stackFrames`. The exact test text was also run on the oracle.
- `a_running_frame_keeps_its_program`: a `Method~new` passed straight to `run`, a `Routine~new` that
  removes its only `.local` reference, and a source-string `setMethod` method that unsets its own
  entry first, so its frame alone reaches it. Each fills, calls an internal label and reads its own
  source and package. Oracle, exact test text, rc 0: `ran 6 do i = 1 to 3` /
  `routine 5 x = .local~remove("R")` / `unset 7 do i = 1 to 3 0`.

Step 1's test, `run::tests::a_freed_programs_id_never_resolves_again` (`src/run/tests.rs`): four
passes of `Method~new`, `Routine~new` and `Package~new` with a `::routine`, each run, under
`enable_stress_collect`. It reads the `cfg(test)` logs `Programs::issued` and `Programs::freed`
and asserts: no id issued twice; some freed slot was reused by a later id (so a stale id is put to
the test); every freed id was issued, resolves to no program and no package; no `plans`, `chunks`,
`compiled_method_names`, `required_paths`, `untranslated`, `method_bodies`, `routine_objects`,
`executable_sources` or `annotations` row names it; and no `Annotated::Compiled` row exists.

A first version derived the freed set from `get(id).is_none()`, which made a lookup that ignores the
generation pass (M14 below stayed green). `e186ebe0c` takes the freed set from the table's own log.

### Other holders probed (not committed as tests)

From a fresh directory, release head against the oracle, a 400-pass `copies('x', 100000)` filler:
`setMethod` with a dropped `Method` object, `.k~define` with one, source-string `setMethod`,
`enhanced` over a dropped directory of methods, a routine from a dropped `Package~new`, a class from
a dropped `Package~new`, a `Message` over a `Routine~new`. All seven lines agree (`a ran` ...
`msg ran`), and a debug build of `1f0b8382f`'s source prints the same with no assertion.

### TDD evidence

The tests were written after the implementation, so there is no RED run against the base for them.
At Task 5's base the two `collect_stress` tests pass (programs were never freed) and the unit test
does not compile (no `Programs`). What shows each is load-bearing is the mutation table below.

### Mutations

Each applied by an exact edit from a saved copy and restored by copy; `git diff` compared against a
saved patch after each batch, identical every time. Harness: scratch `t6/mut/mutate.py`. Unit =
`cargo test -p rexx-exec --lib -- a_freed_programs_id_never_resolves_again` (debug), frames =
`cargo test -p rexx-exec --test collect_stress -- keeps_its_program` (debug unless noted).

| id | mutation | result |
|---|---|---|
| M1 | `Activity::running_programs` drops every activation's program | frames red (debug: `package_object`'s assertion; release: `a message send to a package whose program was freed`, the `unset` line missing) |
| M2 | `record_executable` stores no package in a `Method` object | frames red, `a_held_stack_frame_keeps_its_program`, rc 120 |
| M3 | `free_unheld` never frees | unit red, "no freed slot was reused" |
| M4 | a freed slot keeps its id (no generation bump) | unit red, "issued twice" |
| M14 | `Programs::get` skips the id compare | unit red, "resolves after its free" (after `e186ebe0c`; green before it) |
| M5 | no `plans` prune | unit red |
| M6 | no `chunks` prune | unit red |
| M7 | no `executable_sources` prune | unit red |
| M8 | no `method_bodies` prune | unit red |
| M9 | no `routine_objects` prune | unit red |
| M10 | no `compiled_method_names` prune | unit red |
| M11 | no `annotations` prune | unit red |
| M12 | a `Compiled` site keeps its row and root | unit red |
| M13 | `prune_program_rows` never called | unit red |

M1 first stayed green against a version of `a_running_frame_keeps_its_program` without the
`unsetMethod` line: the method and routine objects were reachable through `run`'s argument and the
send's receiver. The `unset` shape was added for that reason. M3 to M14 were re-run against the
final test.

## Step 4: memory (NOT met)

`docs/superpowers/records/2026-10-14-phase-9b1/task-6/memory.txt` holds the commands, programs and
output.

| loop | oracle 20000 / 80000 | head (`t6d`) 20000 / 80000 | Task 5 (`t5c`) 20000 / 80000 |
|---|---|---|---|
| `m = .Method~new('m', 'return 1')` | 20888 / 20748 kB | 175540 / 175688 kB | 450332 / 1728540 kB |
| `r = .Routine~new('r', 'return 1')` | 19984 / 20692 kB | 174776 / 175120 kB | 447536 / 1722964 kB |
| `self~setMethod('FOO', 'return 1')` | 20324 / 20324 kB | 162844 / 163868 kB | 441144 / 1737580 kB |
| `p = .Package~new('p', 'return 1')` | 20084 / 20488 kB | 420604 / 421156 kB | 593488 / 2240644 kB |

Programs are freed: 80000 peaks where 20000 does, for all four. The peak is still 8x to 20x the
oracle's, for two reasons outside this task's files:

1. **The collector's trigger does not see a program.** Measured on 1000 to 10000 `Method~new`
   (scratch `t6/mem`): about 23 kB of RSS per `Method~new` until the first collection, which comes
   between 5000 (138852 kB) and 10000 (175092 kB). That is Task 7 Step 2 (the allocation charge
   counts a package and a program).
2. **Even charged, the 32 MB byte floor (`COLLECT_BYTES_FLOOR`) is above the bar.** An experiment
   binary charging 20 kB at each program package (`charge_unheld(20_000)` in `package_object`,
   not committed) peaks at 54532 / 53796 / 54624 / 67028 kB for the four 80000 loops: 2.6x to 3.3x.
   Under that floor, about 32 MB of programs can be garbage before a collection, so the oracle's
   20 MB times 2 is out of reach unless the floor or the per-program size moves.

Per-program size, from massif on 2000 `Method~new` (heap peak 48.5 MB against 12.9 MB at startup,
so about 17.8 kB of heap each): `rexx_parse::SymbolTable::intern` accounts for about 6.4 kB per
program, because every parsed program interns the whole keyword set (`token::keywords`). That is in
`rexx-parse`.

## Step 5: budget

Full bench set, from `rust/`, against the pin:

```
timeout 590 bench-programs/callgrind.sh -B 0.5 -j 16 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-6-b pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run-9b1-t6d
```

Exit 0, 3 rounds, no OVER, no SPREAD. `t6d` was built at `40a65669f`, and its `.text` is identical
to a release build at `c27f3b938` (the later commits are test code and the tsv). Records:
`task-6/callgrind/`. Whole-process deltas of the send, call and package paths:

| program | whole d% | -libc d% |
|---|---|---|
| startup | +0.3393 | +0.4829 |
| sendloop | +0.4486 | +0.4724 |
| dispatch | +0.0924 | +0.2916 |
| dispatchclass | -2.8053 | -1.6335 |
| fibcall | -3.4290 | -2.2703 |
| fibfunc | -3.4395 | -2.2753 |
| extcall | -13.1971 | -1.7078 |
| intcall | -10.0600 | -9.0140 |
| pkglocal | -5.6283 | -7.7524 |
| varlookup | +0.0032 | +0.0021 |
| alloc | -0.4156 | -0.8314 |
| hashnew | +0.1393 | +0.1224 |

The first full run (`t6a`, at `1f0b8382f`, `task-6/callgrind-first/`) was OVER: dispatch +1.2912%,
sendloop +2.2313%. Against `t5c` both cost exactly 390 M more, 78 instructions per send. The
function diff showed `invocation_name`, `package_start` and a `Vec` push leaving `begin_invoke`'s
inlining, and the instruction-level diff showed the slot lookup (40-byte slot stride, a shift for
the generation) plus spills. `40a65669f` makes `missing_body` cold (inlining restored, sendloop
+1.41% vs `t5c`), packs the id into one `u64` (+1.14%), and moves the package fields out of the hot
slot so a lookup is one compare (+0.90%). What is left is about 26 instructions per send in
`begin_invoke`, most of them spills (`t6/asm/` in the scratchpad).

startup moved from Task 5's +0.3156% to +0.3393%, leaving 0.16% of headroom. sendloop is at
+0.4486%.

## Representation refusals

The one row tagged Task 6, `dispatch/context.rs:a program this crate did not load`, is removed with
its site: `routine_entry_name` refuses a freed program with `Loud::missing_body()`. The `"Task 6"`
owner is removed from `OWNERS`. The new refusal text in `package_object` contains none of the
phrases.

## Checks (statuses read unpiped; outputs in scratch `t6/checks/`)

All from `rust/`, `CARGO_INCREMENTAL=0`, `-j 4`, `timeout 590 memcap 8G`. fmt, clippy, the lib
tests, both `collect_stress` runs, `sim::tests` and the release corpus ran at `e186ebe0c`; the
tables, `rexx-core` and the debug corpus at `c27f3b938`, which changed only the tsv.

| command | exit | result |
|---|---|---|
| `cargo fmt --all --check` | 0 | |
| `cargo clippy -p rexx-core -p rexx-exec --all-targets -- -D warnings` | 0 | |
| `cargo test -p rexx-exec --lib` (debug) | 0 | 1123 passed |
| `cargo test -p rexx-exec --test collect_stress` (debug) | 0 | 54 passed, 178 s |
| `cargo test --release -p rexx-exec --test collect_stress` | 0 | 54 passed |
| `cargo test --release -p rexx-exec --lib -- sim::tests` | 0 | 24 passed |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | 0 | 29 passed, 1 ignored |
| `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` (debug) | 0 | 29 passed, 1 ignored |
| `cargo test --release -p rexx-exec --test representation_refusals --test refusal_sites --test refusal_dispositions --test owners --test closed_phases --test phase_ten_reasons --test package_requires --test class_lifetime --test class_weak_reference` | 0 | 9 binaries ok. The first run at `e186ebe0c` was 101: `refusal_sites` disagreed on line numbers after `missing_body`'s attributes, re-derived in `c27f3b938` |
| `cargo test --release -p rexx-core` | 0 | 108 passed, incl. `unsafe_sites` |
| suite, Task 5's groups (command below) | 101 | 9 groups, 5 red, as at Task 5; `tests.tsv` identical to Task 5's; TESTRECLAIM01 passed |

Suite command (records in `task-6/suite/`, run at `40a65669f`, whose source differs from the final
only in test code and the tsv):

```
REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/class/Package,base/class/Package_Options,base/directives/REQUIRES,API/oo/METHOD,base/directives/METHOD,base/directives/ROUTINE,base/class/Method,base/class/MethodArgs,base/class/Routine REXX_SUITE_TABLE=<records>/task-6/suite/table.tsv CARGO_INCREMENTAL=0 timeout 590 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite -- --nocapture
```

`rexx-parse` was not touched; its tests were not run.

## Concerns

1. Step 4's 2x bar is not met, and cannot be by this task alone: it needs Task 7 Step 2's program
   charge, and the experiment above says the 32 MB byte floor then still leaves 2.6x to 3.3x. The
   controller should decide whether Task 7 also moves the floor or the bar.
2. `40a65669f` changed `missing_body` and shifted `refusal-sites.tsv` line numbers without
   re-deriving it in the same commit. `c27f3b938` re-derives it. The constraint says the same
   commit, and that was missed.
3. sendloop is at +0.4486% and startup at +0.3393% against the pin: little headroom for Task 7.
4. Observed, not caused here, present at `t5c` too: `w = .WeakReference~new(.Routine~new('t',
   'return 1'))` followed by `call filler` (an allocating loop in an internal routine) leaves `w`
   set where the oracle clears it. The same with the loop inline clears on both sides. Something
   from the first clause stays reachable while the call runs. Scratch `t6/p3/h1.rex`.
5. Tests were written after the implementation (see TDD evidence); the mutation table is the
   evidence that each one can fail.
