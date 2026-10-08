# Task 5a review: a collection trigger that counts bytes

Reviewer t5a-review, 2026-10-09. BASE `37d874a36`, HEAD `d9794b41f`, diff
`review-37d874a36..d9794b41f.diff` read whole. Tree untouched. Scratch
`R` = `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t5arev`.

Binaries: `git archive` of `37d874a36` and `d9794b41f` into `R/base`, `R/head`, `find -exec touch`,
`CARGO_TARGET_DIR=R/tbase|R/thead memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run`,
each with a `Compiling rexx-exec` line. A layout control `R/tpad` = base + `layout-pad.py 48`, built
the same way. In all three scratch copies only `rexx-run.rs` is changed: `REXX_PROBE_STATS` prints
`Outcome::collections` (and on head `peak_body_bytes`) to stderr after the run, and `REXX_PROBE_STRESS`
calls `run_program_collect_every_alloc`. The interpreter is unchanged.

Probe runs: `R/run.sh` (each probe from a fresh `mktemp -d` directory, `memcap 2G /usr/bin/time -f "%M
%e %x" timeout -k 5 120 BIN R/probes/P.rex`), probe texts in `R/probes`, results in `R/r1.txt` to
`R/r7.txt`. Oracle: `( ulimit -v 1048576; LD_LIBRARY_PATH=$ORACLE/lib /usr/bin/time ... timeout -k 5
20 rexx P.rex )` from a fresh empty directory. One run per cell unless stated.

### Spec Compliance

- ✅ Spec compliant for R10 as the brief scopes it: the slot test is kept (`lib.rs:2653`), a byte
  test is folded into one bool (`collect_due`, set at `lib.rs:2665`, reset at the end of
  `collect_now`), `bytes_due` = max(`COLLECT_BYTES_FLOOR`, live bytes) (`lib.rs:2941`), live bytes
  are summed in both mark loops (`heap.rs` main loop and resurrection loop), sizes come from the
  source length, the failing-then-passing test and the short-string case exist (`tests.rs:1045`,
  `:1065`), and the gate record has `## Task 5a` with peak RSS on both switch modes and the oracle,
  the callgrind command and base, and `whole_groups` before and after.
- ❌ One Text-body construction site is not charged: `~copy` of a heap string (Important 1).
- ⚠️ `Outcome::peak_body_bytes` (`lib.rs:322`) is an outward field where the brief says none
  (concern 3, judged under Minor 3).

### Strengths

- The cadence is exactly periodic and drift-free: `do i = 1 to N; x = copies('abc', 100000); end`
  gives 8, 89 and 357 collections at N = 1000, 10000, 40000, which is floor(N x 300 000 / 2^25)
  each time, with `peak_body_bytes` 33 900 000 at every N and RSS flat at 52 MB (base OOMs at 2G
  from N = 10000).
- The fast path really is one bool load: `collect_due` replaces `stress_collect` in
  `collect_if_due` and the stress mode keeps working because `collect_now` resets `collect_due` to
  `stress_collect` (`lib.rs:2941`-`:2942`). Under `run_program_collect_every_alloc` the probes
  `st_concat` and `st_live` collect on every allocation (302 collections for 300 iterations) and
  print correct content in both switch modes.
- Every caller of `Heap::collect` goes through `collect_now` (checked: `grep -rn
  "heap\.collect(\|\.collect_now()" rexx-exec/src`: one `heap.collect`, at `lib.rs:2895`), so
  `collect_due` cannot be left stale by a collection that bypasses the reset.
- The perf round-1 work is careful: the charge moved out of `text_bytes` with the variant table to
  show why, and the `text_bytes` doc says so (`value.rs:220`-`:223`).
- `heap::body_bytes_tests` covers dead, resurrected, inline and the peak's max-of-two with an
  inversion recorded.

### Focus 1: bounded memory beyond the one loop

