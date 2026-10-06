# Task 23 fix round 2, re-review (base 4db3412ee, head cce32934b)

Probes ran in a `git archive` of `cce32934b` (`rust/`, `interpreter/`, `extensions/`) under
`/tmp/claude-1000/p6-t23-rr2/`, own target dirs, `memcap 8G`, deleted after.

**Verdict: CHANGES NEEDED.** F1 and N1 are fixed. C2 is fixed for the collector, but one more walk
of the defect class counts as touches: `detach_exposed_tails` rewrites exposer stems that are
garbage (N3 below). The fix is small either way.

## Items

| # | Item | Verdict | Evidence |
|---|---|---|---|
| 1 | C2 untagged walks | PARTIAL | The collector is fixed. At head the witness `a_collection_in_another_activity_shares_nothing_it_prunes` passes in release and in debug. Mutations of `collect_now` in the scratch copy, release, witness only: whole-body pause removed (prunes still `peek`) passes; prunes back to `heap.get` (pause kept) passes; both removed fails with `program: SharingCount { objects: 111, shared: 102 }`. So either mechanism alone holds the witness, and only removing both is witnessed (see the note below). The enumeration re-runs byte-identical: `python3 .../tagged-sites.py \| diff - tagged-sites.txt` and the same for `debug-checks.py`, both empty at head. Not fixed: the exposer-table walk in `detach_exposed_tails`, N3. |
| 2 | F1 populations | FIXED | Gate and `sharing-groups.md` header both say each group runs until its end or its first refusal or deadline. The quoted counts re-derive at head with the header's command: `refused at` 76, `refused at the run exceeded its deadline` 1, `did not finish` 0, `no test ran` 36, `pass, ` 248, `failure, ` 17, `error, ` 11; `grep -c '^\| [^\|]*testGroup \| ' sharing-groups.md` gives 388, and the five disjoint outcomes sum to it. |
| 3 | N1 every-group test | FIXED | `#[ignore = "most of 8 GB; run alone with RAYON_NUM_THREADS=4 and --ignored"]` at `concurrency_tests.rs:749`; the gate and the record both quote the command with `--exact ... --ignored`. |

## New findings

**N3, Minor: a stem clear by one activity counts another activity's unswept exposer stems as
shared.** `detach_exposed_tails` (`stem.rs:562`) walks `stem_exposers[home]`; `weak_target` is
now `peek`, but the filter reads each exposer with `heap.get` (`:570`) and the rewrite takes each
with `heap.get_mut` (`:604`). An exposer is the local stem of a `PROCEDURE EXPOSE h.1` call; after
the call returns it is garbage, but its weak cell still reaches it until a collection. The
sharing-walks README classifies these reads as resolutions ("reads and writes of the stems that
expose a cleared stem's tails"), but the program cannot reach those stems. Probe (scratch tests in
`concurrency_tests.rs`, `--release --features sharing`; debug gives the same figures): main does
`h. = 0`, then `call p` 100 times with `p: procedure expose h.1; h.1 = h.1 + 1`, then
`.t~new~start('clear', h.)~result`, where `clear` does `use arg s.; s.~empty`:

| variant | program objects | program shared |
|---|---|---|
| started activity clears | 212 | **103** |
| main clears (`h.~empty`), started activity only receives `h.` | 212 | 3 |
| `call GC 'Force'` in main before the start, started activity clears | 211 | 3 |
| no `procedure expose`, started activity clears | 11 | 3 |

The GC-first row shows that the 100 are the dead exposer stems. Changing only the filter's read
(`:570`) to `peek` leaves the figure at 103, so the rewrite's `get_mut` counts them by itself.
Fix, one of: (a) do the rewrite through an untagged mutable accessor (a `peek_mut` beside
`peek`), and add the probe as a witness (started-activity clear, program shared below 50); this
also leaves a live exposer's rewrite uncounted, which then needs saying in the README; or (b) keep
counting and change the README's "Resolutions: counted" sentence to say that a clear counts every
stem the exposer table still reaches, garbage included until a collection.

