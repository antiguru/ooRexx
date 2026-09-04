# Task 1 report — where `MutableBuffer`'s bytes live, decided by measurement

BASE: `fdf4c6624` (`5c follow-up Task 0: the method-body table reads MutableBuffer through a buffer
that has contents`). `$B` = `/home/moritz/dev/repos/claude-build-scratch/5c-followup-task1`; three
`git archive` extracts of BASE live under it — `$B/tree` (candidate A, `CARGO_TARGET_DIR=$B/target`),
`$B/tree-B` (candidate B, `$B/target-B`) and `$B/tree-size` (width probes, `$B/target-size`) — see
section 6. Line numbers cite BASE unless a tree is named.

Every claim below is tagged **measured** (with the command that printed it) or **inferred** (a
reading of code or a deduction, not a run). The worktree at `ooRexx-rust-rewrite` was only read.

## 1. Where the byte machinery already is

**Inferred from reading** `rust/crates/rexx-exec/src/builtin/string.rs` (2071 lines),
`builtin/word.rs` (1035 lines) and the helpers in `builtin.rs:860`-`:1160` in the extract. Every
function below has the signature `(interp: &mut Interp, name: &[u8], args: Args<'_>) ->
Result<ObjRef, Failure>`. `Args` (`builtin.rs:876`) is a pair of `&[Option<ObjRef>]` slices — the
converted values and the original objects — and a builtin reaches its subject through one of two
helpers:

* `required_string(interp, args, n) -> Vec<u8>` (`builtin.rs:973`): `interp.to_text(value)
  .into_owned()` — **copies the subject's bytes out** so the `&mut interp` borrow ends.
* `required_render(interp, args, n) -> Rendered` then `.text(interp) -> &[u8]`
  (`builtin.rs:997`, `value.rs:1044`): borrows the subject's bytes in place, at the cost of every
  `&mut` call (the numeric-argument conversions) having to come first.

The result is then built in a `Vec<u8>` — via `buffer(interp, len)` (a lent, pooled buffer with
fallible reservation, `builtin.rs:1146`) or a fresh `Vec` — and handed to `interp.text_built(out)`
or `interp.text(&[u8])` to become a `Body::Text`. Counts go through `interp.counted(usize)`.

### 1.1 Functions read, and the shape each has

Three shapes appear. **None is "a function from `(&[u8], parsed args)` to bytes"** — the subject
read, the byte operation and the result allocation are interleaved in one body — but the byte
operation in every function is written against a `&[u8]`/`Vec<u8>` and the argument parsing is
already separate from it, so the split is a mechanical extraction rather than a rewrite.

| function | file:line | subject read | byte-level core already factored? |
|---|---|---|---|
| `length` | string.rs:94 | `interp.text_len(value)` | n/a — one call |
| `substr` | string.rs:449 | `required_render` (borrow) | inline slice arithmetic; `push_pad` helper |
| `delstr` | string.rs:479 | `required_string` (copy) | inline |
| `insert` | string.rs:517 | `required_string` ×2 (copy) | inline; `push_pad` |
| `overlay` | string.rs:563 | `required_string` ×2 (copy) | inline; `push_pad` |
| `pos` | string.rs:622 | `required_render` ×2 (borrow) | **yes**: `find_forward(&[u8], &[u8], start, range) -> usize` (`:187`) |
| `lastpos` | string.rs:655 | `required_string` ×2 (copy) | **yes**: `find_backward(&[u8], &[u8], start, range)` (`:325`) |
| `reverse` | string.rs:674 | `required_string` (copy) | `Vec::reverse` |
| `strip` | string.rs:690 | `required_string` (copy) | inline over `&[u8]`; `in_set` (`:362`) |
| `space` | string.rs:714 | `required_string` (copy) | **yes** for the scan: `word::word_slices(&[u8]) -> Vec<&[u8]>` (`word.rs:167`); rejoin inline |
| `countstr` | string.rs:823 | `required_string` ×2 (copy) | **yes**: `count_occurrences(&[u8], &[u8], limit)` (`:344`) |
| `changestr` | string.rs:835 | `required_render` ×3 (borrow) | inline loop over `find_forward` |
| `translate` | string.rs:934 | `required_string` (copy) | inline byte loop; no-table form shares `case_shifted` |
| `verify` | string.rs:995 | `required_string` ×2 (copy) | inline over `&[u8]`; `in_set` |
| `lower` / `upper` | string.rs:1031 / :1039 | `required_string` (copy) | **yes**: `case_shifted(interp, &[u8], start, range, shift) -> Result<ObjRef,_>` (`:1058`) — bytes in, but it allocates the `ObjRef` itself |
| `words` | word.rs:181 | `required_string` (copy) | **yes**: `Words<'a>` scanner over `&[u8]` (`:103`) |
| `word` | word.rs:211 | `required_render` (borrow) | `Words` |
| `wordindex` / `wordlength` | word.rs:240 / :258 | `required_string` (copy) | `Words` |
| `subword` | word.rs:294 | `required_string` (copy) | `Words` |
| `delword` | word.rs:333 | `required_string` (copy) | `Words` |
| `wordpos` | word.rs:376 | `required_string` ×2 (copy) | `word_slices` ×2, inline compare |

Also read, same `required_string` + inline shape, none relevant to a buffer method: `copies`
(string.rs:745), `abbrev` (`:770`), `compare` (`:794`). Not read in detail: `center`, `left`,
`right` (`:382`-`:448`); their signatures were listed, their bodies were not opened.

**The argument layer cannot be shared, and that is measured, not inferred.** The builtin layer
raises 40.x at rc 216 naming the routine; the method layer raises 93.9xx at rc 163. Oracle, from a
fresh directory (`$B/scratch/oracle-argerr/`, files `m1.rex`, `f1.rex`, three descriptors):

