# Heapshape round 1: review

Reviewer, 2026-10-09. Range `ba3aae4fe..118d78b5e` (code `5214a2089`, report and record `118d78b5e`;
`d1e54f178` is a controller edit and was not reviewed). `git diff 5214a2089 118d78b5e -- rust` is empty.
Source read from `git archive 118d78b5e`. A debug `rexx-run` was built from that archive in
`/tmp/claude-1000/p61/hsrv/` (`Compiling rexx-exec` present, own target dir, `-j 4`) for the probes below,
which are kept in `/tmp/claude-1000/p61/hsrv/p1/`.

### Spec Compliance

* **Change.** Live body bytes are a running figure (`Heap::held_bytes`): held at every R10 charge through
  `Interp::charge_body_bytes` (`lib.rs:2739`), held by `Heap::alloc` and `Heap::alloc_immortal`, released by
  the sweep (`rexx-core/src/heap.rs:364`, `release_body_bytes(freed_bytes)`) and by the new shrink sites.
  `bytes_since` and the trigger expression are untouched. Matches the brief.
* **Enumeration named.** The report lists its grep commands and dispositions. My independent pass (every
  `alloc_with`/`alloc_charged`/`alloc_immortal`/`heap.alloc` call, every `heap.get_mut`/`peek_mut` site that
  binds `Body::Array`, `Body::Text` or `native`, every `BufferState` capacity write, every `*native =`,
  `*bytes =`, `*slots =`, `.body =`) agrees with it on which statements change held bytes. It finds one class
  the greps cannot see: error exits between a growth and its charge (Important 1).
* **Debug check.** Present, with a message naming both figures, inverted once and recorded
  (`hsr/logs/inversion-debug.txt`: `the survivors hold 2010209 body bytes, the running figure says 42010208`).
* **Tests.** R10 tests green with bodies unchanged; one R10 test's doc comment was reworded because the
  change made a sentence false (disclosed, acceptable). New test
  `bytes_a_body_stops_holding_leave_the_live_figure` pins 12 collections, passes against the base, and
  fails the inversion in release (`left: 8, right: 12`, `hsr/logs/inversion-release.txt`).
* **Perf.** `callgrind.sh -r 2` over the eight programs, columns base61/head/r1 as briefed. I re-read
  `hsr/cg1/table.txt` and the `.row` files: heapshape +0.0525% and rexxcps +0.4409% recompute from the rows,
  all 48 `.rc` are 0, spreads at most 0.0001%. sha256 of base61 matches `## Task 1`; head and r1 sha256 match
  the record. r1 was built from an archive (`build-new.log`, `Compiling rexx-exec` in `hsr/new-src`).
  Wall clock interleaved as the constraint says. Target met on callgrind.
* **Per-task check.** Logs exist and say what the report says (`test3.log` 147 `ok` result lines and no
  `FAILED`; corpus 29+1 ignored; ir_recorded_oracle 21; seeded gate 1 passed in 124.61 s).
* **Record.** `### Heapshape round 1` under `## Task 9` of the gate record, with commands and binaries.

### Strengths

* The debug check is the right net: it compares against the survivor sum at every debug collection, so any
  missed path that a debug test reaches before a collection panics with both figures.
* The first two debug runs found real misses (immortal literals, crate tests through `Heap::alloc`), and the
  report says so and says how each was closed.
* The new test is judged against the base, not against itself, and its release inversion shows it can see a
  missed release where the assertion is compiled out.
* `array_reshape` now charges growth and releases shrink separately instead of charging `saturating_sub`
  only, which was a latent under-release in R10 itself.
* Perf attribution to `Interp::collect_now` per freed body is stated, with the consequence (cost now scales
  with garbage) named as a concern rather than hidden.

### Issues

#### Critical

None.

#### Important

