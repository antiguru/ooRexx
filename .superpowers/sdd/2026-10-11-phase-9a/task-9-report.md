# Phase 9a Task 9 report: `ReleaseLocalReference` and the `Package` hang

BASE `5d12347c0`. Code commit `7d7036e3a`. Status: Steps 1 and 2 done; Step 3 diagnosed, not fixed (its
premise was wrong and the cause is the deferred package-as-object work).

## Steps 1 and 2: the release

### Memory, `release_local_reference.rs`

`relloop.cpp` (the queued note's probe, verbatim) built by the test with `c++ -shared -fPIC -O1
-I<repo>/api -I<repo>/api/platform/unix` into `CARGO_TARGET_TMPDIR`; `say RelLoop(n)` run under
`memcap 2G timeout -k 5 120 /usr/bin/time -f %M -o <file> rexx-run`, so `time` reads the
interpreter's own rusage. The test asserts the 2,000,000-pass peak is at most twice the 200,000-pass
one.

| side | 200,000 | 2,000,000 |
|---|---|---|
| crate at `5d12347c0` (test red) | 59572 KB | 430344 KB |
| crate at `7d7036e3a` | 32168 KB | 32220 KB |
| oracle | 19980 KB | 20864 KB |

Crate rows from the test's own failure message (before) and from the same command by hand on the
built binary (after); oracle rows from `( ulimit -v 1048576; LD_LIBRARY_PATH=<oracle lib>:<probe dir>
timeout -k 5 20 /usr/bin/time -f '%M KB %e s' <oracle>/bin/rexx FILE )` in a fresh empty directory.

### What changed

* `handles::Table::release` moves the handle from the rooted entries to a `released` map that
  `roots()` does not report. A released handle resolves while the host says its object lives
  (`Table::resolve_live`; `Interp` checks `heap.peek` with the generation), and the map drops swept
  objects each time it passes twice its last size (floor 4096). `Table::remove` is gone.
* `Host::register_local` (default: register) and `Host::release_local` (default: release, all live).
  Every `cx.host.locals().register(...)` in `rexx-api` now goes through `register_local`.
  `Interp::register_local` also pops the top temporary when it is that object and lies above the
  native frame's `temps_floor` (taken at `push_native_frame`): the Host constructors push what they
  mint as a temp, and those temps were never popped inside a call, which held every string.
* `end_library_method` / `end_library_routine` push the call's answer as a temp before the frame
  pops, since the temp that used to root it can now be popped.
* `phase-4-exclusions.txt`: the M2 row is DELIVERED with the figures; the CLOSED row no longer says
  "rooted until the call ends", and says a use after a collection took the object is refused loudly.

Each part is load-bearing for the memory test, by mutation then exact restore (`git diff --stat`
checked after each): no temp pop, 429404 KB vs 57016 KB, red; release keeps the entry, 491352 vs
61408, red; released map never pruned, 135056 vs 36348, red. A release that clears every entry
reddens the two semantics tests. **Removing the answer's temp reddens nothing**: `collect_stress`,
the stress test, and a stress probe that concatenates two heap-sized native answers under `trace i`
all stay green. It restores the rooting the popped temp gave; no path needing it was found.

### Release semantics, oracle-probed

`relsem.cpp`, built against `api/`, run as `say RelTwice()`, `RelStash()`, `RelStale()`,
`RelOne(1000000)`, `RelLength()`, `'end'`. Oracle and crate (`7d7036e3a`), stdout byte-identical,
stderr empty, rc 0 on both:

```
released twice
kept
stale released
held among many
18
end
```

* Double release (`RelTwice`): no-op the second time on both. Oracle: `removeLocalReference`
  (`interpreter/execution/NativeActivation.cpp:1219`) on an object no longer in `saveList` removes
  nothing.
* Release of a handle this call never held, through a context kept from an earlier call
  (`RelStash`/`RelStale`): no-op on both. The oracle's stub (`ReleaseLocalReference`,
  `interpreter/api/ThreadContextStubs.cpp:145`) builds an `ApiContext` from the thread context, which
  is the activity's current native activation (`interpreter/api/ContextApi.hpp:64`-`:70`), so it removes from the
  running call's list, which does not hold it. Here `Table::release` ignores a handle `entries`
  does not hold, and does not add it to `released`.
* Release leaves the others: `RelOne` releases one string of two and then 1,000,000 more; the held
  one answers on both. Under `run_program_collect_every_alloc` (test
  `under_a_collection_at_every_allocation_only_the_released_string_goes`, `RelOne(50)`) it still
  answers, and the unit test `a_released_local_reference_is_collectable_inside_its_call` shows the
  released object swept and the held one kept by `collect_now`.
* Read after release (`RelLength`, 18): answers on both while no collection runs.
* A released handle used after a collection took its object: under the stress mode `RelTwice`'s
  string is collected at the stub's return and the crate ends rc 120 with `rexx-exec: the handle is
  no longer held by this activation is not implemented`. The oracle would read a freed object there.

## Step 3: the `Package` hang

**There is no hang at program end.** `TESTRECLAIM01`
(`ootest/ooRexx/base/class/Package.testGroup:155`) drops a package, wraps it in a `WeakReference`
and loops `until package~value == .nil` with no bound. Every package object here is a global root
(`Interp::package_object`, `environment/identities.rs`), so the loop never ends.

How it was found:

1. Harness, at `7d7036e3a`: `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/class/Package cargo test
   --release -p rexx-exec --test suite`: crate run 1 killed at 60 s with `TESTRECLAIM01` the last
   started test; run 2, it renamed out, ended normally with `Tests ran: 79`. Verdict red: not passed
   `TESTRECLAIM01 TEST_NEW01 TEST_PROLOG`, `TESTRECLAIM01=timeout after 60 s`, oracle 80 of 80.
   `StringTable~items` and `Directory~items` no longer refuse, so the scout's two renames are gone.
2. The whole group by hand in a copy, nothing renamed out, under `timeout 90` and then `timeout 400`:
   both printed a full summary (`Tests ran: 80`) with `Test execution` 00:01:29.97 and 00:06:39.97,
   that is, the bound; `TESTRECLAIM01` failed with `SYNTAX 4.1 ... Program interrupted with HALT
   condition`. The kill's `SIGTERM` halted the loop and the framework then finished. That is the
   "summary then no exit" the scout saw.
3. Standalone, from a fresh directory, one allocation round of 100 x `copies('x', 1000000)` per pass,
   at most 10 passes: for each of `.package~new(file)`, `.package~new('src', source)`, and
   `.package~new(file)` followed by a dropped `Routine` (one `::requires`-ing the file, one not), the
   oracle clears the weak reference after 1 pass and the crate clears none of them in 10.

The oracle caches a loaded package weakly (`PackageManager::addToRequiresCache`,
`interpreter/package/PackageManager.cpp:921`). Making a package object collectable here needs the
package to be a heap object that routines, methods, classes and activations reference, which is
`.superpowers/sdd/queued/2026-10-12-package-as-real-object.md` (Moritz 2026-10-12, deferred). Doing a
narrower version (dropping the global root) would clear a weak reference while a routine of that
package still lives, a new identity rule; so no fix and no crate test that pins today's retention.

Recorded where the next reader looks: the plan's Task 9 Step 3 carries a correction, the gate record's
line on `TESTRECLAIM01` is corrected, and the queued note carries the symptom.

**Group verdict**: before and after this task, red with `TESTRECLAIM01` blocked by `timeout after
60 s`, `TEST_NEW01` and `TEST_PROLOG` failing (the run above, after the release change, which does
not touch packages).

## Checks (each run in the foreground, statuses read unpiped)

`CARGO_TARGET_DIR=/tmp/claude-1000/p9/t9-target CARGO_INCREMENTAL=0 memcap 8G`, `-j 8` (load 1 to 9).

| command | status |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy -p rexx-core -p rexx-api -p rexx-exec --all-targets -- -D warnings` | 0 |
| `cargo test --release -p rexx-api -p rexx-core` | 0 |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus --test refusal_sites --test refusal_dispositions --test phase_ten_reasons --test closed_phases --test owners --test api_group_tests --test api_group_partition --test release_local_reference --test library_routine_memory --test native_entries` | 0 (corpus 29 passed, 1 ignored) |
| `cargo test --release -p rexx-exec --test collect_stress --lib` | 0 (lib 1087, collect_stress 37) |
| `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test release_local_reference --test api_group_tests --test corpus` (debug) | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/class/Package cargo test --release -p rexx-exec --test suite` | 101 (the group's red verdict above; the other 49 tests pass) |

No `Loud` constructor added or relabelled, so `refusal-sites.tsv` is unchanged. No new `unsafe`, no
new dependency. No `.rex` file added (the test programs are inline).

## Concerns

1. Step 3 is not fixed: it needs the deferred package-as-object design.
2. The answer temp in `end_library_*` has no witness that fails without it.
3. A use after release that meets a collection is a loud rc 120 here. M1's reading in the exclusions
   file is that the collection-timing licence does not cover an rc or stderr difference; the row now
   rests on "a use after release is outside the API contract", as the kept-handle row does. That is a
   ruling for the controller to confirm.
4. The crate still peaks 32 MB against the oracle's 20 MB for this loop, flat in the pass count.