Max RSS KB, rc, head collections. Base collections were 0 in every row below except `mixed` (17).

| probe | shape | base normal / every | head normal / every | head collections |
|---|---|---|---|---|
| `concat_grow` | `x = x \|\| copies('a', 1000)` 2000 times | 1 979 252 / 1 978 404 | 59 084 / 59 344 | 58 |
| `concat_dead` | `x = y \|\| y`, 300 KB, 4000 times | 1 192 084 / 1 192 012 | 52 300 / 52 256 | 35 |
| `parse_words` | `parse var s a b rest`, 300 KB, 3000 | 898 404 / 898 604 | 52 448 / 52 292 | 26 |
| `parse_pos` | `parse var s a 150001 b`, 3000 | 898 856 / 899 076 | 52 428 / 51 992 | 26 |
| `stem_drop` | `s.i = copies('x', 300000); drop s.i`, 3000 | 898 200 / 898 332 | 52 592 / 52 832 | 26 |
| `stem_dropall` | 100 tails of 300 KB then `drop s.`, 30 times | 898 480 / 898 072 | 81 668 / 81 792 | 26 |
| `array_item` | `a[1] = copies('x', 300000)`, 3000 | 898 620 / 897 944 | 52 492 / 52 244 | 26 |
| `mbuf_self` | MutableBuffer doubled via `b~string`, 3000 | OOM 2G / OOM 2G | 122 140 / 121 924 | 29 |
| `live` | 30 of 3000 strings kept in an array and a stem, 1 PARSE result, then 2000 dead 300 KB strings; every kept value compared at the end | 1 198 124 / 1 198 048, `bad 0` | 59 928 / 60 012, `bad 0` | 35 |
| `args` | three 300 KB call arguments, `.array~of` of three 200 KB strings, a 400 KB concat; each checked, 1500 times | OOM 2G / OOM 2G | 53 928 / 53 912, `bad 0` | 160 |
| `reply` | REPLY then 2000 x 300 KB on both activities | 1 191 540 / 1 191 564 | 53 492 / 52 828 | 35 |
| `start` | two `~start`ed activities and the main one, 2000 x 300 KB each | 1 777 576 to 1 778 200 | 55 340 / 52 916, results `a300000 b300000 300000` | 53 |
| `mixed` | a 300 KB string and 20 short concats per pass, 20 000 passes | 458 820 | 52 880 | 180 |

No premature collection seen: `live` and `args` compare every kept or in-flight value byte for byte
after dozens of collections, in both switch modes, `bad 0`.

Still unbounded (concern 2 and Important 1), head the same as base, oracle bounded:

| probe | shape | head RSS | rate | oracle |
|---|---|---|---|---|
| `mbuf2000` / `mbuf` | `.MutableBuffer~new` then three appends of a 100 KB string, 2000 / 3000 times | 614 424 KB in 0.18 s / 911 316 KB | about 300 KB per pass, about 3.3 GB/s; 0 collections | 15 704 KB (2000) |
| `array_big1000` / `array_big` | `a = .array~new(100000); a[100000] = i`, 1000 / 3000 times | 1 583 372 KB in 0.38 s / OOM at 2G | about 1.58 MB per array (16 bytes per slot), about 4 GB/s; 0 collections | 22 064 KB / 19 856 KB |
| `strcopy` | `x = y~copy` over a 300 KB string, 3000 times | 899 148 KB in 0.22 s | 300 KB per pass, about 4 GB/s; 0 collections | not run |
| `array_fill` | 200 arrays of 10 000 appended small integers | 69 644 KB (base 69 784) | not distinguishable at this size | not run |

### Focus 2: byte accounting

- Symmetry and drift: `bytes_since` is zeroed and `live_bytes` recomputed at every collection
  (`heap.rs` at the end of `collect`), so nothing accumulates across collections; the cadence figures
  above (8 / 89 / 357) are the empirical check.
