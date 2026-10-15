# Phase 9b1 Task 7 review (4d65804cd..5958a6f63)

Reviewer files (probes, witness tests, stress driver, callgrind table, measuring scripts):
`.superpowers/sdd/2026-10-14-phase-9b1/task-7-review-files/`. Raw callgrind outputs:
`/home/moritz/dev/repos/ooRexx-p9-pin/cg-9b1/task-7-review/`. Binaries were built from a `git archive HEAD`
copy under `/tmp/t7rev` (own `CARGO_TARGET_DIR`, `Compiling rexx-exec` seen on every build). The copy is
deleted. HEAD's crate code equals `f9341ef33`'s (`git diff --stat f9341ef33..HEAD -- rust/crates` is empty).

## Spec Compliance

- ❌ One map in the Method/Routine group is neither a field nor pruned, and it leaks measurably.
  `defined_library_codes` and `library_externals` rows minted by `bind_loaded_method`
  (`environment/identities.rs:627`-`:645`) are keyed by a fresh MethodId for every `loadExternalMethod`
  answer. They are pruned only with a program, and a loaded method has none. Loop
  `m = .Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Init')` (`extmeth*.rex`, with
  `LD_LIBRARY_PATH` set to the oracle's lib):

  | iterations | HEAD | oracle |
  |---|---|---|
  | 20000 | 30768 kB | 19836 kB |
  | 80000 | 54224 kB | 20636 kB |
  | 320000 | 80444 kB | 20076 kB |

  That is about 110 B per call, still growing at 320000. The report's concern 1 names this map ("the same
  shape for `loadExternalMethod` answers") but does not measure it. The progress.md ruling moves only
  `Method~copy` identity rows to Task 8. This map needs its own ruling, most likely the same move.
  `loadExternalRoutine` is flat (19292 / 19200 kB at 80000 / 320000).
- ✅ The rest of item 1. I checked each name in scout-leaks' Method/Routine and Message groups against the
  tree. `prune_dead_object_rows` (`lib.rs:3255`) covers `executable_sources`, `method_flag_writes`,
  `executable_objects` (pruned by value), `message_outcomes`, `started_messages` and `unnotified_messages`.
  `prune_messages` (`scheduler.rs:345`) covers `message_ids` and `senders`. The `freed_classes` block covers
  `Annotated::Class`/`Member`. `prune_program_rows` covers `routine_objects`, `library_routine_codes`,
  `rexx_routine_rows`, `method_bodies`, `generated_methods`, `native_externals`, `guarded_externals`,
  `library_externals` and `special_methods`. I enumerated that last list with
  `awk '/fn prune_program_rows/,/^    }$/' lib.rs | grep -oE "self\.[a-z_]+"`.
  `library_routine_objects` and `internal_routine_objects` are bounded roots.
  `rexx_routine_objects` and `rexx_routine_executables` do not exist (grep). Method~copy identity rows
  are ruled to Task 8.
- ✅ Step 2: the charge sites are as claimed.
  - Every `programs.insert` outside a test module goes through `load_program`, except the library's at
    `lib.rs:2416`. I checked `queue.rs:139`, `trace.rs:860` and `stem.rs:781`: each is inside `mod tests`.
  - `package_object` charges `PACKAGE_BYTES` once per program package build (`identities.rs:228`).
  - `install_class` charges outside the bootstrap (`install.rs:1264`-`:1267`).
- ✅ Step 3: the full set was re-run (below), `ALLOWED` lost the five "Task 7" rows, and `OWNERS` lost
  "Task 7".
- ⚠️ The section 6 memory criterion holds on HEAD (below). The queued item is closed only in the report
  text. The queued file is untouched by the diff, so the controller should mark it closed as far as it is.

## Checks run

### 2. Prune safety

Each probe was run from its own fresh `mktemp -d`, on three interpreters:
- the oracle (`ulimit -v 1048576`);
- HEAD's `rexx-run` under `memcap 4G`;
- HEAD under collect-on-every-allocation. This used a driver calling
  `run_program_collect_every_alloc` (`stress-driver.rs.txt`), built in the copy only.

Each probe's filler was `copies('x',100000)`. The script is `run3.sh`.

| probe | shape | oracle | HEAD plain | HEAD stress |
|---|---|---|---|---|
| `m1_result_after_drop` | a result read after its Message is dropped; send and start; 20 fresh messages in recycled slots checked for a stale `completed`; a raising started message per iteration | rc 0, `a 2 2` / `b 2` / `c done` | identical stdout and rc | identical stdout and rc (collections > 0) |
| `m2_started_dropped` | started message dropped while it runs; a new Message right after; two more started | rc 0, `b completed before send: 0`, `c 7`, `d 8` | identical values and rc | identical, 91 collections |
| `m3_method_dropped` | `self~run(m)` where the method removes its only holder, then a filler and `.context~executable` flags and source. A Method with `setUnguarded`/`setPrivate` passed to `define`, then dropped: its flags and the private send are read back | `exe Method 0 0 ...`, `run 3`, `defined 0 1`, `private send raised 97` | identical | identical, 148 collections |
| `m4_routine_dropped` | `~call`, `r~call` and `~callWith` where the routine removes its only holder mid-call | rc 0, `call 2` / `callvar 10` / `callwith 9`, `.context~executable` answers a Routine | identical | identical, 140 collections |
| `m5_unnotified` | a party raising in `messageComplete` leaves the message unnotified. A second `notify` does not fire. The message is dropped, then 20 fresh messages are notified | rc 0, no fire, then 20 `good notified` | identical | identical, 182 collections |

No pruned row was needed after its object died. Two differences are not this task's:
- In m1, stderr from a started activity's untrapped raise names line 35 where the oracle names 26. The Task
  6 binary does the same.
- In m2, the order of `slow 1 done` against `b result` differs. That is scheduling.

Two pre-existing divergences turned up on the way, both on the pin as well:
- `o~instanceMethod('Y')` after `self~setMethod('Y', aMethodObject)` answers `.nil`. The oracle answers a
  Method (`m3b.rex`: oracle `held 2 0 0`, crate rc 159, 97.1 on `.nil`).
- Error 47.2's label is spelled `"&1"` where the oracle spells `"S1"`
  (`do i=1 to 2; signal on syntax name s1; s1: end`).

### 3. The trigger charge

- `charge_unheld` (`lib.rs:2927`) adds only to `Heap::bytes_since` (`heap.rs:172`), which a collection
  resets. It never touches `held_bytes`, so nothing has to be uncharged at free and nothing can underflow.
- `bytes_due = max(floor, live_bytes)` does not include program weight, so a live program never raises
  the threshold.
- Double charging: `new_subclass` does not charge. `class_factory` (`class_protocol.rs:739`), the
  enhanced dummy (`:1188`) and `install_class` (`install.rs:1266`) each charge once. A program and its
  package are charged once each, at load and at the package's single build.
- Minor: `install_class` charges right after `new_subclass` (`install.rs:1266`), but the class's
  `::METHOD`s are installed later (`install.rs:1410`). So a `::CLASS`'s own methods do not count toward
  the charge. `cls50weak` still clears at 53 against the oracle's 9.

### 4. Memory

Measured with `/usr/bin/time -f %M` on the binary, `ulimit -v 4194304`, fresh directory per run
(`mem.sh`):

| loop | HEAD | change | oracle |
|---|---|---|---|
| `Method~new` 20000 / 80000 | 45044 / 44552 kB | -1.1% | 20588 / 20584 kB |
| `Package~new` 20000 / 80000 | 54856 / 54256 kB | -1.1% | 20088 / 20324 kB |
| `call ext` 20000 / 80000 | 55588 / 55064 kB | -0.9% | 20036 / 20908 kB |
| msgloop 80000 / 320000 | 43636 / 42884 kB | -1.7% | 20532 / 20692 kB |

Every growth bar holds and the report's figures reproduce.

### 5. Untested prunes and charge: each one can be witnessed

I built the tests below in the archive copy, ran them green at HEAD, and ran them red against each mutant
(full text in `witness-tests.rs.txt`).

- **P6** (`unnotified_messages`): run, with `bootstrap_library()` first, a program where a
  `MessageNotification` party raises in `messageComplete` on `m~send` inside a procedure, three times. Assert
  `unnotified_messages` is non-empty. Then `collect_now`, and assert every row is live.
  - Green at HEAD.
  - Red with `lib.rs:3263` commented out (the assert at line 415 of the copy's `run/tests.rs`).
  - It is white-box only. `clear_completion` (`object_protocol.rs:1201`) removes the row on the next
    send, so a stale row is never observable. m5 is green under the mutant's shape for that reason.
- **P7** (`message_ids`): in the same test, before `collect_now`, call `interp.activities.message_id(msg)`
  for a message the program dropped. Assert `message_rows()` are all live afterwards.
  - Red with `scheduler.rs:347` commented out (the assert at line 416 of the copy's `run/tests.rs`).
  - A behavioural shape needs a waiter unwound off a message that never completes. The oracle blocks
    indefinitely on an unsent message's `result` (oracle-crashes.txt), so the white-box test is the
    practical witness.
- **C2** (`PACKAGE_BYTES`): `load_program` a `nop` program. Read `heap.charge_body_bytes(0)` before and
  after `package_object(Package::Program(id))`, and assert the difference is at least `PACKAGE_BYTES`.
  - Green at HEAD.
  - Red with `identities.rs:228` commented out (line 427 of the copy's `run/tests.rs`).
  - No behavioural test can isolate it, because every program package follows its program's own charge.
    The alternative is to fold `PACKAGE_BYTES` into `PROGRAM_BYTES` and delete the separate charge.

### 6. The 8 MB floor

`rust/bench-programs/callgrind.sh -B 0.5 -r 2 -j 28 pin=<pin> head=<HEAD> floor8=<HEAD, COLLECT_BYTES_FLOOR = 8 << 20>`,
full set. It exited 0 in 8m33s, with no OVER and no SPREAD (`callgrind-table.txt`).

- HEAD against the pin reproduces the report's figures exactly: startup +0.3359%, hashnew +0.1097%,
  compound +0.4347%, alloc -0.7166%, pingmsg -1.1210%. Nothing is over +0.5%.
- floor8 against HEAD, whole process: every program is within 0.003%, except hashnew -0.129% (its
  `-libc` column is +0.085%).

The benchmark set is blind to the floor. Almost no program in it moves more than 8 MB of body bytes
between collections. A program that does, with a live set the collector has to mark, pays for a lower
floor. The worst-case programs `worst.rex`, `mid.rex` and `small.rex` hold 200000, 20000 or 2000 live
objects while making 20000 `copies('x', 10000)`. One callgrind run each:

| live objects | HEAD Ir | floor8 Ir | change |
|---|---|---|---|
| 200000 | 1,160,777,114 | 1,773,982,958 | **+52.8%** |
| 20000 | 282,763,954 | 345,975,831 | **+22.4%** |
| 2000 | 196,154,748 | 204,292,796 | +4.1% |

Peak RSS on `worst.rex` is 79188 kB at HEAD and 54708 kB with floor8.

Recommendation: **do not adopt 8 MB as it stands.** Its "no measured cost" holds only because the set has
no body-churn program. If the floor is to move, two things should come first:
- add a churn program like `mid.rex` to `bench-programs`;
- scale the floor with the live slot count, or lower only the share charged by `charge_unheld`.

That is collector tuning under the GC-timing licence, and outside 9b1.

## Strengths

- `prune_dead_object_rows` is one place, run after the sweep and before `free_unheld`, and every prune is
  `heap.peek` liveness. Every probe shape agrees with the oracle under collect-every-allocation.
- The mutation table is honest: P5 is recorded as green before `run_started_to_end` was added, and P6, P7
  and C2 are reported as unwitnessed rather than hidden.
- The program charge is measured, not guessed: about 30 kB per one-line program, plus per source byte.
  It cut the `Package~new` loop from 419 MB to 54 MB and `cls50weak` from 1.3 GB to 78 MB.
- The five Task 7 representation refusals became one internal-guard constructor, with a disposition row
  backed by committed probes.

## Issues

### Critical

None.

### Important

1. The `loadExternalMethod` identity rows leak: `defined_library_codes` and `library_externals` through
   `bind_loaded_method`, `environment/identities.rs:633`-`:643`. About 110 B per call, 80 MB at 320000
   calls against the oracle's flat 20 MB. This map is in the brief's group and no ruling covers it. Fix:
   rule it into Task 8 with the `Method~copy` identities, since it is the same owner question, or prune
   rows whose MethodId no live executable names.
2. P6, P7 and C2 have no test that fails without them (`lib.rs:3263`, `scheduler.rs:347`,
   `environment/identities.rs:228`). Section 5 has a witness for each, each verified red under its
   mutant. These are already queued for the fix round.

### Minor

1. `install.rs:1266`: a `::CLASS` is charged before its methods are installed (`:1410`), so its own
   methods carry no weight. Moving the call after method installation would make `cls50weak`-like loops
   collect sooner.
2. `PROGRAM_BYTES_PER_SOURCE_BYTE` has no mutation of its own. C1 removes the whole charge. Mutating
   only the per-byte term against `cls50weak`'s 300 bound would show whether it carries weight.
3. Out of this task's scope, pre-existing on the pin: `instanceMethod` answers `.nil` after `setMethod`
   with a Method object (`m3b.rex`), and 47.2 spells the label `"&1"`. Worth queueing.

## Assessment

**Spec:** ❌. One map in the brief's group, the `loadExternalMethod` identity rows, is neither a field
nor pruned, and no ruling covers it. Everything else in Steps 1 to 3 is met and verified.

**Task quality:** Needs fixes. Every prune checked is safe against the oracle under
collect-every-allocation, and the memory and budget figures reproduce exactly. The `loadExternalMethod`
leak needs a ruling, and P6, P7 and C2 need their witnesses. The 8 MB floor should not be adopted on the
current evidence: it costs +22% to +53% instructions on body-churn programs the bench set does not
contain.
