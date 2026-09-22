# Price a removed op: the store fusion, 2026-09-22

BASE is `87aee846b`. One commit: **`d6aec7d38`**, "Fuse a value op with the
Store that consumed it".

All instruction figures are `valgrind --tool=callgrind`'s `summary:` line, two
rounds per build interleaved, each build in its own `CARGO_TARGET_DIR`. All op
figures are the summed execution counts at the `jmp *` addresses inside every
`Interp::run_ops_from` instantiation, the method
`2026-09-20-instructions-per-op.md` sets out. The counter was checked against
that document rather than assumed: it reproduces its recorded `rexxcps` rows
(19,441,281 outer `<true>`, 93,024,008 region `<true>`, 1,120,000 and 8,400,000
`<false>` -- the last as 7,840,000, which is the `Condition`/`JumpUnless`
fusion already in BASE), and the op-fusion report's `varlookup` 171,000,547 and
`emptyloop` 50,000,544. Its own self-cost total equals the `summary:` line to
the unit.

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

## The number, and it is two numbers

**A removed op is worth 35.0 instructions on `varlookup` and 38.1 on `arith`,
with the driver's own code held fixed.** The brief's recorded prediction was 8;
it is wrong by a factor of four, and the answer sits nearer the 54.8 end of the
known range than the 8 end.

**But there is a second constant, and nobody asked for it because nobody knew
it was there: adding the four arms to the driver costs about three instructions
per dispatched op on every program, whether or not it fuses anything.** On
`rexxcps` that tax is **+2.13%** and the fusion buys back 109 ops out of 121
million, so this commit makes `rexxcps` *slower*. Both numbers are measured
against the same binary pair.

The two together are the answer to "is widening the ops worth a plan":

> **Widening pays only where the new shape removes more than about a tenth of
> all dispatched ops.** 3 instructions of tax per op against 35 of saving per
> removed op puts break-even near 8.6% of the stream. `varlookup` removes
> 22.2% and wins by 4.9%; `arith` removes 13.9% and wins by 0.3%; `rexxcps`
> removes 0.00009% and loses by 2.1%.

### How the second constant was isolated

`base -> head` alone cannot price a removed op, and the instrument that shows
it is `emptyloop`: **it moves -1.03% while removing three ops in the whole
run.** A quotient there reads 33 million instructions per removed op, which is
not a price, it is a code-generation effect wearing one.

So a third binary was built: the committed code with `fuse_store` gated behind
an environment variable, so **the four driver arms are present and nothing
fuses**. Its `run_ops_from` is byte-for-byte the same size as the committed
one -- 3,443 instructions and a 1,368-byte frame for `<true>` -- so
`noop -> head` isolates the fusion with the driver held fixed, and
`base -> noop` isolates the driver change with the op stream held fixed. The
control was inverted to prove it live: with `REXX_FUSE_STORE=1` the same binary
emits the fused stream.

| | `base -> noop`, arms only | `noop -> head`, fusion only | `base -> head`, what shipped |
|---|---:|---:|---:|
| `varlookup` | +2.8745%, 0 ops | **-7.5229%, 35.00 Ir/op** | -4.8647% |
| `arith` | +0.4976%, 0 ops | **-0.7650%, 38.12 Ir/op** | -0.2712% |
| `emptyloop` | -1.0257%, 0 ops | -0.0001%, 3 ops | -1.0258% |
| `rexxcps` | +2.1289%, 0 ops | +0.0125%, 109 ops | **+2.1417%** |

The arms' cost per **dispatched** op: `varlookup` +2.889, `arith` +3.426,
`rexxcps` +3.539, `emptyloop` **-2.000**. `emptyloop` is the one program that
barely enters the region walk -- 288 region dispatches against 50,000,256 outer
ones -- and it is the one program the arms make faster. That is a correlation
over four programs, not a mechanism anybody has run down.

### A fourth binary decomposes the two shapes

A build fusing only `Load` + `Store`, with the other three arms still in the
driver, splits `varlookup`'s 35.00:

| | delta | ops | Ir per removed op |
|---|---:|---:|---:|
| `noop -> loadonly` -- `LoadStore` alone | -684,016,185 | -19,000,000 | **36.00** |
| `loadonly -> head` -- `ArithStore` and two `LoadConstantStore` | -646,010,826 | -19,000,004 | **34.00** |

The two shapes agree. This is not a `Load`-shaped or an `Arith`-shaped
constant; it is the price of a dispatch plus the register handoff, and it does
not care which op produced the value.

### Against the predictions

* **The brief's prediction, 8 per removed op on this shape, is falsified**, by
  four independent quotients: 35.00, 36.00, 34.00 and 38.12. Its derivation --
  five instructions of region-dispatch preamble and three of latch -- prices
  the *dispatch*, and the measured figure says the handoff between the pair is
  worth roughly another 27.
* **My own prediction was 20 to 24, central 22, recorded for `arith` before its
  figure existed** and with `varlookup`'s `base -> head` quotient of 22.00 in
  hand. It is wrong in the same direction as the brief's and for the same
  reason: it was a prediction about the contaminated quotient, and the number
  it should have been about is 38.12. No prediction was recorded before the
  first `varlookup` run, and there is no honest way to add one afterwards.
