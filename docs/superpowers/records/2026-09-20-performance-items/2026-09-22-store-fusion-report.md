# Price a removed op: the store fusion, 2026-09-22

BASE is `87aee846b`. One commit of code: **`d6aec7d38`**, "Fuse a value op with
the Store that consumed it".

All instruction figures are `valgrind --tool=callgrind`'s `summary:` line, two
to five rounds per build interleaved, each build in its own `CARGO_TARGET_DIR`.
All op figures are the summed execution counts at the `jmp *` addresses inside
every `Interp::run_ops_from` instantiation, the method
`2026-09-20-instructions-per-op.md` sets out. The counter was checked against
that document rather than assumed: it reproduces its recorded `rexxcps` rows
(19,441,281 outer `<true>`, 1,120,000 and 8,400,000 `<false>` -- the last as
7,840,000, the `Condition`/`JumpUnless` fusion already being in BASE), the
op-fusion report's post-fusion 93,024,008 region `<true>`, and its `varlookup`
171,000,547 and `emptyloop` 50,000,544. Its own self-cost total equals the
`summary:` line to the unit.

## Gates

Run at `d6aec7d38` with the tree frozen, each status written unpiped to a
status file whose first line is the commit sha, builds outside the memory cap
and tests under it. Window 2026-09-22T11:06:35+02:00 to 11:19:19+02:00; the
commit was made before the run started.

    commit: d6aec7d38df770387842c02f3bb49ecf661952b8
    cargo fmt --all --check                                                      -> rc 0
    cargo clippy --workspace --all-targets -- -D warnings                        -> rc 0
    cargo build --workspace --all-targets --release                              -> rc 0
    REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast -> rc 0, 2651 passed / 0 failed / 4 ignored, 133 binaries
    cargo build --workspace --all-targets                                        -> rc 0
    REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast           -> rc 0, 2652 passed / 0 failed / 4 ignored, 133 binaries
    tree edited during window: no

`corpus_differential` printed **604 of 604 matching, mode STRICT** inside both
gated runs. The per-binary tallies are summed by script from the `test result:`
lines in each run's own log rather than read off a summary line.

"Tree edited during window: no" is two checks: `git status --short` empty, and
a `find` over `rust/`, `.superpowers/` and `docs/` for files modified inside
the window, excluding `target/`, returning nothing.

The clippy gate finished in 0.09s against a warm `rust/target`, which
`rust/CLAUDE.md` says to treat as provisional. It was re-run from an empty
`CARGO_TARGET_DIR`: **30 crates compiled cold, rc 0, no warnings.**

## The number

**A removed op is worth 35.0 instructions on `varlookup` and 36.0 on `arith`.**
The two axes agree, and the brief's recorded prediction of 8 is wrong by a
factor of four and a half. The answer is much nearer the 54.8 point than the 8
point, and by the brief's own rule the direction is worth a plan rather than an
afternoon.

**But there is a second constant, and it decides the question the first one was
asked for: the four new driver arms cost `rexxcps` 2.32% with nothing fused,
and one arm alone costs 0.26%.** The tax is not a cliff the first arm pays. It
is super-linear in how much driver code you add: 4.9 times the code for 8.9
times the cost.

Put together:

> **Widening pays only where the new shape removes more than about a tenth of
> the dispatched ops, and the price of the next shape is higher than the price
> of the last one.** `varlookup` removes 22.2% of its ops and gains 4.88%;
> `arith` removes 13.9% and gains 0.17%; `rexxcps` removes 0.00009% and
> **loses 2.32%**. This commit makes `rexxcps` slower.

### How the second constant was isolated

`base -> head` alone cannot price a removed op, and the instrument that shows
it is `emptyloop`: **it moves -1.03% while removing three ops in the whole
run.** A quotient there reads 33 million instructions per removed op, which is
not a price, it is a code-generation effect wearing one.

So two further binaries were built from `d6aec7d38`:

* **`noop`** -- `fuse_store` gated behind an environment variable, so the four
  driver arms are present and **nothing fuses**. Its `run_ops_from` is the same
  size as the committed one to the instruction (3,443 for `<true>`, frame
  1,368), so `base -> noop` is the driver change with the op stream held fixed
  and `noop -> head` is the fusion with the driver held fixed. The control was
  inverted to prove it live: with `REXX_FUSE_STORE=1` the same binary emits the
  fused stream.
