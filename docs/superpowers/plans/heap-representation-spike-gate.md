# Heap-representation spike — the gating measurement

Measured 2026-09-09 at `2c63ec66e`, `rustc 1.98.1`, release build of
`rexx-run`, `perf record` on `bench-programs/`.

The spike exists to re-examine D1 (`2026-07-27-rust-rewrite.md`:125), settled
at Phase 1 as *arena + tagged index handles*, now that four phases of real
interpreter exist. D1's own note says the cost of being wrong is "total", so
the first question is not which representation is better but **whether heap
representation is where the time is at all**.

It is not, on ordinary programs.

## Reading

Self time, sampled cycles, `--percent-limit 0`:

| program | SipHash | heap (`collect_now` + `alloc_with`) |
| --- | --- | --- |
| `alloc.rex` | **22.25%** | 0.56% |
| `dispatch.rex` | **14.29%** | 0.00% |
| `varlookup.rex` | 0.00% | 0.00% |
| `strings.rex` | 0.01% | 1.34% |
| `compound.rex` | 0.00% | 0.00% |
| `heapshape.rex` | 3.14% | **11.83%** |

`alloc.rex` re-recorded twice at 997 Hz with no call graph, a different mode
from the 499 Hz dwarf run above: **24.60%** and **24.48%**, with cycle counts
agreeing to 0.01% (9.374e9, 9.375e9). The figure reproduces across recording
modes.

`heapshape.rex` is the counterweight and belongs in the table: it is built to
be a full-GC pause over a ~1M-object graph, and there the collector really is
11.83%. That is the shape of program the collector costs anything on.

## What this does to the columnar layout

The A/B in `records/` measured a hand-rolled columnar arena at **0.50x on
sweep** and **1.06-1.08x on point lookup**, giving a break-even of
`T_sweep > 0.12 * T_get`. Against the table above, sweep is at most 1.34% on
an ordinary program, so the whole available gain is under 0.7% — against a
cost on `Heap::get`, which runs on every value access. **Not worth doing**,
and this measurement is why. Recorded so it is not re-proposed.

## The one thing this does NOT measure

**`Heap::get` has no symbol in the profile because it is inlined**, so its
cost is distributed into its callers and does not appear as a row. Nothing
here says `Heap::get` is cheap — only that the *collector* is. A claim about
handle-resolution cost needs its own instrument, and this is not it.

## Where the time actually is

SipHash — `std::hash::random::RandomState`, the default hasher — on three key
types, on the two message-send-heavy programs. `NameHasher =
rustc_hash::FxBuildHasher` already exists in-tree (`rexx-core/src/lib.rs:64`)
and these maps simply do not use it.

| key | share on `alloc.rex` | map |
| --- | --- | --- |
| `MethodId` | 8.71% | `dispatch.rs:1194` `natives: HashMap<MethodId, NativeEntry>`, read at `:2603` on every send resolving to a native |
| `&[u8]` | 5.95% | not attributed to a site by measurement — see below |
| `ProgramId` | 4.30% | the `ProgramId`-keyed package tables, `lib.rs:3135`/`:3240`/`:3248` and `environment.rs` `package_public_classes` |
| `sip::Hasher::write` | 3.29% | the shared hashing body |

`MethodId` is `pub struct MethodId(pub u32)` (`body.rs:441`) — a dense small
integer being SipHashed.

Alongside, on the same profile: `from_utf8_lossy` 1.79% + `Utf8Chunks::next`
1.68%, and `rexx_package_class` itself 4.48%.

## What is a code reading and not a measurement

**Caller attribution failed.** The dwarf run produced no user-space
callchains — `perf script` resolves no symbol for any of the 1629 samples,
while `perf report` resolves all of them — so the shares above are leaf
attribution only. Reading `environment.rs:768` shows `rexx_package_class`
doing, per library program, one `ProgramId` lookup and one `&[u8]` lookup,
then `String::from_utf8_lossy` on the fallback path before
`classes().lookup`. That is consistent with the leaf shares and is **the
obvious candidate for the unattributed `&[u8]` row**, but it is a reading of
the code, not a measurement of the callers. Attributing it needs working
callchains.

## Recommendation

Do not spend the heap spike on layout. The measurement it was to gate on says
the target is under 1% on every program except one built to be a GC
benchmark. Take the hashing instead: it is 14-25% on send-heavy programs, the
replacement hasher is already in the tree, and `MethodId` is a `u32` that
could index a `Vec` rather than hash at all.

`Heap::get`'s own cost remains open and is the only part of the heap question
this measurement leaves standing.
