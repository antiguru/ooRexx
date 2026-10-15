# Phase 9b1 Task 6 review: program lifetime

Reviewed 8608d4afe..13c5c2390, with the spec amendment at 595139f64. Built HEAD (595139f64) from a
`git archive` copy under `/tmp/t6rev` with its own `CARGO_TARGET_DIR`. That copy had one scratch test
added, `tests/zz_probe.rs`, which runs a file through `run_program_collect_every_alloc`. The copy is
now deleted. Probes, memory programs and the probe test are kept in
`/home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-6-review/probes/`, and the callgrind outputs in
`.../task-6-review/`.

## Spec compliance

❌ Issues found:

- **Step 3 / Review Focus 4: the parked-activity holder is not pinned.** Probe `parked.rex`: a
  source-string `setMethod` method unsets its own entry, then parks in `guard on when flag = 1` on a
  started activity. Main runs a 300-pass `copies('x', 100000)` filler and then releases it. Oracle,
  plain crate and collect-every-allocation all agree, rc 0: `hasW 0` / `parked 6 guard on when flag = 1`.
  Mutant: `scheduler.rs:415` `Activities::running_programs` made empty, so idle activities hand the
  collector no programs. The probe then fails under stress with rc 120, `a message send to a package
  whose program was freed`. The release `--lib` run (1122 passed, with the same single environmental
  failure, `a_move_across_file_systems_copies_and_unlinks`, as the unmutated baseline) and
  `collect_stress` (54 passed) both stay green. So this shape is a real case where the
  non-running-activity walk is the only holder. Task 5's ruling recorded that no such shape was found
  and left the walk unwitnessed. Task 6 makes it the only thing keeping a program loaded, and no
  committed test goes red without it. Fix: add `parked.rex` as a `collect_stress` test. The
  `run_program_collect_every_alloc` harness runs `~start` and `SysSleep` here.
- ⚠️ Corpus, suite and the other test binaries were not run under that mutant.

Verified and compliant:

- **Step 1, no id reuse.** `ProgramId(u64)` has a private field, and `ProgramId::new` is private to
  `plan.rs`. `nth` is `cfg(test)` only. `grep -rnE 'ProgramId\('` outside `plan.rs`, `programs\[`
  and `\.0` on a program id all come back empty in `src/`. `Programs::get`
  (`plan.rs` `get`) compares the whole stored `u64` (slot plus generation) against the requested id,
  and then reads `Option<Rc<Program>>`. A freed slot stores the next, unissued generation, and a slot
  whose next generation would be `u32::MAX` is retired. `g + 1` cannot overflow because issued
  generations stop at `u32::MAX - 1`. So a stale id fails the compare, and a forged id cannot be
  built outside `plan.rs`.
- Every resolution path in the diff goes through `get`: `begin_method`, `delegate_variable`,
  `accessor_variable`, the guarded check (`dispatch.rs`), `Entered::Routine` (`call.rs`),
  `call_program_main` (lib.rs), `executable_at`/`routine_entry_name`/`trace_line_text`
  (`context.rs`), `declared_flag`/`is_rexx_code`/`block_source_lines`, `source_lines`, `body_kind`,
  `constant_name`, and `package_slot`. `plans` and `chunks` key on `BodyKey`, which carries the full
  id, as do `compiled_method_names` and `package_local_miss`. So a stale key cannot match a later
  program even before pruning.
- M14 (drop the compare in `get`) re-run: `a_freed_programs_id_never_resolves_again` goes red with
  `2.0 resolves after its free`.
- **Step 4 under the amended criterion: met.** `memcap 4G /usr/bin/time -f %M <binary> X.rex`, run
  from fresh directories:

  | loop | crate 20000 / 80000 kB | 80000 vs 20000 | oracle 20000 / 80000 kB |
  |---|---|---|---|
  | `Method~new` | 175692 / 175540 | -0.09% | 20228 / 20336 |
  | `Routine~new` | 175160 / 174996 | -0.09% | 20368 / 20292 |
  | source-string `setMethod` | 163508 / 163320 | -0.11% | 20324 / 20876 |
  | `Package~new` | 420480 / 421380 | +0.21% | 20592 / 20640 |

  At 320000 iterations: `Method~new` 177160 kB, `Package~new` 421588 kB, source-string `setMethod`
  163824 kB, so the curve is flat well past 80000. Oracle runs used
  `( ulimit -v 1048576 -t 120; LD_LIBRARY_PATH=... /usr/bin/time -f %M .../build/bin/rexx X.rex )`.
- **Step 5 perf: reproduced exactly.**
  `bench-programs/callgrind.sh -B 0.5 -j 27 -p "sendloop dispatch dispatchclass startup fibcall intcall" pin=... review=<my HEAD build> t6d=...`
  exited 0 with no OVER and no SPREAD. My build: sendloop +0.4486%, dispatch +0.0924%,
  dispatchclass -2.8053%, startup +0.3388%, fibcall -3.4290%, intcall -10.0601%. `t6d` matches to
  the instruction on sendloop and dispatch.

## Strengths

- The id design is tight. One compare against a stored full id, a private constructor, retirement
  before wrap, and `BodyKey`s carrying the full id. Even an unpruned row cannot name a later program.
