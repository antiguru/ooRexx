# Phase 9b1 final whole-branch review

Range `3e73e84b7..bdb4abf60`. Reviewer: opus, 2026-10-15. Read-only on the worktree; every probe ran
against a `git archive bdb4abf60` copy at `/tmp/fr9b1/head` (own `CARGO_TARGET_DIR`), a base copy at
`3e73e84b7`, and a mutant copy, each deleted afterwards.

**Verdict: ready to close with fixes.** One Important finding: a trace that is load-bearing with no test
that fails without it. Nothing found answers wrong on the tree as committed.

## Method

- Collect-every-allocation runs used a driver example that calls `run_program_collect_every_alloc` on a
  file (`crates/rexx-exec/examples/stress.rs`, in the archive copy only). Each shape ran three ways from
  a fresh empty directory: plain `rexx-run` under `memcap 4G`, the stress driver under `memcap 4G`, and
  the oracle under the standard `ulimit -v 1048576` wrapper. The filler was 300 passes of
  `copies('x', 100000)` (1000 or 3000 where a plain-mode collection was needed). No
  `oracle-crashes.txt` shape was run. Entry 49 (assign through a returned procedure's reference, then
  collect) was avoided.
- Memory: `/usr/bin/time -f %M` on the `rexx-run` binary itself, under `memcap 4G`.
- Mutants: exact edits in a third archive copy, restored by copying back from a saved file, with `cmp`
  against the head copy afterwards.

## Strengths

- The cross-task rooting held in every shape I built except the one under I1. All of these agree with
  the oracle on plain and stress runs:
  - a cell of a parked `GUARD WHEN` method in an otherwise unreferenced `Package~new` package, held by
    the main program (s1) or only by the parked frame's alias (s2);
  - a public routine and class merged from a package copy, held only by a REXX-package copy (s3);
  - INTERPRET literals reaching a condition object, `objectName`, a stem default, `.local`, a Message's
    arguments, an array, a `define` source, a table key and value, and a stem (s4);
  - an instance, method and UNINIT of a `Package~new` class after its package is dropped (s5);
  - a Routine object returned from an external program, used to reach that program's package (s7);
  - routines and classes reached through a `Package~new` that `::REQUIRES` a file, and `addRoutine` of
    a `Routine~new` copy onto a package copy (s8);
  - a self-undefining `Method~new` method, and a started Message on a `Package~new` class (s9);
  - retired behaviours with pre-`define` instances (s10);
  - requires chains whose prologues allocate (s12);
  - repeated failing installs (s13);
  - REPLY continuations of a package method and of a self-undefining method (s14, s15);
  - `addRoutine` of a routine whose package is dropped (s16);
  - classes held only by package tables (s17);
  - a running method whose behaviour is retired mid-run (s19);
  - a Method or Routine subclass instance over a dropped context package (s25);
  - stack frames and a condition's frames outliving their package (s24, apart from the queued
    frame-name items).
- Every phase corpus program agrees with the oracle on plain and stress runs.
- Growth: every loop I added is flat. Peak kB at 20000 / 80000 / 320000:

  | loop | 20000 | 80000 | 320000 |
  |---|---|---|---|
  | `Method~new` | 44032 | 44384 | 45008 |
  | `Routine~new` | 45240 | 44708 | 45416 |
  | `setMethod` source string | 46480 | 45656 | 45904 |
  | `Package~new` | 55168 | 54584 | 54316 |
  | Message `send` | 32284 | 32740 | 32280 |
  | package copy + `addRoutine` + REXX copy `addPackage` | 52044 | 52164 | |
  | `addRoutine` under one name | 44920 | 45744 | |
  | INTERPRET with a long literal | 26012 | 26436 | |
  | `Method`/`Routine` `~copy` | 55024 | 54908 | |
  | `.context~executable`, `instanceMethod`, `method`, `.context~package` | 34664 at 80000 | 34292 | 34428 at 1280000 |
  | SysStemSort, `.context~stackFrames`, `filespec` | 38248 | 37896 | 38088 |
  | method copy flags + `define` | 96920 | 96720 | 96824 |
  | `loadPackage` + `::REQUIRES` via `Package~new` | 67400 | 67224 | 66976 |
  | `addPackage` of one package, repeatedly | 54288 | 53144 | 53804 |
  | WeakReference to package, integer, object | 56552 | 55896 | 56088 |
  | `>name` and `USE ARG >` | 34132 | 35756 | 35692 |
  | `define`, `defineMethods`, `enhanced`, `Method`/`Routine~newFile` | 57924 at 5000 | 58044 | 57984 at 80000 |
  | external CALL of a file with a public class | 55116 | 55404 | |

  The executable-context loop's step from 24020 (20000) to 34288 (80000) is warm-up to the 32 MB floor:
  flat from 80000 to 1280000. The oracle's figure for the external-class loop is 24180 at 20000.