* **`onearm`** -- only `Op::LoadStore` exists, variant and arm and emission.
  `rexxcps` fuses **nothing** under it, so its `rexxcps` figure is one arm's
  tax and no fusion at all.

A third, **`loadonly`**, keeps all four arms and fuses only `Load` + `Store`,
which splits `varlookup`'s answer between the two shapes.

| | `base -> noop`: arms only, 0 ops | `noop -> head`: fusion only | `base -> head`: shipped |
|---|---:|---:|---:|
| `varlookup` | +2.8847% | **-7.5488%, 35.00 Ir/op** | -4.8818% |
| `arith` | +0.5936% | **-0.7594%, 36.03 Ir/op** | -0.1703% |
| `emptyloop` | -1.0322% | -0.0000%, 3 ops | -1.0322% |
| `rexxcps` | +2.3164% | -0.0000%, 109 ops | **+2.3164%** |
| `rexxcps`, **one arm** | **+0.2604%** | -- | -- |

The arms' cost per **dispatched** op: `varlookup` +2.889, `arith` +3.888,
`rexxcps` +3.562, `emptyloop` **-2.000**. `emptyloop` is the one program that
barely enters the region walk -- 288 region dispatches against 50,000,256 outer
ones -- and it is the one program the arms make faster. That is a correlation
over four programs, not a mechanism anybody has run down.

### The two shapes agree with each other too

| on `varlookup` | delta | ops | Ir per removed op |
|---|---:|---:|---:|
| `noop -> loadonly` -- `LoadStore` alone | -684,010,257 | -19,000,000 | **36.00** |
| `loadonly -> head` -- `ArithStore` and two `LoadConstantStore` | -646,004,833 | -19,000,004 | **34.00** |

35.00, 36.00, 34.00 and 36.03, from three binary pairs on two axes. This is not
a `Load`-shaped or an `Arith`-shaped constant; it is a dispatch plus the
register handoff, and it does not care which op produced the value.

### Against the predictions

* **The brief's prediction, 8 per removed op, is falsified** by four
  quotients. Its derivation -- five instructions of region-dispatch preamble
  and three of latch -- prices the *dispatch*, and the measurement says the
  handoff between the pair is worth roughly another 27.
* **My own prediction was 20 to 24, central 22, recorded for `arith` before its
  figure existed** and with `varlookup`'s `base -> head` quotient of 22.00 in
  hand. Wrong, and in the same direction and for the same reason as the
  brief's: it was a prediction about the contaminated quotient. Its own
  stopping rule -- "if `arith` comes out materially below 20 I will read that as
  the store tail costing more where the clause holds more live state" -- would
  have fired on the 8.03 that `base -> head` reports and sent me after the
  wrong thing. No prediction was recorded before the first `varlookup` run, and
  there is no honest way to add one afterwards.
* **The brief's two axes were expected to disagree, and they do not.** The
  disagreement in the shipped column -- 22.00 against 8.03 -- is entirely the
  driver tax landing on two programs with very different op mixes. Held fixed,
  35.00 and 36.03.
* **`2026-09-22-clause-shape-distribution.md`'s arithmetic does not close.** It
  gives the `Condition` + `JumpUnless` fusion as "54.8 per removed op, of which
  only 8 was dispatch and 34.8 the value handoff"; 8 + 34.8 = 42.8. The
  op-fusion report it summarises says 20 of dispatch plus 34.8 of handoff, which
  does sum to 54.8. The brief inherited the 8 from the summary.

## What landed

An `Op::Load`, `Op::Const`, `Op::LoadConstant` or `Op::Arith` that leaves its
value in a register, immediately followed by the `Op::Store` that moves that
register into the assignment's target, becomes one op that writes the target
itself: `Op::LoadStore`, `Op::ConstStore`, `Op::LoadConstantStore`,
`Op::ArithStore`. `Op` stays 16 bytes wide, which the existing `const` assert
enforces; the fused ops carry no `index`, which was `Op::Store`'s debug-only
tripwire and is what made the four fit.