**N4, Nit: `debug-checks.txt` has one row that is not a debug-only check.** `ir.rs:646` is a
`#[cfg(debug_assertions)]` on the struct field `settings`. `debug-checks.py` takes the next `{`
after the attribute as the block, which is the start of `impl Chunk`, and lists that impl's calls.
The `-` classification is right (no heap reads), so no figure changes.

**Note (not a finding): the collector's two mechanisms are redundant under the witness.** Either
the whole-body pause or the three `peek`s alone keeps the witness green (item 1). Each is right on
its own terms. A regression that drops one of them is not caught.

## Debug-only checks classified by callee name: bodies read

Read for every `-` row whose callees could reach the heap: `ffi.rs:914` (`CALLING` thread-local),
`ffi.rs:6328` (a `#[cfg(debug_assertions)] #[test]` in a test module), `activation.rs:1073`,
`dispatch/array.rs:380` (a slot slice), `eval.rs:277`, `:1104`, `:1108` (`Number::parse_bytes` on
bytes already in hand), `guards.rs:490` (guard table, activation ids), `install.rs:1491`,
`ir/drive.rs:3582`, `:3941` and `run.rs:3706` (instruction slices), `lib.rs:1075` (plan name map),
`run.rs:434` (plan cache), `run.rs:2488` and `run/loops.rs:1948`, `:2038`, `:2286` (`shape_of` on a
symbol name), `run/loops.rs:708` (`round_via_unary_plus` on a `Number`), `scheduler.rs:752`
(`idle_read_state`: handles, queue length, pin depth, activation ids, frame count), `stem.rs:134`
(`bound_slot_of`, the plan), `value.rs:209` (`inline_text` on bytes), `dispatch.rs:1636`. None
resolves a heap object. `/bin/grep -rn debug_assertions` over the four crates' `src` finds no other
form than `#[cfg(debug_assertions)]` but `rexx-core/src/frame.rs:513`, a `not(debug_assertions)`.

## Checked and holding

- `collect_now`: `sharing_pause(true)` first, `sharing_pause(false)` last; no `return` or `?` in
  between. No Rexx runs inside it (UNINITs are only queued in `uninit_ready`). A panic does not
  resume the interpreter: the pool path re-raises (`library.rs:1146`) or posts the panic for the
  baton's holder to panic with (`pool.rs:39-49`), so the pause depth cannot leak into a continuing
  run. `heap.collect(` has one site (`lib.rs:2870`).
- `Heap::peek` (`heap.rs:476`) is `get` without the `resolved` call. Feature off, `get` and `peek`
  are the same code.
- `unshared!` (`lib.rs:34-43`): feature off it is `{ let answer = $body; answer }`. No call site
  has `?` or `return` inside `$body`; `route.rs` applies `?` to the macro's value. Feature-off
  `.text`: release `rexx-run` built without features from `4db3412ee` and from `cce32934b` at one
  path, `objcopy -O binary --only-section=.text` then `sha256sum`: both
  `7c00da6895962f58e3a7b416c97efaa8430b6a1a05cc39b787de424b8b475d6f`, 3130350 bytes, the value
  `sharing-off-text-hash.txt` records for `fix2`.
- Other walks read and not of the class: `with_idle_activity` (`scheduler.rs:743`) runs a context
  read as the reader's activity, a real cross-activity read; `message_completed`, `switch_to`,
  `swap_running` read no heap object; `rexx_package_class_uncached` reads tables, not objects.
- No tagged accessor is reachable through a `&Heap` under another name: `/bin/grep -rnE
  '(\w+): &(mut )?...Heap\b'` over `rust/crates` finds only `heap:` parameters, which the
  enumeration's pattern matches; outside the three scanned crates the only calls are `rexx-core`
  tests and benches.
- At head: `cargo fmt --check` 0; `cargo clippy --workspace --all-targets -- -D warnings` and with
  `--features rexx-exec/sharing,rexx-exec/pinning`, both finished without a warning; the sharing
  witnesses, excluding the two population tests, pass in release and debug.
- Prose: no em-dashes in added lines (`git diff 4db3412ee cce32934b | grep '^+' | grep -c '—'`:
  0); the gate's outcome counts sit beside their command; nothing forward-looking found.