- No `unsafe` added (`grep` of added lines). `cargo test --release -p rexx-core --test unsafe_sites`:
  2 passed.
- `cargo test --release -p rexx-exec --test representation_refusals`: 3 passed. The gate's own grep
  over `crates/*/src` returns exactly the `ALLOWED` sites.
- Scout-leaks' ranked table: L1, L2, L3, L5, L6, L7, L8, L9, L10 and L11 are fixed, each re-run here
  against the oracle (the phase corpus programs, s20, s21, the memory table). L4 is licensed by
  Deviation 36: `objectName=` on `12345` refuses rc 120 naming the rule, and a subclass and an
  enhanced string accept it. L12 is licensed by Deviation 37 (`phase-4-exclusions.txt:2115`).
- The heavily mutated holders hold. Deleting the executable-package trace (`body.rs:445`, `:1359`)
  reddens 10 `collect_stress` tests. Deleting the class-table traces wholesale reddens the L0 subset
  with a panic.

## Issues

### Critical

None.

### Important

**I1. `PackageData::merged_public_classes`' trace is load-bearing, and no test fails without it.**
`crates/rexx-exec/src/package_state.rs:125`. An external CALL merges the callee's public classes into
the caller's `merged_public_classes` without importing the callee. Task 5's fix round 2 ruled that the
oracle does not import the callee (its `importedPackages` stays 0, and s18 shows `0` here too). For
routines, the merged entry holds the callee's package, which Task 5's fix round 2 and Task 6's ruling
added. For classes, nothing holds the class but this table's trace. Task 4's mutation table said the
classes traces "cannot go red here", and no later task witnessed the merged half once packages became
collectable.

The shape, `s18.rex`, beside `ext18.rex` (`return 'ext'`, then `::class kx public` with
`::method v` returning `'kxv' copies('x', 2)`):

```
call sub
call filler
say 'a' .context~package~findClass('KX')
say 'b' .context~package~importedClasses~hasIndex('KX') .context~package~importedPackages~items
c = .context~package~findClass('KX')
if c \== .nil then say 'c' c~new~v
exit
sub: procedure
  call ext18
  return
filler: procedure
  do i = 1 to 300; z = copies('x', 100000); end
  return
```

- Head: oracle, plain and stress all print `a The KX class` / `b 1 0` / `c kxv xx`, rc 0.
- Mutant: the line changed to `for table in [classes, public_classes]` (with `namespaces`' trace also
  dropped). The stress driver panics at `dispatch.rs:1741` "a live value", rc 101. A plain run with a
  1000-pass filler does the same.
- Green under the same mutant:
  - `collect_stress`, 71 passed;
  - `rexx-exec --lib`, 1134 passed, with 1 failure that head shares
    (`a_move_across_file_systems_copies_and_unlinks`, environmental);
  - `class_lifetime`, `class_weak_reference`, `package_requires`, `unreachable_classes`,
    `collect_policy`, `instance_copy_memory` and `library_routine_memory`.
- Fix: add s18 to `collect_stress`, and check that it is red under the mutant above.
- `namespaces`' trace was also unwitnessed under that mutant. It is redundant with `imports`, which
  `addPackage` writes with the same object (Task 4 review), so that is not a finding.

### Minor