```text
buf = .MutableBuffer~new('abc'); say buf~substr('x')
  stdout: (empty)   rc 163
  stderr: Error 93.924:  Invalid position argument specified; found "x".
say substr('abc','x')
  stdout: (empty)   rc 216
  stderr: Error 40.12:  SUBSTR argument 2 must be a whole number; found "x".
```

The method-layer converters already exist in `dispatch.rs`: `whole_method_argument` (`:7946`),
`refuse_method_argument` (`:7966`), `usize_or_refuse`, `optional_length_argument` (`:7493`),
`required_string_argument` (`:3904`), each taking a `fn(&[u8]) -> Raised` for the 93.9xx family.
`whole_method_argument`'s own doc carries the oracle measurements that separate this family from
the builtin one (`'abcdef'~upper('0.0')` reports `found "0.0"` where `substr('abc','0.0')` reports
the converted `found "0"`), so the pattern for a method-layer `substr`/`pos`/... is in the tree.

### 1.2 Verdict: do they factor over `&[u8]`?

**Partly today, cheaply tomorrow.** The search and word primitives already do — `find_forward`,
`find_backward`, `count_occurrences`, `in_set`, `Words`, `word_slices` are `&[u8]` in, plain values
out, and a `MutableBuffer` method can call them on the buffer's bytes with no `Text` materialised.
The editing operations (`substr`, `delstr`, `insert`, `overlay`, `changestr`, `translate`, `space`,
`delword`, `case_shifted`) are written against slices but build their result into a `Vec<u8>` and
hand it to `interp.text_built` inside the same function. For each, the body between "arguments
converted" and "`text_built`" is a pure `fn(&[u8], usize, usize, ...) -> Vec<u8>` (or `-> ()` over
a `&mut Vec<u8>`) waiting to be named; nothing in those bodies touches `interp` except the final
allocation and, in `changestr`/`insert`/`overlay`/`space`, the fallible `buffer(interp, len)`
reservation, which a buffer method would replace with `try_reserve` on its own `Vec`.

So the phase gets smaller in the way the brief hoped, with one caveat: **the sharing is of the byte
cores, not of the functions.** Each `MutableBuffer` method still needs its own argument-conversion
prologue in the 93.9xx family (measured above) and its own result handling (mutators return the
receiver, `MutableBufferClass.cpp:323`-`:353`; readers return a fresh `Text` or a count). What it
does not need is a second implementation of any string algorithm, provided Task 3 first extracts
the cores from the builtins listed above. The extraction is a refactor the builtin tests
(`string.rs`/`word.rs` `mod tests`, which go through `dispatch`) already cover.

**Inferred**: a buffer method that materialised a `Text` per call instead would cost one
`Bytes`/`Vec` copy of the whole buffer per read — the same O(n)-per-call shape candidate A's
mutation path has (section 3), so "just wrap the buffer as a string and call the builtin" is not a
shortcut this report can recommend for the readers either.

## 2. The representation — two candidates built far enough to measure

### 2.1 The width constraint in `rexx-core/src/body.rs`

**Inferred from reading** `body.rs:588`-`:602` and `bytes.rs:36`-`:62`. `const _: () =
assert!(size_of::<Body>() <= 80);` guards the arena: `Slot` holds one `Body` per object, so a
variant wider than 80 bytes widens every object in the heap including the ones that never take that
variant. `Body::Stem` sets the 80 (`Box<[u8]>` 16 + `Option<ObjRef>` + `NameMap`), `Body::Text`'s
`Bytes` has `INLINE_BYTES = 54` chosen as the largest inline capacity that stays under it, and the
module doc says outright that "Phase 5 adds variants and is expected to trip this ... widening the
arena for a cold value kind should be a deliberate act with a boxed alternative weighed". Both
`Body::Native(Box<NativeObject>)` (`:208`) and `Body::VarRef(Box<VarRef>)` (`:218`) took the boxed
alternative and say so in their docs.