**The fusion is made where the ops are emitted.** `compile`'s `Assignment` arm
calls `push_value`, then replaces the op it just pushed rather than pushing a
`Store` behind it. `op_of`, `first_op_of`, the patch list and every `Clause`'s
`end` all come from `op_index(&ops)` during that same forward walk, so each is
consistent with the shorter stream by construction and nothing renumbers.

`Op::Store` is still reached from `Op::EvalExpr`, `Op::CallExpr`,
`Op::CallArgs`, `Op::Binary`, `Op::Prefix`, and from any value op with an echo
behind it.

### The hazards, against what was built

* **The register must be dead after the store.** The fusion is refused unless
  the value op is the *last* op emitted. A value echo is pushed behind its own
  value op, so a clause that echoes has the echo on the end and never fuses --
  hazard 1 and hazard 3 discharged by one test. The register the pair used to
  hand the value through keeps whatever the previous clause left in it, which
  is a live handle the collector still scans rather than a hole.
* **Nothing may branch to the store.** A jump target is `first_op_of[i]` or
  `op_of[i]`, both of which name an instruction's own first op, and a fused op
  sits inside a clause region rather than at one of those.
* **No conditional was added to the per-clause path.** The new arms are per-op
  arms in the region walk; `Op::Clause`'s own body is untouched.
* **The value is unrooted between its production and the store.** The slot
  write reaches `Roots::set_frame_slot` or `Interp::set_exposed_variable` and
  allocates nothing; every other route is `Interp::assign_evaluated`, whose
  first statement is `self.roots.push_temp(value)`.

### The inlining is half the result, and it was measured wrong first

Written with the shared store tail as one `#[inline]` method, the compiler put
it out of line and called it from each fused arm. That build is **worse than
the unfused pair on both axes**: `varlookup` 17,185,587,576 -> 17,413,610,350
and `arith` 12,394,068,631 -> 12,456,723,808, one round each, with
`call ...store_fused` visible four times in `run_ops_from`'s disassembly. A
call in the driver's hot loop costs the live state it clobbers.

Splitting it -- `#[inline(always)]` over the slot write, `#[inline(never)]`
`store_fused_general` for stems, compounds, unresolved slots and any write that
owes a `>>>` line -- is what turns +1.33% into -4.88% on `varlookup`. That
binary was not kept; its dumps are under `cg-outline-variant/`.

## `arith` cannot resolve better than half a percent, and the cause is glibc

**`arith`'s raw within-build spread is 0.57%**, against `varlookup`'s 0.0002%.
It is not the op stream: its op counts are identical to the unit across five
rounds and its stdout is byte-identical. Diffing two `base` rounds function by
function puts all but 34 thousand of the 70,413,592-instruction difference
inside `<rexx_num::Number>::mul`'s *inclusive* cost on an identical 500,000 calls, and
under that in glibc's `_int_malloc` self cost: **115,232,206 in one round and
63,708,035 in the other**, with `collect_now`, `alloc_with`, `free` and
`realloc` identical to the unit.

So every figure above is **`summary:` minus everything attributed to
`libc.so.6` and `ld-linux`**. Under that correction `arith` reproduces to
**0.0011%** and the other three axes are unchanged in substance -- `varlookup`'s
fusion delta moves by 12 thousand instructions in 1.33 billion.

| | rounds | raw spread | ex-libc spread |
|---|---:|---:|---:|
| `varlookup` | 2 per arm | 0.0002% | 0.0000% |
| `arith` | 5 per arm | **0.5687%** | **0.0011%** |
| `emptyloop` | 2 per arm | 0.0001% | 0.0000% |
| `rexxcps` | 2 per arm | 0.0217% | 0.0018% |

**Anything previously A/B'd on `arith` at under half a percent from raw
`summary:` lines is unresolved, not measured.**

## The driver's frame, read before and after

