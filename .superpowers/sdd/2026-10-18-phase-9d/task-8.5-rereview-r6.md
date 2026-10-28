# Task 8.5 re-review, fix round 6 (controller fix)

Reviewed `521eb0aff..d1e1a5ba9` (code `44bf4419d`, `5c71b2b24`; the second supersedes the first). HEAD was
built from the checkout (only `progress.md` modified) with `CARGO_INCREMENTAL=0
CARGO_TARGET_DIR=/tmp/claude-1000/t85-rr6-target memcap 8G cargo build --release -j 4 --bin rexx`, exit 0.
Its `.text` is byte-identical to `cg-9d/t8.5/rexx-head-5c71b2b24` (`objcopy -O binary --only-section=.text`,
md5 `0773c9a2...` both), so the probes ran on that pinned binary. Earlier answers come from
`rexx-head-2cda11d12` (round 5), `rexx-head-6a8290bb6` (round 4), `rexx-head-4a6321376` and
`rexx-base-6e0bf5e5b`. Every crate run is `memcap 2G timeout -k 5 20` in a fresh `mktemp -d` under
`t85-rereview6-scratch/run/`; stdout, stderr, rc and peak RSS (`/usr/bin/time -f %M`) compared separately.
The oracle ran only acyclic programs (`fp1` to `fp6`, `m1` to `m5`, `tr4`).

For the collector and the witnesses I archived `5c71b2b24` and `2cda11d12` (`git archive ... rust
interpreter`) to `/tmp/claude-1000/t85-rr6-src{,5}` and built there with their own target dirs: an example
`stress.rs` that runs a file under `run_program_collect_every_alloc` (no CLI switch exists), and the two new
unit tests appended to round 5's `tests.rs`. Nothing in the checkout was written outside the scratch dir and
this file.

## Verdict

**APPROVED.** Critical none, Important none, Minor 2. New oracle defects none.

## Verdicts on the fix brief

1. **No walk at the outermost forward; a cycle under it ends in 11.1 without the native stack: ADDRESSED.**
   * `p/h1.rex N` on head: 2000 nodes 0.0014 s, 10000 0.0071 s, 20000 0.0138 s, 100000 0.092 s (TIME('E')
     over the sends), `sum 200010000` at 20000; linear, matching the report and round 5's `4a6321376`
     column. Peak RSS 23 to 147 MB, growing with the chain only.
   * Minor 1's shapes on head, each `SYNTAX 11.1`, rc 0: `f1` 20296 KB 0.08 s, `f2` 20432 KB, `g1` 20160
     KB 0.04 s, `g2` to `g8` 20028 to 20548 KB, all under 0.08 s (round 5: 580 MB to 1.15 GB). `g1` under
     `memcap 1G`: `SYNTAX 11.1`, rc 0. `g9` (`reply` then a cycle on the new activity): `replied`, `end`.
     `f5.rex` at 20, 2000, 20000 and 100000 iterations: `done caught 11.1`, peak 20380, 36324, 36348,
     36932 KB (`6a8290bb6`: 36932 to 36984 KB at the same counts), so flat.
   * The mechanism (`Interp::forwarded`, `value.rs:1282`): the anchor is the receiver at the largest power
     of two at or below the current depth of the open forwards on the activity, restored from `saved` on
     every return, so a cycle of `L` hops after a prefix of `p` is found by depth `2^k + L` with `2^k`
     at least `max(L, p+1)`; the 11.1 return leaves the entry as the enclosing forward set it.
2. **Witnesses and records: ADDRESSED**, with the weaknesses in Minor 1.
   * `cargo test --release -p rexx-exec --lib -- a_forward_answered_after_one_hop_costs_one_hop
     a_cycle_made_inside_a_forward_is_insufficient_stack reference cyclic long_chain` at `5c71b2b24`:
     `21 passed; 0 failed`, finished in 2.09 s.
   * Appended to round 5's code: the h1 witness FAILED at its timeout (`tests.rs:1804`, "the forwards did
     not end", finished in 20.00 s); the cyclic witness passed in 4.78 s.
   * Round 4's and round 5's probe sets (`t85-rereview5-scratch/p` and `r4p`, every `.rex` but `h1`,
     `chains`, `halt`, run without arguments, run dir names masked): identical stdout, stderr and rc on
     head and `2cda11d12` except `p/f5.rex`, whose default of 1000000 iterations ends `done caught 11.1`
     rc 0 on head and is interrupted at 20 s (rc 124) on round 5, which is the fix.

**Perf: confirmed.** `cg-9d/t8.5/r6final2` holds 198 `.vg` files, 33 programs, three rounds per side; 99
head `Command:` lines name `rexx-head-5c71b2b24` (whose `.text` matches my build) and 99 name
`rexx-base-6e0bf5e5b`; every `.rc` is 0. `r6/callgrind.txt` libc subtracted: largest rises fibfunc
`+0.0213`, fibcall `+0.0209`, pingmsg `+0.0002`; pingguard `-0.0001`, pingsem `-0.0002`; every program
under +0.05%. The ping rows match the `.cg` totals (pingmsg head medians 7527412248 against base
7527368112). My own callgrind of `pingguard.rex`: base 1319155392, head 1319154914 Ir.