What I inferred from this before measuring — that an inline `Vec<u8>` + two `usize` + `ObjRef` +
`BehaviourHandle` would be wider than `Stem` — **was wrong, and 2.5 measures it**: that payload is
24 + 16 + 8 + 8 = 56 bytes and fits under the 80 with room for the tag. Candidate B was built boxed
(`Box<BufferBody>`, one pointer, on `Native`'s precedent) before that was measured; the doc comment
on the variant in `$B/patches/candidate-B.patch` repeats the wrong inference and is left as built,
since the tree was not to be edited after its binary was measured. Candidate A touches `Body` not
at all. The unboxed shape is a live option and 2.5 and section 4 weigh it.

### 2.2 The third shape (`NativeObject::entries`), and why it is not built

`NativeObject::entries` is `HashMap<Box<[u8]>, ObjRef>` (`body.rs:509`): a name-to-object table
whose values are handles, so the buffer's bytes would still have to be a `Body::Text` reached
through a hash lookup — it is candidate A's shape with a `HashMap` where A has a three-entry `Vec`,
and a `Body::Native` object does not dispatch as an instance of a user-visible class
(`receiver_kind`, `dispatch.rs:1573` onward, special-cases the `Package` native by class identity
and the `Method`/`Routine` ones after it). Not built.

### 2.3 Candidate A — object variables in `Body::Instance`'s `pools`

Patch: `$B/patches/candidate-A.patch` (one file, `rexx-exec/src/dispatch.rs`, 165 diff lines).
Tree: `$B/tree`. No `rexx-core` change.

* The constructor keeps `new_instance` and then binds three names in the instance's pool under
  the class's scope: `!BUFFER_TEXT` (a `Body::Text` handle), `!BUFFER_CAPACITY` and
  `!BUFFER_DEFAULT` (tagged small integers via `Interp::counted`). **The patch's comment says the
  leading `!` keeps a subclass's `EXPOSE` off these names, and that is wrong, measured**
  (`$B/scratch/oracle-bang/`): `!x = 5; say !x` prints `5` on the oracle, so `!` is an ordinary
  symbol character; and `alias.rex` — a `MutableBuffer` subclass whose method does `expose
  !buffer_text; return !buffer_text` — prints **`abc` under A2 on both engines** where the oracle
  prints `!BUFFER_TEXT` (an unset variable's own name), rc 0 on both: a silent divergence, and a
  program that can read and overwrite the buffer's state through an ordinary variable. A production
  A would need a scope no class can name (a reserved `ObjRef`), not a reserved spelling; the
  prototype is left as built and measured. Capacity arithmetic is
  `newRexx`'s (`MutableBufferClass.cpp:100`-`:122`): `defaultSize` is the second argument or
  256, capacity is `max(defaultSize, len(initial))`.
* `~length`: `heap.get(receiver)` → `Body::Instance { class, pools }` → `pools.get(class,
  "!BUFFER_TEXT")` → `interp.text_len(text)`.
* `~append`: read the text handle, copy its bytes into the lent result buffer (`take_result_buffer`,
  the same buffer the builtins build in), append each argument, apply `ensureCapacity`'s
  `max(needed, 2*capacity)` to the stored capacity, `text_built` the new string, `set_pool_variable`
  twice. **Every append allocates a new `Body::Text` and copies the whole contents**; the old
  `Text` is garbage until the next collection.
* Write access goes through `Interp::set_pool_variable(owner, scope, name, value)` (`lib.rs:6879`),
  which is the `::ATTRIBUTE` setter's path and is private to the crate root — callable from
  `dispatch.rs` as a child module without any visibility change.

**Process note, so the binaries are accounted for.** The first A build (`$B/bin/rexx-run-A`, sha256
`5b793469e93e3b0dcaff506a4c642ff168f5045231c19fcaa96b97ca0150a2a4`) had the two rows appended to
`NATIVE_CLASS_METHODS` (the class-side table `NEW` lives in, ending at `dispatch.rs:710`) instead of
`NATIVE_METHODS` (`:246`), and panicked at model build on both engines: `NATIVE_CLASS_METHODS names
MutableBuffer~LENGTH, which that class's class behaviour does not answer` (`$B/scratch/probes/
*.A.err`, rc 101). The registry check did what it is for. The rows were moved beside `("String",
"LENGTH", …)` at `:550` and rebuilt as `rexx-run-A2`, which is the binary every A figure below
comes from. `rexx-run-A` is kept, not deleted, and is not used anywhere below.

### 2.4 Candidate B — a new boxed `Body` variant

Patch: `$B/patches/candidate-B.patch` (six files: `rexx-core/src/body.rs`, `rexx-core/src/lib.rs`,
`rexx-exec/src/dispatch.rs`, `value.rs`, `stem.rs`, `run.rs`; 277 diff lines). Tree: `$B/tree-B`.

* `rexx-core`: `Body::Buffer(Box<BufferBody>)` with `pub struct BufferBody { class: ObjRef,
  behaviour: BehaviourHandle, bytes: Vec<u8>, capacity: usize, default_size: usize }`, exported;
  `Body::trace` gains `Body::Buffer(buffer) => out.push(buffer.class)` (the class handle, as the
  `Native` arm traces its own; the bytes are plain data).
* The constructor does `new_instance`'s four steps (abstract check, behaviour, rooting, `UNINIT`
  registration) with the buffer as the body, and `try_reserve_exact(capacity)` so the `Vec`'s
  reservation tracks the observable capacity from the start.
* `~length`: `heap.get(receiver)` → `Body::Buffer(buffer)` → `buffer.bytes.len()`.
* `~append`: each argument's bytes are copied into the lent result buffer (so the heap can then be
  borrowed mutably), then `heap.get_mut(receiver)` → `&mut buffer.bytes`, `ensureCapacity`'s rule
  on `buffer.capacity`, `try_reserve_exact` up to the new capacity, `extend_from_slice`. No object
  is allocated.
* **What the compiler demanded, measured by building** (`$B/scratch/build-B.err`): exactly one
  exhaustive match outside `rexx-core` failed to compile — `run.rs:4034`
  (`forward_arguments_conversion`), where a buffer joins the `Body::Instance` arm because it too has
  a `makeArray` (`Setup.cpp:1445`). `receiver_kind` (`dispatch.rs:1551`) and `stem.rs`'s
  `body_variant_name` were changed ahead of the build because their docs say they are exhaustive on
  purpose. The three rendering matches — `text_len_inner` (`value.rs:642`), `to_text` (`:921`) and
  `try_text` (`:1012`) — have `other => unreachable!(...)` arms, so they *compile* without a
  `Buffer` arm and would **panic** the first time a buffer is rendered (a `TRACE` value line, an
  error-message substitution). The patch adds an arm to each, answering the contents, which is
  `MutableBuffer::stringValue` (`MutableBufferClass.cpp:740`). That is the whole of B's blast
  radius in this crate: one forced arm, two exhaustive-by-policy arms, three wildcard arms that had
  to be made honest.

Under B, `receiver_kind` answers `Primitive::Instance { class, behaviour }` for a buffer, so
dispatch, `~class`, `~hasMethod` and the behaviour lookup see it as an instance of its class. What a
`Body::Buffer` does **not** have is `Body::Instance`'s `pools`, `own` and `name`, and **that is an
observable gap, measured** (`$B/scratch/oracle-subclass/`, program `sub.rex`: a `::class sub
subclass MutableBuffer` whose `init` does `expose n; n = 42`, then `~n`, `~length`, `~append`,
`~objectName=`, `~objectName`):

```text
oracle              rc 0    42 3 / 42 5 / named / 5
candidate A2        rc 0    42 3 / 42 5 / named / 5          both engines, byte-identical
candidate B         rc 120  stderr: rexx-exec: EXPOSE on an object with no variable pool
                            is not implemented (Phase 5)     both engines
```