| | BASE | one arm | `d6aec7d38` |
|---|---:|---:|---:|
| `run_ops_from::<true>` frame | **1,416** (`sub $0x588,%rsp`) | 1,400 (`$0x578`) | **1,368** (`$0x558`) |
| `run_ops_from::<false>` frame | 1,384 (`$0x568`) | 1,384 (`$0x568`) | 1,320 (`$0x528`) |
| `run_ops_from::<true>` instructions | 2,897 | 3,009 | 3,443 |
| `run_ops_from::<true>` bytes | 15,071 | -- | 17,599 |
| whole `.text` | 2,526,059 | -- | 2,531,307 |

**The frame shrank while the code grew**, which is the interaction
`2026-09-22-driver-frame-pressure.md` asked for and the opposite of what that
note's mechanism predicts: the frame is the direct instrument it names, it
moved the *right* way, and the change still cost `rexxcps` 2.32%. **So the tax
is not frame size.** What it is was not established here, and the outlining
variant that note proposes is now more interesting rather than less, because
this is a second measurement saying the driver's cost tracks something other
than its frame.

(The 2,897 reproduces that note's 2,894 for a different commit; the count is
objdump mnemonic lines between the symbol and the next blank line.)

## The streams, before and after

    $ target/release/rexx-ir rust/bench-programs/varlookup.rex

| BASE | `d6aec7d38` |
|---|---|
| `12: Clause index=3 end=17` | `10: Clause index=3 end=14` |
| `13: Load read=Simple at=1 dst=2` | `11: Load read=Simple at=1 dst=2` |
| `14: LoadConstant dst=3` | `12: LoadConstant dst=3` |
| `15: Arith op=+ hint=0 lhs=2 rhs=3 dst=2` | `13: ArithStore op=+ hint=0 lhs=2 rhs=3 at=1` |
| `16: Store index=3 at=1 src=2` | |
| `17: Clause index=4 end=20` | `14: Clause index=4 end=16` |
| `18: Load read=Simple at=1 dst=2` | `15: LoadStore read=Simple from=1 at=3` |
| `19: Store index=4 at=3 src=2` | |
| `20: LoopNext index=2` | `16: LoopNext index=2` |

24 ops become 20. Thirteen golden op-stream tests moved with it and were
updated; `a_prefix_operator_compiles_to_a_native_op_and_its_own_echo` and
`a_compiled_read_names_the_symbol_its_expression_does` did **not**, which is
the refusal being selective rather than the fusion being absent -- the first
because `Op::Prefix` is not one of the four, the second because it compiles
under `intermediates()` and the echo blocks it.

`rexxcps`: 720 static ops become 712. `Store` 28 -> 20, and the eight are five
`LoadConstantStore`, one `ConstStore` and two `ArithStore`. Executed, that is
**109 ops out of 121,425,289**, which is why it banks nothing there. Its
constant-to-store pairs in the timed body are separated by a live
`TraceLiteral`, exactly as the brief said.

## Differential

The arbiter, `corpus_differential`, is **604 of 604 STRICT** in both gated runs.

