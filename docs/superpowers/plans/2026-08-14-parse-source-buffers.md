# `PARSE`'s source strings, and the two allocations every clause makes

**Goal:** remove the per-clause allocations `Interp::parse_strings` makes, which are the largest identifiable allocation cause left in `samples/rexxcps.rex`.

**Status:** open, not started. Base `3aba1cacd`. Entry 56 of `phase-4f-record.md` took the smaller half of `PARSE` (the trigger operand) and named this as the larger one.

## What is measured, and by whom

`heaptrack` on the pinned `rexxcps` (see "Tooling" below for what "pinned" means and why it is not optional), at `7ecd6f510`: 539,048 allocations, of which `exec_parse` accounted for about 123,000 -- **95,205 through `parse_strings`** and 28,001 through `apply_trigger`.
Entry 56 removed the second number. At `3aba1cacd` the program allocates 505,447 and `parse_strings` is still about 95,000 of it.

For scale, the whole session's other work took the same program from 695,620 to 505,447.

## The two allocations

`rust/crates/rexx-exec/src/parse_template.rs`, `parse_strings` (near line 459) and `exec_parse` above it.

**One: an owned copy of the source string.** Every arm ends in `to_vec()` or builds a fresh `Vec`:

```rust
ParseSource::Value(Some(expression)) => { ... ("VALUE", self.to_text(value).to_vec()) }
ParseSource::Var(id)                 => { ... ("VAR",   self.to_text(value).to_vec()) }
```

**Two: the one-element `Vec` that carries it.**

```rust
Ok(vec![value])
```

`exec_parse` then does `self.parse_strings(...)?.into_iter()` and `next_template` pulls owned `Vec<u8>`s out of that iterator into a `Cursor`.

## Why the copy exists, which is the constraint the fix has to respect

`Cursor` owns its `string: Vec<u8>` (`parse_template.rs:119`) and lives for the whole clause. Between creating it and dropping it, `apply_trigger` and `assign_targets` call back into `&mut self` -- evaluating operand expressions, writing targets, tracing. **A borrow of the heap's bytes cannot be live across those calls**, which is why the source is copied out rather than read in place.

That is also why entry 56's trigger fix worked and this one is harder: a trigger's operand is read and finished with *before* the next `&mut self` call, so it could borrow; the parse source cannot.

## Two candidate changes

**A: lend the source copy.** Take the buffer from a pool, hand it to the `Cursor`, and give it back at the end of `exec_parse`. Needs `Cursor` to expose its `Vec` (`fn into_string(self) -> Vec<u8>`), and needs a pool that is not the builtin result buffer -- a `PARSE` clause holds its source for the whole clause, and a trigger's operand expression can call builtins, so borrowing `result_buffer` for that long would make every such builtin allocate instead. A dedicated `parse_buffer` field, in the shape `key_buffer` already uses.

**B: lend the outer `Vec`.** `exec_parse` takes a `Vec<Vec<u8>>` from a pool, `parse_strings` fills it rather than returning one, and `next_template` walks it by index with `std::mem::take` on each slot instead of consuming it through `into_iter`. Returning it at the end keeps the outer allocation across clauses.

**B also opens the door to keeping the inner capacities**: if the slots are `mem::take`n and the `Cursor`'s buffer is put *back* into its slot when the cursor is superseded, both allocations disappear across clauses rather than one. That is the version worth aiming at, and it is also the one most likely to get the lifetime of a comma-fenced template wrong -- `next_template` is called once per fence, so a clause can build several cursors.

## The traps, all of them measured this session

* **The exchange rate is about 45 instructions per allocation** (entry 50, from `strings`; entry 49 gives 138 for a different shape). 95,000 allocations on this program is worth roughly a percent of it, not more. **Do not assume a large allocation fall is a large instruction fall.**
* **A change can remove allocations and cost time.** Entry 52 held the division's working remainder in the inline `Digits` type: it removed *every* allocation division makes and cost **+3.104%** on `arith`, because `push` and `as_mut_slice` branch on which arm holds the value, inside a per-digit loop. It was measured, rejected, and the reason is a comment beside the `Vec` it did not replace. A pooled buffer that adds a branch to a hot loop can lose the same way.
* **Measure the halves separately.** Entry 52 shipped one half and rejected the other only because both were built and measured alone. If A and B are both attempted, measure A, B and A+B.
* **Probe with tracing on and off.** Entry 56's gate was `trace_mode().results`; the probe was run plain and under `TRACE R` because that gate is exactly the difference between them. Any gating here needs the same.
* **`PARSE ARG` is the only multi-string source** and it returns early, before the `vec![value]` line. It has its own `Vec::with_capacity(arguments.len())` and its own `to_vec()` per argument. Do not assume the single-string path covers it.
* **An axis that executes no `PARSE` must come out a bound.** `strings`, `arith`, `emptyloop`, `varlookup` and `compound` are the controls; entry 56 got all five.
* **The do-nothing control still does not exist.** Entries 47, 48 and 55 each recorded an axis moving outside its own spans with no mechanism on that axis -- `arith` twice, `emptyloop` once, at 4.3M, 1.0M and 50M instructions. Until a control that changes nothing is measured across the same axes, a movement of that size is not attributable. **This is the most useful measurement not yet made, and it should probably be made before this task rather than after.**

