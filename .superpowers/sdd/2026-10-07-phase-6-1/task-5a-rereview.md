# Task 5a fix round 1 re-review

Reviewer t5a-rereview, 2026-10-09. FIX_BASE `d9794b41f`, HEAD `17cbe8a9e`, diff
`review-d9794b41f..17cbe8a9e.diff` read whole (ledger commit `19e3b9d34` ignored). Tree untouched.
Scratch `R` = `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t5arr`.

Binaries: `git archive` of `d9794b41f` (base), `17cbe8a9e` (head), `37d874a36` (b2, the task's
base) and head with `COLLECT_BYTES_FLOOR = 1 << 60` (off), each `rust` + `interpreter`, `find -exec
touch`, `CARGO_TARGET_DIR=R/t<name> memcap 8G cargo build --release -j 4 -p rexx-exec --bin
rexx-run`, one `Compiling rexx-exec` line each (`R/build-*.log`). In base, head and off only
`rexx-run.rs` is changed: `REXX_PROBE_STATS` prints `Outcome::collections` to stderr,
`REXX_PROBE_STRESS` runs `run_program_collect_every_alloc`. Probes `R/probes/*.rex`, runner
`R/run.sh` (fresh `mktemp -d` per run, `memcap 2G /usr/bin/time -f "%M %e %x" timeout -k 5 120`),
results `R/r1.txt` to `R/r5.txt`; oracle `R/orc.sh` (standard wrapper, fresh directory),
`R/orc.txt`. One run per cell unless stated. RSS in KB.

### Finding Verdicts

- **Important 1: `~copy` of a heap string builds an uncharged Text body** -- ADDRESSED.
  `object_protocol.rs:633` allocates through `Interp::alloc_charged` (`lib.rs:2678`, with `charge_growth` at `:2670`), which
  charges `Body::held_bytes` taken before the move. `copy` (`y~copy` of 300 KB, 3000 passes): base
  899 224 / 0 collections, head 53 028 / 26 collections, oracle 15 864. Crate test
  `copied_strings_are_collected_on_their_bytes` (`tests.rs`) is in the diff.
- **Ruling: MutableBuffer and Array bodies are charged (creation and capacity growth)** -- NOT
  ADDRESSED for one growth path; see New Breakage, Important 1. Every other shape probed is bounded
  (base / head / head collections / oracle):

  | probe | shape, passes | base | head | coll. | oracle |
  |---|---|---|---|---|---|
  | `arrnew` | `.array~new(100000); a[100000] = i`, 1000 | 1 583 196 | 54 824 | 47 | 20 544 |
  | `arrput` | `.array~new` then `a[k * 50000] = k`, k = 1..10, 400 | OOM 2G | 67 060 | 80 | 42 144 |
  | `arrappend` | `a[200000]`, two `append`, `a[400001]`, `append`, 400 | OOM 2G | 74 524 | 133 | 53 272 |
  | `arrinsert` | `a[200000]`, `insert('x', 1)`, `insert('y')`, 400 | OOM 2G | 65 920 | 66 | 38 868 |
  | `arrcopy` | `b~copy` of a 100 000-slot array, 1000 | 1 584 264 | 56 604 | 47 | 24 652 |
  | `arrof` | `s~section(1)` of a 100 000-slot array, 1000 | 1 584 624 | 56 156 | 47 | 24 372 |
  | `arrmulti_new` | `.array~new(1000, 1000)`, 300 | OOM 2G | 97 360 | 99 | not run |
  | `mbappend` | `~new`, `append(s, s, s)`, s 100 KB, 3000 | 910 168 | 44 560 | 35 | 13 724 |
  | `mbinsert` | three `insert`s of s, 3000 | 911 476 | 53 664 | 35 | 15 748 |
  | `mboverlay` | `overlay(s, 1)`, `overlay(s, 200001)`, 3000 | 911 592 | 53 112 | 35 | 14 436 |
  | `mbsetsize` | `~new; setBufferSize(300000)`, 3000 | 32 052 / 0 coll. | 19 596 | 26 | 14 920 |
  | `mbnewcap` | `.mutableBuffer~new('', 300000)`, 3000 | 31 608 / 0 coll. | 19 676 | 26 | 14 228 |
  | `mbnewtext` | `.mutableBuffer~new(s)`, s 300 KB, 3000 | 899 500 | 53 024 | 26 | 14 488 |
  | `mbmix` | append, insert, overlay, `a[60000] = b~string`, `append(b~copy)`, `changeStr`, concat; 2000 | 181 128 / 54 coll. | 58 936 | 243 | 25 588 |

  Output identical across base, head and oracle in every row that completed. (`mbsetsize` and
  `mbnewcap` reserve without touching, so base's RSS stays low; the 0 against 26 collections is
  the change.)