Fifteen hand-written probes beyond that, each run from a fresh empty directory,
oracle wrapped
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`,
with **stdout, stderr and exit status compared as three separate descriptors**
against both the oracle and BASE:

1. an unset read through a fused `LoadStore` -- `ZV`, rc 0
2. `signal on novalue` over the same -- `novalue at 2`, rc 0
3. `trace r` over four assignments including a compound target -- the `>>>`
   lines on stderr, rc 0
4. `trace i` -- the shape that refuses to fuse, rc 0
5. stem default, compound read, compound write, whole-stem read, rc 0
6. `zb = za / 0` -- error 42 on stderr, **rc 214**
7. `procedure expose` with a fused store into an exposed slot, rc 0
8. `drop` between two fused stores inside a loop, rc 0
9. `signal on syntax` over `'abc' + 1` -- `syntax 41 at 3`, rc 0
10. `trace value 'R'` then `trace off`, rc 0
11. 4,000 iterations of `copies()` and a fused `LoadStore` of the result, rc 0
12. a class method whose `expose`d variable is written through a fused store, rc 0
13. `numeric digits 30` arithmetic through `ArithStore`, rc 0
14. `interpret 'zw = zv'` beside a fused store of the same value, rc 0
15. an assignment to the loop control variable inside its own loop, rc 0

All fifteen byte-identical on all three descriptors, oracle and BASE and HEAD.

## Binaries

    objcopy -O binary --only-section=.text <bin> <out> && sha256sum <out>

    BASE  748b2f0679dcddb9cf65d2e30d5c50a18e4f90095ecb634f8d018c4bfcce4c32
    HEAD  821b3ba83f37147cf4eb0d6f26d8d5fbd69eddc8e1b217691c8ef38e7fc45ccf

The HEAD figure was confirmed against an independent rebuild of `d6aec7d38` in
a fresh worktree and a fresh `CARGO_TARGET_DIR`: the same `.text` hash.

## A defect the existing suite caught

`Chunk::interned_symbols` is sized by scanning the stream for the ops that read
it, and that scan named `Op::LoadConstant` only. With `Op::LoadConstantStore`
absent from it the table is short, every lookup misses, `remember_symbol`
cannot take, and the constant is rebuilt on every execution.
`a_constant_symbol_is_built_once_and_each_one_gets_its_own_entry` failed 5
against 32 and named it. This is the "cache that lies" shape: the program
answers correctly and pays for it forever.

## Concerns

1. **This commit makes `rexxcps` 2.32% slower**, and `rexxcps` is the axis
   Moritz's standing goals are stated against. The fusion is not what costs it;
   the arms are. Reverting gives that back and loses `varlookup`'s 4.88% and
   `arith`'s 0.17%. **That is a decision, not a defect, and it is not mine to
   take** -- the commit is the instrument the brief asked for, and it can be
   reverted with the measurement kept.
2. **The tax is super-linear and that is the finding that bears on a
   superinstruction set.** One arm is +112 driver instructions and +0.26% on
   `rexxcps`; four are +546 and +2.32%. 4.9x the code, 8.9x the cost. A fixed
   set of ten shapes is not ten times one arm's price, it is more, and nothing
   here says where it stops. **Two points do not fix a curve** -- a build at two
   or three arms would.
3. **Item 2 is the prerequisite, not the alternative.** Ten further `Store` ops
   in `rexxcps`' hot body are constant-fed with exactly one `Op::TraceLiteral`
   in between, and those echoes exist only because `trace_flow::analyse`
   answers `Unknown` for a body containing two `trace value <expr>` clauses. Get
   those ops out of the stream and the ten sites become strictly adjacent and
   fuse with **no new arm at all** -- the tax is already paid. At 1,740,000
   executions and 35 Ir each that is 60.9M, 0.30%, against the 2.32% already
   spent. **That arithmetic rests on the 1,740,000, which is somebody else's
   measurement and was not re-derived here.**
4. **The fused ops dropped `Op::Store`'s `index` field and with it
   `debug_assert_names_the_clause`.** The run-time check that the clause is an
   `InstructionKind::Assignment` is retained and still returns
   `Loud::store_op_off_its_node`, so a fused op on the wrong clause fails
   loudly rather than writing a stranger's slot; what is gone is the debug
   tripwire that would have named it earlier.
5. **`Op::Binary` and `Op::Prefix` were deliberately left unfused.** They are
   the two remaining value ops that can end an assignment's region. By point 2
   they would cost more than the four already added, and on the four axes
   measured here they would remove nothing, because no hot clause on any of
   them ends in either.
6. **The one-arm `rexxcps` reading is two rounds of `summary:` (20,233,461,832
   and 20,233,551,971, 0.00045% apart) and one intact dump**, because a stale
   background script wrote the same filename and truncated the first round's
   dump after its status line was recorded. The ex-libc figure above is from
   the surviving dump.

## Scratch

`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/storefusion/`
holds `gates/` with the six logs and the cold clippy run, the callgrind dumps
under `cg/` and the discarded out-of-line variant's under `cg-outline-variant/`,
`opcount.py`, `ours.sh` and `final.py`, each variant's `target/` with its
binary and its `objdump` listing, the fifteen probes with their
three-descriptor outputs, and `prediction-arith.txt`.

The five temporary worktrees were removed and `git worktree list` is back to
what it was; `variant-patches/` holds the diff against `d6aec7d38` that makes
each of `noop`, `loadonly` and `onearm`.