## Verification, in the order it has to happen

1. `cargo fmt --all --check`, then `cargo clippy --all-targets --all-features -- -D warnings`, then `cargo test --release --workspace`. Never from the repository root; never read a diagnostic list through `head`; never read an exit status through a pipe.
2. The `PARSE` probe from entry 56, run **twice**, plain and with `trace r` inserted, each compared to the oracle stream by stream. It covers a string pattern, a caseless one, absolute, relative both ways, both length forms, a parenthesised variable operand, the placeholder period, a comma fence, `UPPER`/`LOWER`, `PARSE VALUE`/`VAR`/`SOURCE`/`VERSION`/`ARG` with an omitted argument, and the 26.4 raise.
3. A mutation witness: break the change deliberately and confirm the probe fails. Restore with `cp` from a backup, `touch`, `sha256sum -c`, and **rebuild** before believing any later number.
4. `heaptrack` on the pinned `rexxcps`, before and after.
5. Five interleaved rounds per arm on every axis, both arms staged at one fixed binary path, minimum of each, spread reported beside the difference.
6. Append an entry to `phase-4f-record.md`. Never rewrite an earlier one; check with `git diff --numstat` that the deletion count is 0.

## Tooling notes worth not rediscovering

**`samples/rexxcps.rex` adjusts its own workload to the speed of the interpreter running it** (`count=(1%total + 1) * count`, inside `do trial=1 to 2`). Under `heaptrack` three identical runs gave 2,775,674 / 2,775,676 / 2,082,427 allocations. **Pin it** by replacing `do trial=1 to 2` with `do trial=1 to 1`, keeping `count`/`averaging` at 20; that gives 695,620 at entry 48's base, twice. At the default 100/100 the first trial takes over a second so the adaptation never fires, which is why the *instruction* axis is unaffected -- checked, not assumed. Entry 48 has the full account.

**Attributing a `heaptrack` profile to a function** needs the folded export plus a mangled-name reader, because the textual report's backtraces are unreadable at this depth:

```
heaptrack_print --flamegraph-cost-type allocations -F out.folded -p 0 -a 0 -T 0 -f heaptrack.*.zst
```

then, per stack, take the deepest frame belonging to `rexx_num`/`rexx_exec`/`rexx_core`/`rexx_parse` whose v0-mangled name yields a readable identifier -- parsing length-prefixed segments, and skipping the allocator plumbing (`with_capacity`, `try_allocate_in`, `finish_grow`, `grow_amortized`, `from_elem`, `to_owned`, `to_string`, `to_vec`, `clone`, `Global`, `RawVec`). Without that skip list the top bucket comes back 31% unresolved and the real leaders are hidden underneath it.

**The oracle** is read-only, wrapped, and run from a fresh empty directory with absolute paths:

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx /abs/path/FILE )
```

Compare **stream by stream**, never merged: trace goes to stderr and `SAY` to stdout, and a merged comparison shows lines moved rather than changed. That produced one false alarm in entry 47.

## Still open elsewhere, for whoever picks the next thing

* **`POS` bounds a match differently from the oracle.** `pos('an', 'banana bandana abracadabra', 6, 4)` is 9 there and 0 here: the oracle bounds where a match may *begin*, this crate requires the match to fit. Pre-existing, unfiled, entry 54. `find_backward`'s own doc carries the *opposite* rule for `LASTPOS`, measured -- so the two builtins differ and only one is right.
* **`rexx_num::format::render_integer_padded`** builds three `String`s to render one integer and the caller copies the result again -- 9.4% of the pinned `rexxcps`' allocations. `rexx-num` has no `Interp` to lend from, so it wants a caller-provided buffer, which is the same design question entry 51 left open for division's working values.
* **`apply_binary`** is 13.5% of that program's allocations and has never been examined.