1. **A buffer grown on a MutableBuffer error exit is never held.** `rust/crates/rexx-exec/src/dispatch/buffer.rs:1642`
   to `:1646` (`INSERT`), and the same shape at `:1664`-`:1668` (`OVERLAY`), `:1689`-`:1695` (`REPLACEAT`,
   two fallible steps), `:1737`-`:1743` (`CHANGESTR`), `:1769`-`:1781` (`CASELESSCHANGESTR`):
   `buffer_capacity` raises `state.capacity`, then `insert_bytes` / `out.try_reserve` / `replace_at_bytes` /
   `changestr_bytes` can return `Err` through `?`, and `interp.charge_growth(grown)` is never reached. The
   enumeration greps find the growth and the charge in the same function and so count the site as covered.
   Reproduced on the debug build of `118d78b5e` (`hsrv/p1/oom.rex`, `b~insert('z', 450000000)` under
   `signal on syntax`, then a slot churn):

   ```
   ( ulimit -v 1400000; memcap 2G timeout 120 .../hsrv/target/debug/rexx-run oom.rex )
   assertion `left == right` failed: the survivors hold 450010623 body bytes, the running figure says 10878
   ```

   (`ulimit -v 1100000` fails the first reservation, capacity stays 256 and the run is clean;
   `1700000` succeeds.) Why it matters: the missed charge is R10's own, but under R10 the next collection's
   survivor sum corrected `live_bytes`. With a running figure it is permanent: the figure stays 450 MB low for
   the buffer's life, and when the sweep frees it `release_body_bytes` saturates, so in release the figure
   stays low for the rest of the run (more collections than R10's condition gives). In debug a trappable
   Rexx condition becomes a panic. `ulimit -v` is the oracle-run harness's own setting, so this is not only a
   theoretical OOM. Fix: charge the growth before any later fallible step (end the `state` borrow after
   `buffer_capacity`, charge, re-borrow), or reserve `out` before growing the buffer; add a crate test that
   reaches one of these exits followed by a collection, and add the shape (a `?` between a capacity change
   and its charge) to the report's enumeration.

#### Minor

1. **`get_mut` whole-body replacement: not reachable today, not prevented.** At `118d78b5e` the only
   `.body =` in non-test source is `rexx-core/src/heap.rs:263` (a cleared `WeakRef` becomes `WeakRef(NIL)`,
   both hold 0), and no `*body =`, `*object =` or `mem::replace` of a body exists in `rexx-exec`, `rexx-core` or
   `rexx-api`. So the `collect.rs` `reference_cycles_are_collected` edit is a test artefact, not a hole.
   `Heap::get_mut` and `peek_mut` still hand out `&mut Object` with no stated contract. Fix: one sentence on
   `get_mut` saying a caller that changes what the body holds outside its slot records it with
   `hold`/`release`/`rehold_body_bytes`.
2. **Most release sites are unwitnessed.** The assertion runs on every `Heap::collect` in debug
   (`#[cfg(debug_assertions)] assert_eq!` after the sweep, unconditional), and the debug workspace suite
   passed with it live. But only `setBufferSize`'s release is shown to be reached before a collection (the
   inversion). `finish_string`, the `array_reshape` shrink, `stream_init`/`stream_uninit` against a buffer,
   and the `writable` failure have no witness. My probes could not reach two of them:
   `a~empty` then `a[2,2]=` refuses with 93.926 (no reshape), and `self~run(.stream~method('INIT'), ...)` on a
   `MutableBuffer` subclass fails with 97.1 on `!C_STREAM_INIT`. That is not a claim they are unreachable.
   Fix: none required for this gate; the report's dispositions for these sites rest on reading.
3. **Wall clock on pingsem and pingguard is outside the noise band against base61** (`wall3`: pingsem
   +8.26%, pingguard +4.20%), and the record's verdict column reads `inside` from callgrind alone. r1 and
   head execute the same instructions there, so the work this round added is not the cause, and the report
   flags it as Concern 2. Fix: the record should say the wall-clock verdict is unresolved for these two, so
   the Task 12 cumulative check does not inherit `inside`.
4. **rexxcps has 0.06% left under the budget**, and the per-freed-body read grows with garbage. Within
   budget; noted so Tasks 10 to 12 plan against it.

### Assessment

The round does what the brief asks, and its perf numbers reproduce from its own outputs. One missed path
is reproduced with a probe: buffer growth on the five MutableBuffer error exits is never held. Under R10 that
miss healed at the next collection. With the running figure it is permanent in release and a panic in
debug.

**Quality:** Needs fixes
