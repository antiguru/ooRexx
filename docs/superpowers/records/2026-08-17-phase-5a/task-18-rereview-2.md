# Task 18, fix rounds 2-3: re-review

Range `14479a7fa..9a0f249e9`. Everything below that says "measured" was run by me, fresh directory
per batch, absolute paths, three descriptors compared separately, both sides bounded.

**Verdict: CHANGES REQUESTED, on one number in the report and nothing else.** The digest section 13.3
offers as its restore witness matches no version of `lib.rs` anywhere in this range or its history.
Everything that digest exists to prove, I checked directly and it holds. **No code, test, corpus or
sitting change is implied** -- the fix is one line of prose.

**The work itself is clean.** I could not make the new checks refuse anything the oracle accepts
across thirty shapes built to break them, all four mutations reproduce exactly what the report
claims, the sole-instrument story is measured twice over, and the sitting's every cell is the TSV's
own value.

---

## The defect

**Section 13.3's restore witness is a digest of nothing in this range.** It reads *"`sha256` of
`lib.rs` `5c56baa1d375a9cfd3eb517fe2a8afdcba57a3aff28e28a4d5f1830621b629b0` before and after every
one"*. `crates/rexx-exec/src/lib.rs` is **`b256ab483217cbd3bb1011eecbb4f65ed4ce8f1762885ceddcbb4c6cf7202f31`**
at `ac2e92bf0`, at `0baa2ccae` and at `9a0f249e9` alike, and `60c5dbf29a9db3c652fb2718eaa8438a0f4df3858e2f80d2d13f7d7d593d74b0`
at the base `14479a7fa`. Searched: every version of `lib.rs` in its own history, every other file the
range touches at each of the four commits, and every tracked file under `rust/` at HEAD. `5c56baa1…`
matches none of them.

It is the one number whose whole job is to let a reader confirm the mutation runs were undone byte
for byte, so a digest that identifies no state cannot do it. **The state it was meant to witness is
in fact correct**: the tree is clean at `9a0f249e9`, `lib.rs` hashes to `b256ab48…`, and I reproduced
all four mutation outcomes from that source. Fix round 1's report quoted `5297769b…` for the same
file and that one did match, so this reads as a slip rather than a habit.

**Fix:** quote `b256ab483217cbd3bb1011eecbb4f65ed4ce8f1762885ceddcbb4c6cf7202f31`, or say which state
the mutations actually ran against.

---

## What I verified

### Over-refusal: 30 shapes, every one byte-identical on both engines