- **Minor 1: 35% cycle regression on `array_fill20` invisible to callgrind** -- ADDRESSED (diagnosed
  as one build's placement, recorded in the gate record). Pinned `perf stat`, 3 interleaved runs
  (`R/inter-af20.txt`): b2 3.04 to 3.06 G cycles, head 3.03 to 3.05 G, off 3.03 to 3.07 G;
  instructions 18.029 to 18.030 G on all three. No slow mode on this build either.
- **Minor 2: stale doc on `stress_collect` and `collect_at`** -- ADDRESSED. `lib.rs:1463`-`:1467`
  now matches `enable_stress_collect` (`:2632`-`:2635`, sets both) and `collect_now` (`:2961`,
  `collect_due = stress_collect`); `collect_at` (`lib.rs:1500`-`:1503`) names both halves.
- **Minor 3: `Outcome::peak_body_bytes` is an outward field** -- ADDRESSED. `#[cfg(test)]` at
  `lib.rs:322`-`:323` and at both constructors; the integration-test literals no longer name it.
- **Minor 4: short-string test blind to the inline exemption** -- ADDRESSED. `tests.rs`
  `short_strings_keep_the_slot_cadence` asserts `peak_body_bytes < 1 << 18`; the report's
  inversion (exemption removed: 1 513 074) is the witness. The exact `collections == 6` pin stays;
  the review offered a range as a suggestion, not as the defect.
- **Minor 5: `loop999`'s cost not in the gate record** -- ADDRESSED.
  `docs/superpowers/plans/phase-6-1-gate.md` `### loop999's cost` and the `### Fix round 1`
  diagnosis paragraph.

### Checks run

- **Shrink leaves no phantom charge** (`R/r3.txt`). One allocation per pass so a due collection can
  run: `mbshrink3` (`setBufferSize(1000000)` then `setBufferSize(10)`, 20 000 passes) head 588
  collections, predicted 20 000 x 999 990 / 2^25 = 596; `mbshrink4` (`append` of 500 KB then
  `setBufferSize(0)`, 20 000) head 294, predicted 298. Periodic, proportional to real churn, not one
  per allocation (20 000 allocations each). RSS 19 120 / 21 032 (base 22 724 / 23 168, 0
  collections). `arrshrink` (`a[100000]`, two `delete`s, 20 000) 0 collections on both: capacity
  is kept, so regrowth charges nothing. Cadence controls equal base: `shortref` 6 / 6, `objref`
  0 / 0, `mbdelete` (in-capacity delete and append on a 1 MB buffer, 200 000) 0 / 0, `arrchurn`
  0 / 0; `biglive` (48 MB array + 50 MB buffer live, 200 000 short concats) base 7, head 8.
  Without an allocation in the loop (`mbshrink`, `mbshrink2`) head makes 0 collections: the due
  flag waits for an allocation, and shrinking freed the memory, RSS 19 516 / 20 404.
- **Live values survive** (`R/r4.txt`). `live2000` / `live200`: per pass a dead 300 KB string, a
  buffer appended, grown by `setBufferSize(200000)` and appended again, an array grown by
  `a[i * 10]` then `append` then `a[1]` set after growth, and one long-lived array grown each pass
  by `g[i * 100]`; every 50th array and buffer kept, all contents compared at the end. head normal
  2000 passes: `kept 40 g 2000 bad 0`, 48 collections (base 17); head stress 200 passes: `bad 0`,
  1235 collections; oracle `bad 0` for both sizes.
- **Perf.** The gate record quotes the callgrind command, both binaries' shas and base `37d874a36`.
  Spot-check, `memcap 8G bash rust/bench-programs/callgrind.sh -r 1 -j 3 -o R/cg -p rexxcps
  b2=R/tb2/release/rexx-run base=R/tbase/release/rexx-run head=R/thead/release/rexx-run`, exit 0:
  base +0.2034%, head +0.2383% against b2, spreads 0.0000%. The head figure reproduces the gate
  record's rexxcps cell exactly.
- **Wall-clock diagnosis.** `loop999` collections: head 59, off 30 (base's cadence, so the floor
  edit took). Pinned (`taskset -c 4`) `perf stat` cycles, 8 interleaved rounds under `memcap 2G`
  (`R/inter.sh`, `R/inter-loop999.txt`): b2 3.12 to 3.19 G; head 3.27 to 4.01 G; off 3.17 to 4.27 G
  (6 of 8 at or above 3.65 G). Instructions b2 14.45 G, head 14.40 G, off 14.47 G. The key claim
  holds: with the byte trigger off the slow mode persists, so it is not this task's collections.
  The cause the report names (data placement in `copies_bytes`) was not re-checked.

### New Breakage in the Fix Diff

**Important 1. A multi-dimensional extend replaces an array's slots uncharged.**
`rust/crates/rexx-exec/src/dispatch/array.rs:375`-`:414` (`array_reshape`, store at `:409`) builds `grown =
empty_slots(size)` and stores it with `*slots = grown` with no `charge_growth`. It is the one slot
replacement the round's growth sites (`array_resize`, `array_grow`, `array_splice_slot`) do not
cover. `arrmulti60` (`a = .array~new(10, 10); a[1000, 1000] = i`, 60 passes, about 16 MB of slots
each): base 956 976, head 956 744 in 0.08 s, 0 collections on both; oracle 68 580. At 300 passes
both base and head are OOM-killed at the 2G cap (`R/r1.txt`). Creating the same shape directly
(`arrmulti_new`, `.array~new(1000, 1000)`) is bounded (97 360, 99 collections), so only growth
reaches it. The fix round is where arrays became charged on capacity growth and the gate record
(`### Fix round 1`) says arrays are charged on "creation and slot growth", which this path makes
false. Fix: in `array_reshape`, charge `grown.capacity()` minus the replaced capacity, times
`SLOT_BYTES`, after the store (the `array_resize` pattern), with an `assert_collected_on_bytes`
case for the loop above.

Enumeration behind "the one": every `Body::Array` slot mutation in `rexx-exec/src` (`grep -rn
"slots\.\(push\|resize\|insert\|extend\|reserve\|try_reserve\|append\|splice\)\|slots = \|\*slots"`)
outside tests: the charged sites above, slot vectors built before an `alloc_charged`,
`array.rs:97` (`empty_slots`, charged when allocated), `array.rs:409` (this one), and
`object_protocol.rs:1239` (a `~notify` party pushed onto a Message's array: one slot per
call, not a growth shape). No `BehaviourId::ARRAY` allocation outside tests bypasses
`alloc_charged` (`grep -rn -A1 "alloc_with(\|alloc_immortal_with(\|alloc_with_uncollected(" |
grep ARRAY`: only `value/tests.rs:279` and `eval/object_operand_tests.rs:575`).

No other breakage found: `held_bytes`' new arms agree with what the growth sites charge (slot
capacity times `SLOT_BYTES`, buffer `capacity`); every `ensure_capacity` / `buffer_capacity` /
`set_buffer_size` caller charges the growth it answers (`grep -n "ensure_capacity\|buffer_capacity(\|set_buffer_size("`
over `buffer.rs` and `library/surface.rs`); `set_mutable_buffer_capacity` still ignores a failed
reservation as before.