So the oracle's buffer is an ordinary object with an object-variable dictionary, A keeps that for
free, and B as built loses it — loudly (the `EXPOSE` path checks the body kind, `lib.rs:1059`,
before `set_exposed_variable`'s `expect` could fire), but loses it. Section 4 names the shape that
has B's bytes and A's instance-ness at once.

### 2.5 `size_of::<Body>()` under each candidate

**Measured** with `$B/scratch/size/size-probe2.sh` in a third extract (`$B/tree-size`, `rust/` +
`interpreter/` of `fdf4c6624`, `CARGO_TARGET_DIR=$B/target-size`): each variant of `body.rs` gets the
line `const _: [(); 0] = [(); size_of::<Body>()];` appended and `cargo check -p rexx-core` is run;
rustc's E0308 then prints the true width ("expected an array with a size of 0, found one with a size
of N"), and the crate's own `<= 80` assertion would appear as a second error if it tripped. Outputs
in `$B/scratch/size/run2/check-<variant>.{out,err,rc}`, each `body.rs` variant saved beside them as
`body-<variant>.rs`. Every check ended `rc=101` with **exactly one** error, the probe's
("could not compile `rexx-core` (lib) due to 1 previous error"), so the `<= 80` assertion held in
all four.

| variant | `size_of::<Body>()` | assertion at `body.rs:602` |
|---|---|---|
| pristine `fdf4c6624` | **80** | holds |
| candidate B as built: `Buffer(Box<BufferBody>)` | **80** | holds |
| B-prime: `Body::Instance` gains a field `native: Option<Box<[u8]>>` — a **fat** pointer, 16 bytes, so a thin `Option<Box<BufferState>>` (8) fits a fortiori | **80** | holds |
| an **unboxed** `Buffer(BufferBody)` (56-byte payload) | **80** | holds — my prediction that this trips was **falsified** |

(The first attempt, `$B/scratch/size/check-*.{err,rc}`, never reached `body.rs`: the extract had no
`interpreter/`, and `rexx-inventory`'s build script reads `interpreter/messages/rexxmsg.xml`. Kept
as the record of an environmental failure that read like a result.)

So neither candidate widens the arena, and two more shapes are also free: an unboxed `Buffer`
variant (one indirection fewer than B on every access), and an `Instance` that carries an optional
boxed native payload (which is how a buffer could keep `pools`, `own` and `name`). Section 4 uses
both.

## 3. Measurements

### 3.1 Instrument

**Measured**: `perf stat -e instructions:u -r 2 /bin/true` from inside this sandbox printed
`146,572 instructions:u ( +- 0.00% )` with no `<not supported>` and no counter shortage
(`$B/scratch/perf-probe.err`). So the instrument is `perf stat -e instructions:u -r 5`, one event
at a time (the sandbox has one PMU slot — `bench-baselines/README.md`). Every figure below names it;
if any run had fallen back to `valgrind --tool=callgrind`, its row would say `Ir` instead.

### 3.2 Prediction, written before running

Written 2026-09-05 before either candidate was built, so that the run can confirm or falsify it.

* **Append loop** (`append.rex`: 100000 one-byte appends from empty). Candidate A copies the whole
  current text into a fresh `Vec`, appends, and allocates a new `Body::Text` per send — the total
  bytes copied over the loop are the triangular sum, about 5×10⁹ — plus a heap `Vec` allocated and
  later freed per send once the text passes `INLINE_BYTES`, plus the collector pressure of 100000
  dead `Text` objects. Candidate B does an amortised `Vec::extend` on the buffer's own bytes.
  **Prediction: A ≥ 2× B in `instructions:u`**, on both engines, and the gap in wall time larger than
  the gap in instructions because the copies miss cache. Both engines should show the same ratio
  direction; the ir/tree-walker difference is in the loop, not the send, and is the same under
  both candidates.
* **Length loop** (`length.rex`: `l = buf~length` 100000 times over a 300-byte buffer). A does one
  `heap.get`, a linear scan of a pool with three entries comparing names bytewise, then
  `text_len` on the text handle (a second `heap.get`); B does one `heap.get` and a `.len()`.
  **Prediction: A within 1.00–1.10× of B** — the difference is tens of instructions against a send
  that costs hundreds to a thousand.
* **`size_of::<Body>()`**: B adds a variant whose payload is one `Box` (8 bytes), under the 80-byte
  ceiling `Body::Stem` sets, so **the assertion at `body.rs:602` does not trip**. A adds nothing to
  `Body`. Prediction: both build; `size_of::<Body>()` stays 80 under both.

### 3.3 Binaries (sha256, mtime)

| binary | sha256 | mtime | build |
|---|---|---|---|
| `$B/bin/rexx-run-baseline` | `dff098f31b5eb9569e92238768b16534f65558c9a8129179233649295c9f6e3a` | 2026-09-05 00:08:24 +0200, 27059200 bytes | pristine `fdf4c6624` extract, `cargo build --release -p rexx-exec --bin rexx-run` (`$B/scratch/build-baseline.{out,err,rc}`, rc 0) |
| `$B/bin/rexx-run-A` | `5b793469e93e3b0dcaff506a4c642ff168f5045231c19fcaa96b97ca0150a2a4` | 2026-09-05 00:12:30 +0200 | **not measured** — rows in the wrong table, panics at model build (2.3) |
| `$B/bin/rexx-run-A2` | `2fbe8006bd32ef76ce4a266fea2c5834f6cd485ebbe0d2d6fd2fc7fa12c79521` | 2026-09-05 00:15:35 +0200 (source `dispatch.rs` 00:14:14) | `$B/tree`, `$B/target`, `$B/scratch/build-A2.{out,err,rc}` rc 0, 0 warnings; `/bin/grep -a -c -F 'candidate A stores capacities'` = 1, `'Body::Buffer'` = 0 |
| `$B/bin/rexx-run-B` | `d23da4287dd5c5c58682c6333fe8c79950680826217266547ff6a1e3f3eec9b5` | 2026-09-05 00:16:25 +0200 (last source edit `run.rs` 00:14:57) | `$B/tree-B`, `$B/target-B`, `$B/scratch/build-B2.{out,err,rc}` rc 0, 0 warnings; `'Body::Buffer'` = 1 |

**Both candidates answer the probes, measured** (`$B/scratch/probes/*.{A2,B}.{out,err,rc}`, both
engines, all rc 0, stderr empty): `sanity.rex` prints `3 / 5 / 10`, which is the oracle's answer
from a fresh directory (`$B/scratch/oracle-sanity/sanity.{out,err,rc}`, rc 0); `append.rex` prints
`100000`; `length.rex` prints `300`; `control.rex` prints `control`.

**Negative control on the baseline, measured** (`$B/scratch/probes/*.base.{out,err,rc}`, three
descriptors, both engines): `append.rex`, `length.rex` and `sanity.rex` are all **rc 120** with
stderr `rexx-exec: method "APPEND" of class "MutableBuffer" is not implemented (Phase 5)` (or
`"LENGTH"`) and empty stdout; `control.rex` is rc 0 printing `control`. So a probe that later
answers under a candidate binary answers because of that candidate's patch and nothing else, and
the same probes distinguish a stale binary from a rebuilt one.

### 3.4 Append loop

All figures **measured** by `$B/scratch/measure.sh run1 $B/bin/rexx-run-A2 $B/bin/rexx-run-B`
(instrument `perf stat -e instructions:u -r 5 -o <cell>.perf`, cells interleaved A/B; raw files in
`$B/scratch/measure/run1/`) and `EVENT=cycles:u $B/scratch/measure2.sh run2-cycles …` (same
cells, `perf stat -e cycles:u -r 5`; `$B/scratch/measure/run2-cycles/`). Every cell's five runs
printed the expected stdout and rc 0. The control program has the loop, the constructor and an
assignment `l = buf` in place of the send, so *probe − control* is the cost of 100000 sends and
their argument evaluation.

`append.rex` — 100000 × `buf~append('x')` from an empty buffer:

| engine | cand. | `instructions:u` (±) | per send, minus control | `cycles:u` (±) | wall (perf, run1) |
|---|---|---|---|---|---|
| ir | A | 1,016,009,624 (0.03%) | 8,234 | 1,300,232,237 (0.14%) | 1.507 s |
| ir | B | 386,580,108 (0.01%) | 1,940 | 114,878,770 (1.00%) | 0.048 s |
| ir | **A/B** | **2.63×** | **4.24×** | **11.3×** | **31×** |
| tree-walker | A | 1,015,551,384 (0.03%) | 7,901 | 1,300,086,775 (0.09%) | 1.492 s |
| tree-walker | B | 385,795,551 (0.01%) | 1,602 | 116,011,853 (1.20%) | 0.048 s |
| tree-walker | **A/B** | **2.63×** | **4.93×** | **11.2×** | **31×** |

Control (`l = buf`, no send): ir A 192,555,392 / B 192,559,735 (0.002% apart); tree-walker A
225,450,534 / B 225,574,434 (0.055% apart) `instructions:u` — on a program that never enters the
buffer code the two binaries differ by less than a tenth of a percent, which is code-layout noise
between two LTO builds and the check that the probe ratios above are the buffer code's.

**Instructions understate A's cost by a factor of four against cycles**, and the reason is
visible in the shape: A's per-send work is dominated by copying the whole current text (the sum
over the loop is about 5×10⁹ bytes), and a vectorised or `rep movsb` copy retires very few
instructions per byte while paying for every cache line. The `cycles:u` ratio (11×) and the wall
ratio (31×, which also includes page faults) are the truer price.