Two batches, oracle against `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, three descriptors
compared separately. **Nothing over-refuses.**

**Names shared across kinds and scopes, all rc 0 and all matching:** one name as a `::CLASS`, a
`::ROUTINE` and a `::RESOURCE` in one file; a member named like its own class (`::CLASS A` carrying
`::METHOD a CLASS`); a member, a `::CONSTANT` and an `::ATTRIBUTE` each named like a *different*
class in the same file; `::RESOURCE d` beside `::CLASS d`, beside `::ROUTINE d`, and beside
`::RESOURCE e`; `::RESOURCE d` beside a member `d` on a class; `::CLASS A` carrying `::ATTRIBUTE A`;
two `::CLASS` names differing only past a space (`"a b"` / `"a c"`); and the `END DONE` form of
`::RESOURCE` with distinct names.

**Refusals, each the oracle's own code and echoed clause:** `::CLASS "a b"` beside `::CLASS "A B"`;
`::CLASS A` beside `::CLASS A PUBLIC`; `::CLASS Array` beside `::CLASS array`; `::CLASS a.b` beside
`::CLASS A.B`; `::CLASS 1` twice; a pair separated by a `::ROUTINE` and a `::RESOURCE`; a pair
separated by two other classes; `::RESOURCE "D"` beside `::RESOURCE d`; the `END DONE` form beside
the `::END` form of one name; `::RESOURCE "a b"` beside `::RESOURCE "A B"`.

**The upcasing edge, which is where a byte-wise key could have diverged:** `::CLASS "é"` beside
`::CLASS "É"` is rc 0 on the oracle and rc 0 here, both engines -- `to_ascii_uppercase` and the
oracle's `upper()` leave the high bytes alone alike.

**Orderings, each the source order the single walk gives:** a duplicate member pair above a duplicate
`::CLASS` pair is 99.902; a `CLASS` keyword with no `::CLASS` above a duplicate `::CLASS` pair is
99.905; a duplicate `::RESOURCE` pair above a duplicate `::ROUTINE` pair is 99.942 and the blocks
swapped is 99.903; a duplicate `::CLASS` pair above a `::CLASS` naming an unresolvable superclass is
99.901.

**The C++ behind the keys, re-read.** `classDirective` builds `commonString(name->upper())`
(`parser/DirectiveParser.cpp:347`) and tests it at `:349`; `isDuplicateClass` is `:217`-`:220` and
asks `classDependencies`; `resourceDirective` builds the same at `:2277` and tests `resources` at
`:2316`. All exact, and all on the path their examples take. **Every `::CLASS` enters that table**:
`addClassDirective` (`:270`-`:275`) is called unconditionally at `:369` and is its only call site, so
the crate's unconditional insert is not wider than the oracle's.

### The two new corpus rows

Both match the oracle on all three descriptors under both engines: `class_duplicate_class.rex` at
rc 157 with `Error 99.901: Duplicate ::CLASS directive instruction.`, and
`class_directive_names_are_their_own_table.rex` at rc 0 printing `the class R` / `the routine R` /
`a second class`.

### The four mutations, run by me

Applied to the committed tree, corpus gate read, tree restored from a byte-identical copy, and
`target/release/rexx-run` rebuilt from the restored source. `lib.rs` back to
`b256ab483217cbd3bb1011eecbb4f65ed4ce8f1762885ceddcbb4c6cf7202f31` and `git status --porcelain` empty
after each.

| mutation | result | reddened |
|---|---|---|
| the `::CLASS` check removed | 203 of 204 | `class_duplicate_class.rex` alone |
| its key is the stored name, not the upcased one | 203 of 204 | `class_duplicate_class.rex` alone |
| one table for classes and routines | 203 of 204 | `class_directive_names_are_their_own_table.rex` alone |
| the `::RESOURCE` check removed | **204 of 204** | nothing in the corpus; `a_duplicate_resource_name_is_refused_and_a_distinct_one_is_not` FAILED |

Exactly the report's four results, and **no program that predates this round reddens under any of
them.** The second row is the one that matters most: it says the rewritten `::CLASS "a"` / `::CLASS A`
pair really discriminates the upcased key, which the first attempt's `::CLASS a` / `::CLASS "A"` pair
could not.

### The sole-instrument claim, measured twice

The mutation above is the first half: with the `::RESOURCE` check gone the whole differential stays
at **204 of 204** and only the in-crate test notices.

The second half I ran directly rather than taking on trust. Dropping a `::RESOURCE` program into
`corpus/lang/` makes `every_corpus_program_tiles` (`crates/rexx-parse/tests/tiling.rs:334`) fail it
byte by byte -- *"byte 'o' at offset 29 sits after the last clause span and belongs to no node"*,
then every remaining body and marker byte. Removing the file returns that test to `ok`. So the row is
impossible rather than merely absent, and **no corpus program carries a real `::RESOURCE`** -- the
two that match a `grep` mention it only in prose, one of them saying so explicitly.

### The `declared` first-wins comment

True on every path, not just the directive walk. `declared` has exactly one construction site,
`lib.rs:4118`-`:4123`; every other mention is a `&HashMap` parameter threaded into
`class_install_order`, `install_class_at`, `resolve_class_target` and `class_dependencies`. That site
is reached only after the first walk returns `Ok`, and the walk refuses a second `::CLASS` of one
upcased name before it can set `current_class` -- so reaching the map implies every name in it is
unique and `or_insert` can only insert.

### The sitting

`18-fixround-3` and `18-fixround-3-contribution`, both at `0baa2ccae`, both over all eight axes.
Every one of the sixty cells in section 13.5's two tables is the TSV's own value, checked
individually. Ten cells at or above 1% with `size` kept, and it is the same ten as fix round 1's.
The contribution arm's largest deviation is `rexxcps` `ir` at 1.000017, so "nothing above 0.002%"
holds.

**The bimodal cell is settled, and the absolutes say it rather than the ratio.** `dispatchclass` `tw`
`small` read 0.998019 in fix round 1 and reads 1.000002 here over a build carrying strictly more
code. The round-3 absolutes show why: `base` is median 13,096,101,315 with min 13,096,019,233 and max
13,096,135,435 -- all five runs in the low mode -- while `changed` is median 13,096,094,329 with max
13,155,989,166, one run in the high mode. Same two modes as fix round 1, opposite occupancy. That is
a bimodal sample, not a 0.2% effect.

I also confirmed the report's process claim: `global-constraints.md:37`-`:38` now carries the same
eight-axis command as the plan's `:313`-`:314`.

### Section 13.4's uncovered items

All three are stated as uncovered rather than implied, and the operative one is true: a check that
started refusing `::RESOURCE d` beside `::CLASS d` would pass every gate, because no corpus program
carries a `::RESOURCE` and the in-crate test declares no `::CLASS`.

**An observation, not a defect: the blindness is overstated, not understated.** "Six of the eight
have no corpus witness" undercounts what is covered -- besides the class/routine split, the control
also witnesses two distinct `::CLASS` directives, `class_init_activate_order.rex` carries `INIT` as a
member name in three different classes, and `class_constant_expression_later_class.rex` carries `C`
as a constant name in two. At least four of the eight have a corpus witness. Erring toward more
blindness costs nothing and hides nothing.

### Constraints

No `unsafe` and no `forbid` claim among the added lines; zero non-ASCII bytes in the added lines
under `rust/`; no historical framing; no comment names the size of a set.

## Gates, run by me at `9a0f249e9`, tree clean

```
cargo fmt --all --check                                              FMT_EXIT=0
cargo clippy --workspace --all-targets -- -D warnings                CLIPPY_EXIT=0, zero warning/error lines
REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast   EXIT=0
```

98 `test result: ok`, zero `FAILED`, zero `panicked`, **204 of 204 matching**.

## State the tree is left in

`git status --porcelain` empty at `9a0f249e9fee99446e68113d05ebf6e8f26df6c5`.
`crates/rexx-exec/src/lib.rs` restored from a byte-identical copy after each of the four mutations,
`sha256 b256ab48…7202f31` verified each time, and `target/release/rexx-run` rebuilt from the restored
source. The one file I added to `corpus/lang/` for the tiling probe was deleted by name and the tiling
test re-run to `ok`. `bench-baselines/phase-5a-arms.tsv` was not written to. Probe files live in the
scratchpad.