### Out-of-Scope Observations

None.

### Verdict

**Fix round:** Findings remain open -- the controller's ruling (arrays charged on capacity
growth) is not met for `array_reshape` (New Breakage, Important 1). Important 1 and Minors 1-5
of the review are addressed. Counts: Critical 0, Important 1, Minor 0.

## Fix round 2

HEAD `1fd487d02` (code `72287a744`). Diff `17cbe8a9e..1fd487d02` read: `array.rs`,
`object_protocol.rs`, `tests.rs`, the gate record, the report's `## Fix round 2`. Binaries: `git
archive 1fd487d02 rust interpreter` into `R/fr2`, `rexx-run.rs` printing `collections` under
`REXX_PROBE_STATS`, own target dir, one `Compiling rexx-exec` (`R/fr2-build.log`). Base figures
are this re-review's runs of `d9794b41f` and `17cbe8a9e` above.

- **Reshape charged** -- ADDRESSED. `dispatch/array.rs:404`-`:417` (capacity read at `:409`, charge at `:416`) takes the slot capacity before
  `*slots = grown`, charges the delta times `SLOT_BYTES` after the borrow ends (the `array_resize`
  pattern). `arrmulti60`: base 956 976 / 0 collections, `17cbe8a9e` 956 744 / 0, head 81 868 / 20;
  `arrmulti` (300 passes): base and `17cbe8a9e` OOM at 2G, head 81 412 / 100, output `1000000 300`
  as the oracle (69 692 at 300). `R/r6.txt`.
- **`reshaped_arrays_are_collected_on_their_bytes`** -- present and live. `memcap 8G cargo test
  -j 4 -p rexx-exec --lib collected_on_their_bytes` at `1fd487d02`: exit 0, 6 passed. Negative
  control: the same tree with `dispatch/array.rs` from `17cbe8a9e`, own target dir
  (`R/tfr2n`, one `Compiling rexx-exec`): exit 101, only this test fails, `0 collections for 60 x
  16000000 bytes` (`R/fr2neg-test2.log`). (A first control run sharing the head's target dir
  compiled nothing and passed; discarded.)
- **`~notify` push** -- correct. `object_protocol.rs:1239`-`:1242`: capacity before and after one
  `push`, delta charged; a push never shrinks, so the unsigned subtraction cannot underflow.
  `notify` (5000 `~notify` of one counter, `~send`, one late notify): stdout byte-identical to the
  oracle (`result 3 notified 5000` / `late 5001`), stderr empty on both, rc 0 on both.
  `notify_big` (200 000 notifies, an allocation each pass): rc 0, `3 200000`, 34 268 KB.
- **Gate-record sentence** -- true. "arrays (creation, and growth through `array_grow`,
  `array_splice_slot` and `array_resize`; a multidimensional extend was left uncharged until fix
  round 2)" describes `d0a3d5db3`: those three charge there, `array_reshape` does not. It does not
  claim those were the only uncharged paths, so the also-uncharged notify push at that commit does
  not falsify it.
- **No regression** on the round-1 probes at head: `live2000` `bad 0` (48 collections), `copy`
  52 404 / 26, `mbmix` 58 800 / 243, `arrmulti_new` 96 672 / 99.

New breakage: none.

**Fix round 2 verdict:** All findings addressed, no new Critical/Important breakage. Counts:
Critical 0, Important 0, Minor 0.