**Peak memory, measured** with `/usr/bin/time -v env REXX_ENGINE=ir <bin> append.rex`
(`$B/scratch/rss/append.ir.{A2,B}.time`): A **4,068,068 KB maximum resident set** with 1,016,070
minor page faults; B **17,324 KB** with 3,948. **Inferred** cause: `Interp::collect_if_due`
(`lib.rs:7128`) triggers on the arena's slot count, not on bytes held, so A's 100000 dead `Text`s
of up to 100 KB each stay allocated between collections. A program of this shape would be killed
under the oracle wrapper's `ulimit -v 1048576`, so under candidate A the differential could not
even run such a witness.

### 3.5 Length loop

`length.rex` — 100000 × `l = buf~length` over a 300-byte buffer (same runs and files as 3.4):

| engine | cand. | `instructions:u` (±) | per send, minus control | `cycles:u` (±) |
|---|---|---|---|---|
| ir | A | 359,115,068 (0.01%) | 1,666 | 109,604,917 (0.99%) |
| ir | B | 344,170,096 (0.01%) | 1,516 | 105,617,172 (1.82%) |
| ir | **A/B** | **1.043×** | **1.099× (+150 per send)** | 1.038× (inside the ±) |
| tree-walker | A | 377,190,387 (0.01%) | 1,517 | 115,782,118 (0.96%) |
| tree-walker | B | 362,112,926 (0.01%) | 1,365 | 115,425,741 (2.39%) |
| tree-walker | **A/B** | **1.042×** | **1.111× (+152 per send)** | 1.003× (inside the ±) |

A pays about 150 instructions per `~length` for the pool scan by name and the second `heap.get`
that `text_len` on the text handle costs; in cycles that is inside the run-to-run spread.

### 3.6 Prediction: confirmed or falsified

* Append, A ≥ 2× B in instructions: **confirmed** (2.63× whole program, 4.2–4.9× per send).
* Append, wall gap larger than instruction gap: **confirmed**, and by more than expected (31× against
  2.6×); cycles at 11× sit between.