- Freed excluded, resurrected included: covered by `a_collection_sums_the_survivors_body_bytes`
  (200-byte dead body excluded, 300-byte UNINIT body included). Runtime probe `uninit`
  (1000 objects with an UNINIT method each holding a 300 KB string): head 4 collections, peak 300 MB,
  RSS 312 632 KB; the resurrected bodies are counted live, so `bytes_due` doubles with them. See the
  aside below for why they stay live.
- `collect_due` stuck true would show as a collection per allocation and stuck false as 0
  collections; neither appears in any row (normal mode 8 to 357, stress mode one per allocation).
- `REXX_SWITCH_MODE=every`: every head row matches normal mode in collections and peak.

### Focus 3: concern 1 is real

`loop999` = `do i = 1 to 1000000; x = copies('abc', 333) || i; end`.

- 10 interleaved runs (`R/inter.sh`, `R/inter-loop999.txt`), wall s: base 1.03 to 1.20, median
  1.045; head 1.03 to 1.25, median 1.235 (+18%). 9 of 10 head runs are slower than base's median.
- Pinned (`taskset -c 4`), 6 interleaved runs: base 1.03 to 1.06; head 1.12 to 1.26. With ASLR off
  (`setarch -R`): base 1.03 to 1.06; head 0.95 to 1.27.
- `perf stat`, pinned: instructions base 14.45 G, head 14.40 G; cycles base 3.09 to 3.18 G (pad48
  control 3.08 to 3.13 G), head 3.06 to 3.86 G. `cache-misses` are not higher on head (3.2 to 3.8 M
  against 3.7 to 4.3 M). Head collects 59 times, base 30.
- So the cost is real, in cycles not instructions, mostly present (the fast mode is the minority),
  and not LLC misses. Not diagnosed further. The pad48 control is flat, but see Minor 1: a second
  program shows this binary has a layout-induced cycle regression that callgrind cannot see, so part
  of `loop999`'s cost may be layout rather than the extra collections.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

1. **`~copy` of a heap string builds an uncharged Text body.**
   `rust/crates/rexx-exec/src/dispatch/object_protocol.rs:627`-`:633` clones the receiver's
   `Body::Text` and allocates it through `alloc_with` with no `charge_text`. `y = copies('a',
   300000); do i = 1 to 3000; x = y~copy; end` reaches 899 148 KB on head (0 collections), the same
   as base's 898 392 KB; at 1000 passes 312 268 KB. This is the shape the task exists to bound, through
   a Text body, the one body kind the design says is counted (report "What is counted"). Fix: in
   `copy_object`, take `body.held_bytes()` before the move and charge it after the allocation
   (`interp.charge_body_bytes(n)` when `n > 0`), with a crate test of the loop above asserting
   `collections > 0` and the peak bound. Enumeration behind "only this one": every `alloc_with` /
   `alloc_immortal_with` / `heap.alloc` call in `rexx-exec/src` whose body is not a literal
   constructor (`grep -rn "alloc_with(\|alloc_immortal_with(\|alloc_with_uncollected(\|heap.alloc("
   rexx-exec/src -A2`, literal `Body::` lines removed): `stem.rs:601` (a stem), `string.rs:1974` (an
   array), `object_protocol.rs:633` (this one); `value.rs:136` `interned_literal` is immortal and
   needs no charge. Dynamically, `.String~new(y)`, `reverse`, `~reverse`, `translate`, `substr`,
   `~makeString` all collect (`R/r6.txt`).

#### Minor (Nice to Have)