* **The arithmetic in `2026-09-22-clause-shape-distribution.md` does not close.**
  It gives the `Condition` + `JumpUnless` fusion as "54.8 per removed op, of
  which only 8 was dispatch and 34.8 the value handoff"; 8 + 34.8 = 42.8. The
  op-fusion report it summarises says 20 of dispatch plus 34.8 of handoff, which
  does sum to 54.8. The brief inherited the 8 from the summary. 20.0 and 8 are
  different quantities -- one is an A/B of deleting trace ops, the other a
  static read of a preamble -- and neither is the price of removing an op with
  a live handoff, which is what this measures.

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

### The inlining is the whole result, and it was measured wrong first

Written with the shared store tail as one `#[inline]` method, the compiler put
it out of line and called it from each fused arm. That build is **worse than
the unfused pair on both axes**: `varlookup` 17,185,587,576 -> 17,413,610,350
and `arith` 12,394,068,631 -> 12,456,723,808, one round each, with
`call ...store_fused` visible four times in `run_ops_from`'s disassembly. A
call in the driver's hot loop costs the live state it clobbers.

Splitting it -- `#[inline(always)]` over the slot write, `#[inline(never)]`
`store_fused_general` for stems, compounds, unresolved slots and any write that
owes a `>>>` line -- is what turns +1.33% into -4.86% on `varlookup`. That
binary was not kept; its dumps are in the scratch directory under
`cg-outline-variant/`.

## The driver's frame, read before and after

| | BASE | `d6aec7d38` |
|---|---:|---:|
| `run_ops_from::<true>` frame | **1,416** bytes (`sub $0x588,%rsp`) | **1,368** (`sub $0x558`) |
| `run_ops_from::<false>` frame | 1,384 (`sub $0x568`) | 1,320 (`sub $0x528`) |
| `run_ops_from::<true>` instructions | 2,897 | 3,443 |
| `run_ops_from::<true>` bytes | 15,071 | 17,599 |
| whole `.text` | 2,526,059 | 2,531,307 |

**The frame shrank while the code grew**, which is the interaction
`2026-09-22-driver-frame-pressure.md` asked for and the opposite of what that
note's mechanism would predict: the frame is the direct instrument it names,
and it moved the *right* way on a change that cost `rexxcps` 2.13%. So the
+2.13% is not frame size. What it is was not established here.

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
it, and that scan named `Op::LoadConstant` only. With
`Op::LoadConstantStore` absent from it the table is short, every lookup misses,
`remember_symbol` cannot take, and the constant is rebuilt on every execution.
`a_constant_symbol_is_built_once_and_each_one_gets_its_own_entry` failed
5 against 32 and named it. This is the "cache that lies" shape: the program
answers correctly and pays for it forever.

## Concerns

1. **`arith`'s within-build spread is 0.16% to 0.21%, twenty times
   `varlookup`'s, and it is not the op stream.** Its op counts are identical to
   the unit across rounds and its stdout is byte-identical, so the varying
   instructions are outside op dispatch -- most likely allocation or collection
   that depends on something run-to-run. The `arith` cells above rest on two
   rounds against a signal of 0.77%; further rounds are running and this report
   will be corrected if they move it. `varlookup`, `emptyloop` and `rexxcps`
   reproduce to 0.0002%, 0.0001% and 0.02%.
2. **The four arms' cost is a measurement of this change, not a per-arm
   constant.** Whether it is four times one arm's price or a cliff that the
   first arm pays is a separate experiment, and it is the one that decides
   whether a superinstruction set is viable. A one-arm build is running against
   `rexxcps`, where it fuses nothing, and its result belongs in this report.
3. **This commit makes `rexxcps` 2.14% slower**, and `rexxcps` is the axis
   Moritz's standing goals are stated against. The fusion is not what costs it;
   the arms are. Reverting would give the 2.14% back and lose `varlookup`'s
   4.86% and `arith`'s 0.27%.
4. **Item 2 is the prerequisite, not the alternative.** Ten further `Store` ops
   in `rexxcps`' hot body are constant-fed with exactly one `Op::TraceLiteral`
   in between, and those echoes exist only because `trace_flow::analyse`
   answers `Unknown` for a body containing two `trace value <expr>` clauses. Get
   those ops out of the stream and the ten sites become strictly adjacent and
   fuse with no new arm at all -- at 1,740,000 executions and 35 Ir each that is
   60.9M, 0.30%, against a tax already paid. **That arithmetic rests on the
   1,740,000, which is somebody else's measurement and was not re-derived
   here.**
5. **The fused ops dropped `Op::Store`'s `index` field and with it
   `debug_assert_names_the_clause`.** The run-time check that the clause is an
   `InstructionKind::Assignment` is retained and still returns
   `Loud::store_op_off_its_node`, so a fused op on the wrong clause fails
   loudly rather than writing a stranger's slot; what is gone is the debug
   tripwire that would have named it earlier.
6. **`Op::Binary` and `Op::Prefix` were deliberately left unfused.** They are
   the two remaining value ops that can end an assignment's region. Adding them
   is two more arms at roughly +1.5% of driver tax by the figures above, against
   whatever they remove -- and on the four axes measured here they would remove
   nothing, because no hot clause on any of them ends in either.

## Scratch

`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/storefusion/`
holds `gates/` with the six logs and the cold clippy run, the callgrind dumps
under `cg/` and the discarded out-of-line variant's under `cg-outline-variant/`,
`opcount.py` and `analyse2.py`, the four build trees and their disassembly, the
fifteen probes with their three-descriptor outputs, and
`prediction-arith.txt`.