* Length, A within 1.00–1.10× of B: **confirmed on the whole-program figure** (1.043×, 1.042×);
  **at the edge on the per-send figure** (1.099× ir, 1.111× tree-walker), which is the number the
  prediction should have been about.
* Same direction on both engines: **confirmed** — the ratios agree to two digits across engines.
* `size_of::<Body>()` stays 80 under both, assertion does not trip: **confirmed**. The extra claim
  in 2.1 that an *unboxed* variant would trip it: **falsified** (2.5).
* Not predicted, and the largest single finding: A's **4 GB peak RSS** against B's 17 MB.

## 4. Recommendation

**The bytes live in the object, as a `Vec<u8>` beside a carried `capacity` and `default_size` —
candidate B's storage — but inside `Body::Instance`, not in a variant of its own.** Concretely:
`Body::Instance` gains one field, `native: Option<Box<BufferState>>` (or a name that admits later
native payloads), where `BufferState { bytes: Vec<u8>, capacity: usize, default_size: usize }`.
The figures that decide it: on the append loop A costs 2.63× B's `instructions:u`, 11× its
`cycles:u`, 31× its wall time and **4 GB against 17 MB of peak RSS** (3.4) — A's every-mutation-
reallocates shape is not a constant factor but a quadratic one, and a corpus witness of the plan's
own shape (`do 100000; buf~append('x')`) could not run under the oracle wrapper's 1 GB `ulimit`.
On the length loop the two are within 5% whole-program and within run-to-run spread in cycles
(3.5), so the read path does not distinguish them. What distinguishes B-as-built from A in the
other direction is instance-ness: a `MutableBuffer` subclass with `EXPOSE` runs on the oracle and
on A and refuses on B (2.4), and `~objectName=` / `~setMethod` live in `Body::Instance`'s `name`
and `own`. And the width measurement (2.5) says an `Option<Box<_>>` on `Instance` costs the arena
nothing — `size_of::<Body>()` stays 80 — so the shape that has both is free. The unboxed
`Body::Buffer(BufferBody)` is also free of the assertion, but it would have to carry `pools`, `own`
and `name` too to close the subclass gap, and 56 + 64 bytes does not fit; boxing the buffer state
inside `Instance` is the one shape that fits everything.