1. **A 35% cycle regression on an Array-append loop that callgrind cannot see.** `array_fill20`
   (`do i = 1 to 20; a = .array~new; do j = 1 to 10000; a~append(j); end; end`), pinned `perf stat`,
   4 runs each: instructions base 18.029 G, head 18.030 G; cycles base 3.00 to 3.03 G, pad48 3.02 to
   3.04 G, head 4.09 to 4.12 G. Zero collections on both, no charged bytes, so the task's logic is
   not executed; `perf record` puts 61.6% (base) and 70.4% (head) in
   `rexx_exec::dispatch::collection::occupied`, a function this diff does not touch, whose symbol
   moved from `0x1e3b80` (0 mod 64) to `0x1e3c20` (32 mod 64). Layout, not logic; wall clock
   interleaved 3 runs at 100 arrays: base 4.96 to 5.01 s, head 5.61 to 6.52 s. Not in the gate's
   program set and within no budget's reach, but the task's perf evidence (instruction counts plus a
   five-program wall clock) is blind to it. Separately and older than this task, `occupied` building
   a Vec of every slot offset per `append` (with a `realloc`/`memmove` per call) makes `append`
   quadratic: about 90 000 instructions per append at 10 000 items.
2. **Stale doc on `stress_collect`.** `rust/crates/rexx-exec/src/lib.rs:1462`-`:1470`: "`alloc_with`
   reads it once, in an `if`" is now false; `collect_if_due` reads `collect_due`, never
   `stress_collect`, and `collect_now` reads `stress_collect`. Likewise `lib.rs:1503`-`:1504` "half of
   this crate's trigger policy. The other half is `Heap::will_grow`" no longer names the whole
   policy. The prose rule deletes a false sentence.
3. **Concern 3, `Outcome::peak_body_bytes` (`lib.rs:322`).** The only reader is the in-crate test
   `tests.rs:1045`. A `#[cfg(test)]` field (or a `#[cfg(test)]` accessor on the run) would serve it:
   integration tests link the non-test library, so their four literal constructors would not need
   the field at all. It sits beside `collections`, a field of the same kind, so it is not a blocker;
   a cfg(test) field removes the outward interface the brief rules out and the four constructor edits.
4. **The short-string test cannot see the inline exemption it describes.**
   `tests.rs:1065`-`:1069`: its strings are 21 to 26 bytes, so with `charge_text`'s
   `len > INLINE_BYTES` test removed they would be charged, but 200 000 x 26 bytes is 5.2 MB, under
   the 32 MiB floor, so the pinned `collections == 6` holds either way. It witnesses that the slot
   cadence is unchanged, not that Bytes-inline strings are exempt. An exact collection-count pin is
   also brittle to any allocation change elsewhere; `>= 5 && <= 7`-style or a ratio would survive
   unrelated work.
5. **`loop999`'s wall-clock cost is in the report but not in the gate record**
   (`docs/superpowers/plans/phase-6-1-gate.md` `## Task 5a`). It is the one measured cost of the
   change and the record is where the project looks; add the interleaved figures and that
   instructions are flat.

Aside, older than this task: `uninit` prints `done 0` on both base and head where the oracle prints
`done 990`, and stays at 312 MB where the oracle is at 19 MB. Resurrected objects wait in
`uninit_ready` until `GC('force')` or termination (`lib.rs` `collect_now` comment), so a loop of
finalizable objects holding large strings grows with them. The output difference is GC timing
(licensed); the memory growth is not bounded by this task's rule because the bodies are live.

### Focus 4 and 5

- Concern 3: Minor 3.
- Concern 4 (mark-loop sum about 0.1%): accepted, within budget.
- Perf claims: the gate record quotes the callgrind and wallclock commands, the base binary, its
  sha256 and the pad48 control; callgrind maxima +0.2035% (rexxcps), inside +0.5%. Not re-run.

### Assessment

**Task quality:** Needs fixes

**Reasoning:** The byte trigger is correct, periodic, drift-free, cheap on the fast path, and bounds
every string-producing shape probed (concat, PARSE, stems, arrays, REPLY and started activities)
with no premature collection; but `~copy` of a string is a Text-body path left uncharged and still
grows at about 4 GB/s, which is the defect class the task is for and a one-line fix with a test.
Findings: Critical 0, Important 1, Minor 5.