**M1. The refusal gate cannot see `"a package whose program was freed"`**
(`environment/identities.rs:183`, added in `1f0b8382f`). It is the same internal guard as the allowed
`"a package whose object is no longer live"` beside it, but none of `PHRASES` matches its wording. Add
a phrase that catches it (for example "was freed") and an internal-guard row.

**M2. Silent defaults where a stale `ProgramId` misses.** These sites answer a default where
`package_object` refuses:

- `dispatch/executable.rs:430` `block_source_lines` and `dispatch/package.rs:480` `source_lines`
  answer an empty source;
- `environment.rs:1134` `build_package_string_table`'s `?` answers `None`, so `~addRoutine` skips the
  `.ROUTINES` write without a word;
- `dispatch/context.rs:955` `build_internal_frame_at` takes `.unwrap_or(0)`, which answers the
  innermost context when the caller activation is not found. This copies `:907`.

Each is unreachable while the holders are right, which is the point of a loud guard. Make them
refusals, or `debug_assert` them as `package_object` does.

**M3. Prose.**

- `rexx-bench/src/lib.rs:76`, `:79` and `:82` carry "Added in Phase 9b1 Task N's fix round", which is
  task history in a comment. It follows the file's earlier "Phase 9a final review" line.
- `package_state.rs:38` says "which all three `setSecurityManager` setters write", which names a set's
  size.
- No em-dashes were added. No added comment describes a mechanism that does not exist.

**M4. A ruling whose stated reason is false.** Ledger line 75 (Task 5) licenses "a package defining a
class is never collected" under D59, "recorded for the gate record". Task 7's note and Task 8's
closure of the class-collection item show that such packages and classes are collected. Re-run here:
s31's first two WeakReferences clear on both sides. The ruling is superseded but the line is not
amended. The gate record must not license L7 under D59.

**M5. The compound figure is path-sensitive (an instrument note).** `callgrind.sh -B 0.5 -p compound`
gives different whole-process deltas for the same source, depending on where the program files sit:

| run from | whole-process delta | libc-subtracted delta |
|---|---|---|
| the archive copy `/tmp/fr9b1/head/rust` (programs under it) | +0.6138% (OVER), 3 rounds, spread 0 | +0.0033% |
| the repository's `rust/` | +0.0009% | +0.0033% |

The repository's own `target/release/rexx-run` gives the same +0.6138% from the archive path. The
pin's own figure moves by 0.047% between the two paths. Run the close's callgrind from the
repository's `rust/`, or it will see a phantom OVER.

Subset from the archive path, whole-process against the pin:

| program | delta |
|---|---|
| startup | +0.3378% |
| churn | +0.2036% |
| pingguard | -0.0822% |
| sendloop | -0.1340% |
| hashnew | -0.3584% |
| interpret | -0.5483% |
| alloc | -1.0442% |
| intcall | -10.0610% |

## Declined to judge

- `.context~stackFrames` under `Routine~call` names the routine frame `CALL` and lacks the `CALL`
  method frame (s24, t2). This is on base too, and queued in `2026-10-01-routine-call-context-name`
  and `2026-10-14-native-method-frames`.
- UNINIT runs at program end, where the oracle runs it at the next activation return after a
  collection (s6). This is on base too and unrelated to 9b1.
- A WeakReference does not clear when its referent's handle last sat in a still-live register
  (s21/s22 plain, s31 `t`, s30). This is the Task 6 GC-timing ruling.
- The error header names the requiring program where the oracle names the required file (s28). This is
  on base too.
- A failed `::REQUIRES` install stays cached until a collection (s26, s27). The oracle does the same,
  measured both ways. Base held it forever, so this phase fixed it.
- Semicolons in added comments (4 lines): `rust/CLAUDE.md:132` says "There is no rule against
  semicolons in comments here", which contradicts the brief, so they are not filed.
- `Message~receiver` is not understood by the crate. Not checked against the oracle in isolation, and
  outside 9b1.

## Assessment

**Ready to close: with fixes.** I1 is a witness gap on the phase's own recurring defect class. The
tree answers right today, but deleting one trace line would ship a use-after-free past every gate.
M1 to M5 are small. Run M5's callgrind from the repository's `rust/` at the close.