## The new mechanism, hunted

* **Every exit path removes the entry.** All exits from `forward` are `Result` returns through the cleanup
  at `value.rs:1317` to `1330`; the only `catch_unwind` sites (`scheduler/pool.rs:41`, `dispatch/library.rs:1295`)
  resume or report a panic. `p/l1.rex N` runs, per iteration, forwards ending by a trapped `RAISE SYNTAX`
  (on a stem and on a subclass instance), the same raised two forwards deep, `SIGNAL` inside the
  `UNKNOWN`, `RAISE HALT`, `RAISE USER`, a cycle's 11.1 one and two forwards deep, and `reply` followed by
  a forward on the new activity; then `s.~ok`, `u~ok`, another deep raise and a fresh stem's forward. At
  100, 1000 and 10000 iterations every count is right, `after ok ok E 40.1`, `fresh ok`, peak RSS 31352,
  38660, 38892 KB. A leaked entry of depth 2 or more would have made `s.~ok` 11.1. (`2cda11d12` times out
  at 20 s at 1000 iterations; `4a6321376` overflows its stack.)
* **Collector.** Under collect-on-every-allocation (`stress` example, 66 to 384109 collections per run):
  `f1`, `f2`, `g1` to `g3`, `g5` to `g9`, `fp2`, `fp5`, `l1 1000`, `c2`, `c3` give the same stdout as the
  normal binary. `g4` panics there, but on a defect older than the task (out of scope, first item).
* **Concurrent forwards.** `p/c1.rex` and `p/c2.rex` (six started messages each building a 1000 or 2000
  node h1 chain, five passes of one-hop forwards plus a trapped cycle every 97 nodes, `syssleep 0` inside
  the forwarded methods in `c2`): six runs of `c2`, five of `c1`, every activity `2502500 50` /
  `10005000 100`, then `after ok`. `p/c3.rex` (two activities wait on an event semaphore inside a forward
  while main and a third activity trap cycles and the third ends; then the waiters build cycles): three
  runs `cyc 11.1 mc 11.1 ok ok ma held 11.1 mb held 11.1 cyc 11.1 ok ok end`, also under stress.
* **False positives.** `p/fp5.rex`: an object's methods recurse back through the stem by send, by
  `FORWARD`, by `FORWARD CONTINUE`, through `UNKNOWN` alternating message names, and 3000 deep: all
  `done`, as on round 5, `4a6321376` and the oracle. A repeat of the same receiver at the same activation
  depth needs a native method that sends back to the stem it was forwarded from; I tried the native
  senders I found (`Object~send` via a `Message` held as the default, `fp3`/`fp4`; `Message~notify`,
  `fp6`; argument conversions, `fp1`/`fp2`). Each either never reaches the second nested forward or stops
  on an error that the oracle and every side give identically (`97.1` "does not understand", `88.914`).
  None found.

## Issues

### Critical

None.

### Important

None.

### Minor

1. **The two witnesses are weak against what they witness.** The h1 test
   (`tests.rs`, `a_forward_answered_after_one_hop_costs_one_hop`) fails on round 5's code only by about
   5% (the report's 21 s against the 20 s timeout; my run ended at 20.00 s), so a quadratic regression
   with a slightly smaller constant, or a faster machine, passes it. The cyclic test
   (`a_cycle_made_inside_a_forward_is_insufficient_stack`) passes on round 5's code (4.78 s), so it does
   not see Minor 1's cost at all. The report says no peak-memory or depth assertion is practical; a wall
   time bound is (head finishes the nine bodies in well under a second, round 5 in 4.78 s), and a larger
   N or a two-size ratio would widen h1's margin.
2. **Dead branch.** In `Interp::forwarded` (`value.rs:1308`) the 11.1 return's `if depth == 1 {
   swap_remove }` cannot run: the return needs `!depth.is_power_of_two()` and 1 is a power of two.

## Out of scope

* **`trace i` over a reference assignment crashes, and did before Task 8.5.** `p/tr4.rex` (`trace i`, then
  `q = >y` 300000 times) panics `dispatch.rs:1831:9: a live value` (rc 101) on head and on
  `rexx-base-6e0bf5e5b`, without stress mode; `p/tr2.rex` (one such assignment) panics under stress mode.
  Backtrace: `assign_evaluated` -> `result_text` -> `string_value_text` -> `to_text` -> `not_in_arena`,
  so the traced result (the fresh reference) is not rooted while it is rendered. `trace r` (`tr3`) does not
  panic. A rooting defect worth queueing; it predates this task's base.
* `.message~new(x., ...)` held as `x.`'s default and sent: forwarding back to that running message is
  `97.1` on the oracle and every side (`m5`); `Message~notify` refuses a stem (`88.914`). Both agree with
  the oracle.

## Queue candidates

* The `trace i` reference-assignment rooting crash above (`p/tr2.rex`, `p/tr4.rex`).

## New oracle defects

None. The oracle ran only acyclic programs and agreed with the crate on all of them except `tr4`, where it
parses `q = >y` as an expression rather than a reference (not a defect of the oracle for this review).