- `e186ebe0c` caught that deriving the freed set from `get(id).is_none()` made the test pass under a
  lookup that ignores the generation, and changed it to read the table's own `freed` log. M14
  confirms the fix.
- The perf work was diagnosed rather than guessed: making `missing_body` cold, packing the id into
  one `u64`, and moving the package fields out of the hot slot.
- Every probe I ran is oracle-identical on all three sides, plain and under collect-every-allocation:
  a held `stackFrames` (`frames.rex`), INTERPRET inside a `Method~new` method (its frames held after
  the method drops, and an INTERPRET running in a method that has unset itself, `interp.rex`), a
  Method or Routine object as the only holder (`objects.rex`), plan-cache reuse across freed and
  reissued slots with differently shaped bodies (`plans.rex`: `2 3 4` ... `10 15 20`), a
  `Method~new` method still running after its last reference drops, and a Routine that removes its
  own `.local` entry (`running.rex`), the parked activity above, and package copies outliving their
  original (`pcopy.rex`, `pcopy2.rex`). One exception, already queued: the oracle's native
  `Compiled method "RUN"` frame is missing from `interp.rex`'s frame list (TESTNATIVECODEFRAME).

## Issues

### Critical

None.

### Important

1. **The parked-activity frame is unpinned** (Spec compliance, first item). Add the test.
2. **An external CALL loop keeps every callee program for the caller's lifetime.** Not in this
   diff, but it is the reason section 3's freeing never reaches these programs. `install.rs:847`
   `merge_package` pushes `from` into `into`'s `merged_from` on every merge, even when nothing was
   merged (Task 5 fix round 2). `enter_external_program` (`lib.rs`) registers a fresh program and
   package on every call, so each call's package, and with it its program, is traced from the
   caller's package for good. I found that by reading the code. The growth itself is measured:
   `do 20000 / 80000: call ext` with `ext.rex` = `return 1` peaks at 640528 / 2502824 kB here,
   against 20604 / 20412 kB on the oracle. Task 5's `t5c` peaks at 638400 kB at 20000, so the leak
   predates Task 6. That is about 31 kB per call, unbounded. It is a "grows with the iteration count"
   case that is not one of section 6's named loops. Fix: push only when the merge copied something
   that names `from`. The controller decides which task this goes to.

### Minor

1. **Report table, Step 2: `special_methods` is missing** (`lib.rs:1591`,
   `Vec<Option<AccessScope>>` indexed by MethodId, with `AccessScope::package: Package::Program(id)`).
   It is not pruned. Its rows cannot misapply, because MethodIds are minted monotonically
   (`rexx-classes/src/registry.rs:66`) and ProgramIds are never reused. But a PRIVATE, PACKAGE or
   PROTECTED method of a freed program leaves its row behind, and the Vec only grows. Every other
   ProgramId-bearing Interp field I found is in the table, or is transient and rooted:
   `activity.failure_origin` and the condition `origin` through `hold_origin_package`, and `Caller`.
2. **M2 is red for a reason other than lifetime.** Re-run: `a_held_stack_frame_keeps_its_program`
   fails with rc 120, `a message send to a method object this crate did not build`. That refusal
   fires whether or not anything was collected. It shows the test depends on
   `set_executable_package`. It does not show that a Method object keeps its program across a
   collection.
3. **Several prunes have no witness.** The unit test asserts `plans`, `chunks`,
   `compiled_method_names`, `required_paths`, `untranslated`, `method_bodies`, `routine_objects`,
   `executable_sources` and `annotations`. It does not assert `generated_methods`,
   `external_packages`, `native_externals`, `library_externals`, `guarded_externals`,
   `library_routine_codes`, `rexx_routine_rows` or `required_packages`, and the mutation table has no
   row for any of them. Leaving any of them unpruned would leak but could not misresolve.
4. **New panic path.** `install.rs` `unattached_method_id` has
   `.expect("the package whose table names it holds the program")`. The crate's rule elsewhere in
   this diff is a loud refusal, not a panic.
5. **sendloop cost (question 5).** From the `task-6-try3` profiles, `t5c` to `t6d` adds 130.0 M Ir
   over 5 M sends, about 26 per send: begin_invoke +24 and drive_from +2. Line attribution inside
   `begin_invoke`:
   - `plan.rs` (the `get` compare): +4 per send. This is the inherent cost.
   - `activation.rs:859` (building `Activation { .. }` in `Activation::method`): +10 per send net,
     new at +13.
   - `dispatch.rs:3282` (`push_activation(callee)`): +8 per send.
   - `index.rs`: +3 per send.
   - `option.rs`, `non_null.rs`, `raw.rs` and `mod.rs`: -5 between them.

   `Activation` gained no field and `ProgramId` is still 8 bytes. So about 20 of the 26 are codegen
   around building and moving the activation struct, not the lookup itself. In principle that is
   removable. I did not try to remove it. Everything stays within +0.5%.

## Assessment

**Task quality:** Needs fixes

**Reasoning:** Id issuance and lookup are sound: a stale or forged id cannot resolve, and every probed
holder agrees with the oracle under collect-every-allocation. Memory meets the amended criterion and
perf reproduces within budget. But the parked-activity frame, which my mutant shows is the only
holder in a real program, has no test. And an external CALL loop still leaks a program per call
through `merged_from`.