Cost of the recommended shape against B-as-built: one `Option` test on the buffer paths only (the
`Instance` arm already matched; nothing on any other object's path changes), and every
`Body::Instance { … }` *construction* gains `native: None`. A grep for `Body::Instance {` not
followed by `=>`, read with context, finds four constructions — `new_instance` (`dispatch.rs:4934`),
`run.rs:3172`, and two in tests (`value.rs:1931`, `eval.rs:4233`) — and three patterns that already
use `..` (`value.rs:1524`, `dispatch.rs:1833`, `run.rs:3156`). **The compiler is the enumeration**,
not this grep: a struct-variant construction missing the field is an error, a pattern with `..` is
not, so the recommended shape cannot be built with a site forgotten. Not built here.

`defaultSize` (the plan's review question 3): **carry it**. `setBufferSize(0)` reads it and nothing
else does (`MutableBufferClass.cpp:686`-`:691`), it is the constructor's second argument or 256
(`:110`-`:116`), and it is one `usize` in a boxed struct the arena never sees.

### 4.1 The API the methods will need (`&mut` access through `Interp`)

How the read path works today: `native_length` (`dispatch.rs:8011`) calls `interp.text_len(receiver)`
(`value.rs:559`), which does `self.heap.get(value)` → `redirect_of(&object.body)` → and for a
`Body::Text` a second `heap.get_mut` to reach `bytes.len()`; `try_text(&self, value) ->
Option<&[u8]>` (`value.rs:975`) is the shared-borrow form, and `Interp::render` + `Rendered::text`
(`:1044`, `:1450`) is how a builtin holds several values' bytes at once. There is no mutable
analogue anywhere yet, because a Rexx string is immutable (`bytes.rs` module doc) — the buffer is
the first heap object whose bytes change in place.

The mutable analogue, beside `array_slots`/`array_body` (`value.rs:737`, `:757`), which are the
existing "borrow one kind of body's payload" accessors:

```rust
/// A `MutableBuffer`'s state, or `None` for a value that is not one.
pub(crate) fn buffer(&self, value: ObjRef) -> Option<&BufferState> {
    match &self.heap.get(value)?.body {
        Body::Instance { native: Some(state), .. } => Some(state),
        _ => None,
    }
}
pub(crate) fn buffer_mut(&mut self, value: ObjRef) -> Option<&mut BufferState> { /* get_mut */ }
```

and a method body has the shape both prototypes already have (`native_mutable_buffer_append` in
either patch): **convert every argument first** — `whole_method_argument` / `required_string_argument`
for the 93.9xx family (section 1.1) and the argument's bytes copied into `take_result_buffer()`
(no allocation for the common short argument) — **then** take `buffer_mut(receiver)` and do the
byte work against `&mut state.bytes` with `try_reserve_exact` up to `state.capacity`, giving the
lent buffer back afterwards. The borrow checker enforces the discipline that matters for the
collector: while the `&mut BufferState` is live no `interp` call can allocate, so nothing can
collect under it. Readers that answer a new string do `let bytes = &interp.buffer(receiver)?.bytes`,
compute into `take_result_buffer()`, then `text_built`; readers that answer a count use `counted`.
A `None` from either accessor is the refusal path (`Loud::native_method`), never a panic.

### 4.2 `makeString` / rendering

Three facts, all **measured** (`$B/scratch/oracle-render/`, both candidates on `ir`):

1. `say buf` and `'x' || buf` are **`MAKESTRING`-loud (rc 120) under both A2 and B** — the
   required-string protocol sends `MAKESTRING`, and no `to_text` arm changes that. So the plan's
   Task 2 rule ("keep `makeString` out of it until Task 3; `say buf` stays loud rather than wrong")
   holds under the recommended shape too, and the flip is a `("MutableBuffer", "MAKESTRING", …)`
   row plus `("MutableBuffer", "STRING", …)` (`Setup.cpp:1446`, `:1473`), each answering
   `interp.text(&state.bytes)` — a fresh string per call, which is what `MutableBuffer::makeString`
   does (`MutableBufferClass.cpp:717`, read: `return new_string(data->getData(), dataLength);`);
   `stringValue` (`:740`) and `primitiveMakeString` (`:753`) both forward to it.
2. **The infallible renderings are a different path, and A gets them wrong today.** Under `trace i`,
   `x = buf` prints `>V> BUF => "abc"` on the oracle and on B; on A2 it prints `>V> BUF => "a
   MutableBuffer"` at rc 0 — a silent divergence, because `redirect_of` sends an unnamed instance to
   `Redirect::InstanceDefault`. The same path feeds error-message substitutions (`string_value_text`).
   Under the recommended shape `redirect_of` gets one arm ahead of the `name: None` arm —
   `Body::Instance { native: Some(_), .. } => Redirect::None` — and `to_text`, `try_text` and
   `text_len_inner` each get `Body::Instance { native: Some(state), .. } => state.bytes` where they
   now match `Body::Instance { name, .. }`. Under A the same fix would need a class-identity test on
   every instance rendering, or a fourth pool read.
3. A **named** buffer still renders as its contents — oracle: after `buf~objectName = 'named'`,
   `say buf` and `say buf~string` are `abc` and only `~objectName` is `named` — so the `native:
   Some(_)` arm must win over the `name: Some(_)` arm in the body match, and `~objectName` reads
   `name` as it does for any instance. `Body::Instance` already has both fields; a `Body::Buffer`
   variant would have had to grow a `name` to say this.

## 5. Optional — is Task 3 one task or several?

**Inferred from reading** `interpreter/memory/Setup.cpp:1416`-`:1479` (the 51 `AddMethod` rows
between `CompleteClassMethodDefinitions` and `CompleteMethodDefinitions`, `New` excluded) against
the builtin inventory of section 1 and `MutableBufferClass.cpp`'s function list. Names as
`Setup.cpp` spells them; the count column is the declared argument count (`A_COUNT` for `Append`).

| family | names (Setup.cpp) | what each maps to |
|---|---|---|
| **capacity / state** (buffer-only, no builtin) | `Append` A_COUNT, `SetText` 1, `GetBufferSize` 0, `SetBufferSize` 1, `Length` 0, `Delete` 2, `DelStr` 2 (one C++ method, `mydelete`, two names) | `ensureCapacity` / `setBufferSize` / `setDataLength` on the state itself; `Delete`'s byte core is `delstr`'s (`string.rs:479`) |
| **mutators sharing a builtin core** | `Insert` 4, `Overlay` 4, `ReplaceAt` 4, `[]=` 3, `ChangeStr` 3, `Upper` 2, `Lower` 2, `Translate` 5, `Space` 2, `DelWord` 2 | `insert`, `overlay` (`ReplaceAt` and `[]=` are `overlay` without the pad/extend — C++ `replaceAt` `:570`, `bracketsEqual` `:545`), `changestr`, `case_shifted`, `translate`, `space`, `delword`; each writes back into the buffer and answers the receiver |
| **readers sharing a builtin core** | `Substr` 3, `[]` 2, `Pos` 3, `LastPos` 3, `CountStr` 1, `Verify` 4, `SubWord` 2, `Word` 1, `WordIndex` 1, `WordLength` 1, `Words` 0, `WordPos` 2 | `substr` (`[]` is `substr` without pad — `brackets` `:787`), `find_forward`, `find_backward`, `count_occurrences`, `verify`, `Words`/`word_slices`; each reads the bytes and answers a fresh `Text` or a count |
| **derived readers, no builtin but trivial over the cores** | `Contains` 3, `ContainsWord` 2, `StartsWith` 1, `EndsWith` 1, `Match` 4, `MatchChar` 2, `SubChar` 1 | `pos != 0`, `wordpos != 0`, prefix/suffix compare, `primitiveMatch` (`:1615`, a slice compare at an offset), one byte against a set, one byte |
| **caseless** (no builtin counterpart anywhere in this crate) | `CaselessPos` 3, `CaselessLastPos` 3, `CaselessContains` 3, `CaselessContainsWord` 2, `CaselessCountStr` 1, `CaselessChangeStr` 3, `CaselessWordPos` 2, `CaselessMatch` 4, `CaselessMatchChar` 2, `CaselessStartsWith` 1, `CaselessEndsWith` 1 | each is its cased twin with an ASCII fold on the compare (`StringUtil::caselessPos` etc. in the C++); the natural shape is a comparator parameter on `find_forward`/`find_backward`/`count_occurrences`/the word compare, which the cased family would then also use |
| **conversion** | `String` 0 (`RexxObject::makeStringRexx`), `makeString` 0, `MakeArray` 1, `SubWords` 2 | `interp.text(&bytes)`; `makeArray(div)` is `String~makeArray` with a separator argument (`native_string_makearray`, `dispatch.rs:8060`, is the no-separator form only); `SubWords` is `word_slices` into an array |

Counting names: 7 + 10 + 12 + 7 + 11 + 4 = 51.

**Which families share enough to be one commit.** The dividing line is not the family, it is the
**prerequisite refactor**: the mutators and readers that share a builtin core cannot land before the
cores are extracted out of `string.rs`/`word.rs` into `fn(&[u8], …) -> …` functions the buffer can
call (section 1.2), and that extraction is one commit of its own with the existing builtin tests as
its witness. After it:

1. **Task 2 as planned** (`new`, `length`, `string`, `endsWith`, `append`, `delstr`, `getBufferSize`,
   `setBufferSize`) already spans the capacity family, one derived reader and one conversion — the
   witness fixes the grouping and it is right to leave it.
2. **Readers-over-cores + derived readers** are one commit: none mutates, each is "convert the
   arguments at the method layer, call a core, wrap the answer", and one corpus program can exercise
   all 19 names in a few dozen lines.
3. **Mutators-over-cores** are one commit for the same reason with the write-back added; `Delete`
   joins here if Task 2 has not already landed it.
4. **Caseless** is one commit, because its cost is the comparator plumbing through the cores, not
   the eleven bindings, and its witness has to hold mixed-case data that the cased witness need not.
5. **`makeString`/`String`/`makeArray`/`SubWords`** is a small commit and the one that flips `say
   buf` from loud to answering (section 4.2), so it should land with its own witness and not be
   folded into a reader commit whose witness never prints a buffer.

So: **several** — the core extraction, then four Task 3 commits (readers, mutators, caseless,
conversion) rather than the six families the plan's question names; the family lines are the right
lines, "search" and "word" collapse into readers-over-cores, and the capacity family is Task 2's.

## 6. Files and directories created under `$B`

`$B` = `/home/moritz/dev/repos/claude-build-scratch/5c-followup-task1`. Nothing was deleted or
overwritten except where noted; nothing was written outside `$B` (the worktree at
`ooRexx-rust-rewrite` was only read, through `git archive` and file reads).

* `task-1-report.md` — this file.
* `tree/` — `git archive fdf4c6624` (73 MB), with candidate A applied to
  `rust/crates/rexx-exec/src/dispatch.rs`. `target/` — its cargo target dir (95 MB); the baseline
  binary was built here first, then A on top.
* `tree-B/` — a second `git archive fdf4c6624` with candidate B applied to six files (2.4).
  `target-B/` — its target dir (95 MB).
* `tree-size/` — `rust/` and `interpreter/` of `fdf4c6624`, used for the `size_of` probes; its
  `rust/crates/rexx-core/src/body.rs` is left at the last probe variant (unboxed `Buffer`, with the
  probe line). `target-size/` (28 MB).
* `bin/rexx-run-baseline`, `bin/rexx-run-A` (the mis-tabled build, unused), `bin/rexx-run-A2`,
  `bin/rexx-run-B` — sha256 in 3.3.
* `orig/` — pristine copies of every file a patch touches (`body.rs`, `dispatch.rs`, `eval.rs`,
  `rexx-core-lib.rs`, `run.rs`, `stem.rs`, `value.rs`), the `diff -u` base for the patches.
* `patches/candidate-A.patch` (7078 bytes), `patches/candidate-B.patch` (12106 bytes).
* `scratch/perf-probe.{out,err}` — the perf availability check.
* `scratch/build-baseline.{out,err,rc}` — **overwritten once**: the first write recorded a `cargo
  build -p rexx-cli` that failed (`rexx-cli` is not a package; the binary is `rexx-exec`'s
  `rexx-run`), the second is the real baseline build. `scratch/build-A.*`, `build-A2.*`,
  `build-B.*` (the one-error compile, 2.4), `build-B2.*`.
* `scratch/measure.sh`, `scratch/measure2.sh` (the `EVENT`-parametrised copy).
* `scratch/probes/{append,length,control,sanity}.rex` and their outputs
  `<probe>.<engine>.<base|A|A2|B>.{out,err,rc}`.
* `scratch/measure/run1/` (instructions:u) and `scratch/measure/run2-cycles/` (cycles:u):
  `<probe>.<engine>.<A|B>.{perf,out,err,rc}` — `A` there is `rexx-run-A2`.
* `scratch/rss/append.ir.{A2,B}.{out,time,rc}`.
* `scratch/size/size-probe.sh`, `size-probe2.sh`; the first attempt's `body-*.rs` and
  `check-*.{out,err,rc}` (no `interpreter/`, never reached `body.rs`); `scratch/size/run2/` — the
  valid `body-*.rs` and `check-*.{out,err,rc}` for the four variants.
* Oracle probes, each in its own fresh directory: `scratch/oracle-argerr/` (`m1`, `f1`, `m2`),
  `scratch/oracle-sanity/`, `scratch/oracle-subclass/` (`sub.rex`, oracle and both candidates),
  `scratch/oracle-render/` (`render.rex`, `saybuf.rex`, `concat.rex`, `tracebuf.rex`, oracle and
  both candidates), `scratch/oracle-bang/` (`bang.rex` on the oracle; `alias.rex` on the oracle and
  on A2, both engines).

**Inferred, not run**: both prototype trees should fail
`a_constructor_taking_arguments_answers_an_instance_and_refuses_its_state` (`dispatch.rs:9331`),
whose name says it asserts the state is *not* kept. Neither prototype's test suite was run — only
`rexx-run` was built in each tree — and they build nothing that ships.

## 7. Open questions not settled here

* Whether every `Body::Instance { … }` match outside the five construction sites really uses `..`
  — asserted from the grep in 2.4, not from a build of the recommended shape.
* The exact `93.9xx` sub-codes and messages for each method's argument refusals: measured for one
  (`substr('x')` → 93.924) and inferred to differ per method; Task 3 measures each.
* Whether `makeArray(div)` with a separator shares anything with `native_string_makearray`
  (`dispatch.rs:8060`, no-separator form) — not read.
* The `String~caseless*` methods do not exist in this crate either, so the caseless comparator
  plumbing in 5 would be new to the tree; whether `String`'s 112 loud rows want the same plumbing
  later was not checked.
* `collect_if_due` triggering on slot count rather than bytes (3.4) is inferred from `lib.rs:7128`
  and the 4 GB reading, not from a heap-instrumented run.
