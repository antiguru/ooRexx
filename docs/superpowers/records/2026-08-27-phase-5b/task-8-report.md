# Phase 5b Task 8 report: multidimensional `Array`, and `methodsbyclass` (D63)

BASE `1f57c3d02`, committed as **`dcd8b468d`**, working tree clean afterwards.
Written first and appended to as the work proceeded.

## Status

Complete. Committed, five gates green, both controls run. Gate statuses are section 10 and are
written only from status files read in the turn that committed.

---

## 1. What the brief claimed, re-measured

Every "verified by the controller" line was re-run. All four held.

* **The row is rc 120 with an `Array~NEW` refusal, stdout empty.** Measured on both engines:

  ```
  REXX_ENGINE=$E timeout -s KILL 20 target/release/rexx-run <abs>/mbc.rex
  ```
  `ir` and `tree-walker` both: `rc=120`, stdout `""`, stderr
  `rexx-exec: method "NEW" of class "Array" is not implemented (Phase 5)`.
  The oracle, standard wrapper from a fresh empty directory, is rc 0 with the four lines the brief
  quotes and empty stderr.

* **`Body::Array` was `Array(Vec<Option<ObjRef>>)` in `rexx-core/src/body.rs`.** Confirmed by
  reading the file; the change lands in `rexx-core` and crosses the crate boundary as the brief says.

* **`gate_table_c.rs`'s `methodsbyclass` row declared `oracle_lines: 4`** with the committed control
  the brief quotes. Confirmed by reading the file.

* **The plan's discriminator program answers exactly what the plan states.** Measured, oracle,
  standard wrapper, fresh empty directory: rc 0, empty stderr,
  `x a12 a21 b23` / `y 2 2 3` / `z 6 3`.

**The blast-radius warning was directionally right and the ratio was not.** Counted at BASE over
every tracked `.rs` file, `Body::Array` appears on **36** lines and `Body::Array(` -- a construction
or a pattern -- on **32** of them. The compiler enumerated 32 sites and every one needed changing:
`From` and `Deref` absorbed nothing here, because there is no such impl on `Body`. The four
remaining lines are all prose -- three doc comments and one ordinary comment. So the grep
over-counted by the prose mentions and by nothing else.

---

## 2. The finding that changes the task: the plan's stated discriminator does not discriminate

The plan says, of the asymmetric second cell:

> `m[1,2]` and `m[2,1]` are distinguishable only under the right mapping

**That is false, and it is false for the reason the paragraph directly above it already gives.** A
transposed mapping transposes the write and the read alike, so `m[1,2]` and `m[2,1]` swap places
*together* and each still reads back what was written to it. Nothing built out of a write and its
matching read can see a relabelling the two share.

**Measured rather than argued.** With the crate's own `multi_dimension_position` accumulating the
offset in reverse -- row-major where the oracle is column-major, the whole of the mutation being
`.enumerate()` -> `.enumerate().rev()` -- the plan's discriminator program is **byte-identical to the
oracle on all three descriptors, on both engines**:

```
diff.sh <dir>        # oracle wrapper vs REXX_ENGINE=ir and =tree-walker, three descriptors
ALL AGREE (1 programs, both engines)
```
crate stdout `x a12 a21 b23` / `y 2 2 3` / `z 6 3`, identical to the oracle's.

**What does discriminate is a reader of the slots in their own order.** `~toString('l', ' ')` is
one. Measured, oracle rc 0: a 2 by 3 array with `m[i, j]` set to `i || j` at every cell renders
`11 21 12 22 13 23` -- the **first** subscript moves fastest, which is
`ArrayClass::validateMultiDimensionIndex`'s `offset += multiplier * (position - 1); multiplier *=
dimension;` (`interpreter/classes/ArrayClass.cpp:1408`-`:1410`). Three dimensions confirm it:
`.array~new(2,3,4)` with `(2,1,1)`, `(1,2,1)` and `(1,1,2)` set renders them in that order.

**The plan is corrected in place** (`docs/superpowers/plans/2026-08-27-phase-5b.md`, Task 8), with
the false sentence replaced by the measurement and by what the probe now has to carry. The `Done
when` line there gained "and a reader of the slots in their own order". The spec's D63 says nothing
about the discriminator and needed no change.

---

## 3. What was built

### Representation (`rexx-core`)

`Body::Array` became a struct variant:

```rust
Array {
    slots: Vec<Option<ObjRef>>,
    dimensions: Option<Box<[usize]>>,
},
```

mirroring `ArrayClass::dimensions` exactly, including the case that is easy to lose: **a
one-element dimensions array is still single-dimensional** (`isMultiDimensional()` is
`dimensions != OREF_NULL && dimensions->size() != 1`, `classes/ArrayClass.hpp:312`) and its element
is never read. `.array~new(0)` is the shape that has one -- measured, its `~dimension` is `1` where
`.array~new()`'s is `0`, and it refuses `a[2,3] = 'x'` at 93.926 where the other answers it.
`Body::array(slots)` is the single-dimensional constructor every existing caller now uses.

`Body`'s width assertion (`<= 80`) still holds; the variant is 24 + 16 bytes of payload.

### Methods (`rexx-exec`)

New `NATIVE_METHODS` rows `Array~"[]="`/`Array~PUT` (`ArrayClass::putRexx`) and `Array~DIMENSION`;
new `NATIVE_CLASS_METHODS` row `Array~NEW` (`ArrayClass::newRexx`). `Array~"[]"`/`Array~AT` extended
from one subscript to a validated subscript list.

The index machinery is a port of `validateIndex` and its two halves, plus `extend`, `extendMulti`
and `createMultidimensional`, with `IndexUse::Get`/`Put` standing for `IndexAccess`/`IndexUpdate`.

### The refusal surface, measured before it was written

Every one of these is an oracle transcript taken under the standard wrapper from a fresh empty
directory, then reproduced byte for byte by both engines:

| shape | oracle |
| --- | --- |
| `m[1]` on 2x3 | 93.925 `Not enough subscripts for array; 2 expected.` |
| `m[1,2,3]` on 2x3 | 93.926 `Too many subscripts for array; 2 expected.` |
| `a[1,2]` on a list array | 93.926 `... 1 expected.` |
| `m[1,0]`, `m['x',1]`, `m[1.5,1]`, `m[.array,1]` | 93.924 `Invalid position argument specified; found "..."` -- **no argument position, and the argument's own text, not its converted value** |
| `a[0]` on a list array | 93.907 `Method argument 1 must be a positive whole number; found "0".` |
| `a~put('v',0)` | 93.907, **argument 2** |
| `m[,2]` | 93.903 `argument 2 is required` |
| `m[,2] = 1` | 93.903 `argument **3**` -- `putRexx` passes `ARG_TWO`, so the list starts one later |
| `m~put('v')` | 93.901 `Not enough arguments for method; 2 expected.` |
| `a~put(,1)` | 93.903 `argument 1 is required` |
| `.array~new(-1)`, `.array~new('x')` | 93.906 `Method argument 1 must be zero or a positive whole number; found "..."` |
| `.array~new(2,'-1.0')` | 93.906, **argument 2**, `found "-1.0"` -- the argument, not the conversion |
| `.array~new((,3))` | 93.903 `argument 1 is required` |
| `.array~new(100000000000000001)` | 93.959 `An array cannot contain more than 100000000000000000 elements.` |
| `m~dimension(0)` | 93.907 argument 1 |
| `m~dimension(1,2)` | 93.902 `Too many arguments in invocation of method; 1 expected.` |

Two error constructors are new (`Raised::not_enough_subscripts`, `Raised::array_too_big`); 93.906,
93.903, 93.924, 93.907, 93.901 and 93.926 reused existing ones. **93.902 is raised by nothing this
task wrote** -- it comes from `Arity::Fixed(1)`'s own count check on the `DIMENSION` row, which is
what makes `m~dimension(1,2)` that error rather than one of the body's.

### The answering surface

`.array~new()` / `(0)` / `(n)` / `(d1, d2, ...)` / `((d1, d2))`; `~dimension` and `~dimension(n)`;
`[]`/`AT`/`[]=`/`PUT` with a subscript list or a lone array spread as one; a read past a dimension
answering `.nil`; a write past one reshaping the array in that dimension and moving what is already
there; an array with no dimensions array taking its shape from the first multidimensional subscript
list written through it; and the single-dimension `[]=`/`PUT` that extends instead.

---

## 4. The probe, and why it has the line it has

```rexx
matrix = .array~new(2, 3)
matrix[2, 3] = 0
matrix[1, 2] = 'a12'
matrix~"[]="('a21', 2, 1)
matrix~"[]="('z23', 2, 3)
say 'element' matrix[2, 3] matrix[1, 2] matrix[2, 1]
say 'order' matrix~toString('l', ' ')
say 'dimensions' matrix~dimension matrix~dimension(1) matrix~dimension(2)
say 'methods' matrix~hasMethod("[]") matrix~hasMethod("[]=")
say 'environment' .array~id .directory~id .stringtable~id
```

Oracle, rc 0, empty stderr, **five** lines:

```
element z23 a12 a21
order a21 a12 z23
dimensions 2 2 3
methods 1 1
environment Array Directory StringTable
```

* The **equivalence** arm is the *overwrite*: `matrix[2, 3] = 0` then `matrix~"[]="('z23', 2, 3)`,
  read back through the operator form. A message form that reached a different cell would leave `0`
  there. Both spellings of the setter are witnessed -- the operator form by `matrix[1, 2] = 'a12'`
  being read back, the message form by the overwrite.
* The **asymmetric second cell** is `a12` at `[1,2]` against `a21` at `[2,1]`.
* The **`order` line** is the only thing in the probe that can see which cell a subscript names, for
  the reason in section 2.
* `oracle_lines` moved 4 -> 5 in the same commit.

## 5. Both controls, run, with transcripts

Read under the constraints file's invocation:

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_c --test gate_table_d --no-fail-fast
```

**Clean tree: `exit=0`.** Table C `5b: 6 rows, 0 not yet agree`; table D `5b: 2 rows, 0 not yet
agree`; `gated by this run: 0 row(s)` in both tables. `method "NEW" of class "Array"` is gone from
the loud-rows-waiting-on list.

**Control 1, the row's own committed control** -- `native_array_put` takes only the first subscript
as a flat index, which is "route `matrix[2, 3] = 0` to a single-index `[]=`":

```
exit=101
5b: 6 rows, 1 not yet `agree`
gated by this run: 1 row(s) whose owning phase is closing or closed and whose verdict is not `agree`
  diverge-stdout loud=no  5b   methodsbyclass ...
      oracle rc=0  out="element z23 a12 a21\norder a21 a12 z23\n..."
      crate  rc=0  out="element The NIL object The NIL object z23\norder a12 z23\n..."
```

**Control 2, transposing the index mapping** -- `multi_dimension_position`'s subscript/dimension
walk reversed, the whole mutation being `.enumerate()` -> `.enumerate().rev()`:

```
exit=101
5b: 6 rows, 1 not yet `agree`
gated by this run: 1 row(s) ...
  diverge-stdout loud=no  5b   methodsbyclass ...
      oracle rc=0  out="element z23 a12 a21\norder a21 a12 z23\n..."
      crate  rc=0  out="element z23 a12 a21\norder a12 a21 z23\n..."
```

**The second transcript is the whole argument of section 2 in one line:** the `element` line is
*identical* under a transposed mapping and only the `order` line moves. A probe without the `order`
line would have read `agree` against a build that indexes the array the wrong way round.

Under Control 2 the plan's own discriminator program is `ALL AGREE (1 programs, both engines)`
against the oracle, while the shipped probe diverges -- both measured in the same build.

**Control 1 also settles the BASE count the brief states.** A single red `methodsbyclass` produces
exactly `5b: 6 rows, 1 not yet agree` in table C, and the row was measured rc 120 against oracle rc 0
at BASE, which is not-agree by any reading. I did not run the phase gate at BASE itself; that count
is an inference from the control's transcript plus the BASE row measurement, not a direct reading.

Each control was applied to a scratchpad-backed copy of `dispatch.rs` and restored from that copy
(`sha256` re-checked equal after each restore, and the mutation string grepped for and absent).

## 6. Differential coverage

`diff.sh` runs each program under the oracle wrapper from a fresh empty run directory and under
`REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, comparing stdout, stderr and exit status as three
separate files. Final run, on the committed build:

```
d1    ALL AGREE (1 programs, both engines)     the probe alone
d2    ALL AGREE (21 programs, both engines)    the answering surface
d3    ALL AGREE (33 programs, both engines)    the refusal surface
d4    ALL AGREE (1 programs, both engines)     the plan's discriminator
d5b   ALL AGREE (1 programs, both engines)     array_multidimensional.rex
d6    ALL AGREE (1 programs, both engines)     array_multidimensional_refusals.rex
d8    ALL AGREE (1 programs, both engines)     the committed probe
d10   ALL AGREE (1 programs, both engines)     the zero-size extension shape
```

## 7. Corpus

`corpus/phase-5b.txt` and `EXPECTED_SUBSET_5B` gained, in the same commit:

* `gate-tables/concepts/methodsbyclass.rex` -- the row this task makes agree, per the constraints
  file's rule that a probe moves into the phase file in the task that makes its row agree;
* `lang/array_multidimensional.rex` -- the shapes `~new` builds, the reshape a write past a
  dimension performs in each dimension, an array taking its shape from the first subscript list it
  is written through, and the single-dimension `[]=`;
  * its `grew` line exists to **assert a property the body's comment would otherwise only
    describe**: `.array~new(0)` carries a one-element dimensions array whose entry is not the
    extent, so extending it leaves `~dimension` answering `1` and `~dimension(1)` answering the new
    size. Measured, oracle rc 0: `zero~put('v', 3)` then
    `zero~size zero~items zero~dimension zero~dimension(1) zero~dimension(2) zero[3]` is
    `3 1 1 3 0 v`.
    **Mutation-checked, and it is the only witness that catches it**: dropping
    `native_array_dimension`'s `dimensions.len() != 1` guard so the entry *is* read makes that line
    `3 1 1 0 0 v` and reddens this program, while `d2`, `d3`, `d6` and `d8` -- the 56 other
    programs, both engines -- all stay `ALL AGREE`. Restored from a scratchpad copy afterwards and
    the sha256 checked against the pre-gate tree hash's own entry for `dispatch.rs`.
    This row was added after the first gate run had started; that run was stopped, the row added,
    and every gate re-run from the start on the tree that is committed.
* `lang/array_multidimensional_refusals.rex` -- the error surface, trapped rows reported by number
  with an untrapped tail for one full text.

Both `lang/` programs gained a `crates/rexx-parse/tests/sourceline_oracle/<name>.txt`, generated by
the driver in that test's own module comment (counts 72 and 99, matching the files' line counts).
Neither contains a CR or a `CTRL-Z` and both end with a newline, which is what that test asserts.

**The corpus cannot see a substituted argument position**, because `condition('A')` is not
implemented in this crate (`rexx-exec: CONDITION option "A" answers an Array or .NIL, which is not
implemented`) and `condition('D')` is empty for SYNTAX on the oracle -- measured both ways. The
trapped rows therefore compare error *numbers*, which is the existing corpus idiom
(`array_index_refusals.rex`, `object_send_refusals.rex`). Two unit tests carry what the corpus
cannot: `a_subscript_refusal_names_the_position_its_own_method_counts_from` asserts the six
substituted texts above, and `new_on_a_subclass_of_array_is_loud` covers the one refusal this task
adds that has no oracle counterpart.

---

## 8. Divergences and gaps -- what I did NOT do

* **`.Array~subclass('K')~new(2, 3)` is a loud refusal here and answers on the oracle.** Measured,
  oracle rc 0, `~size` is `6`; this crate is rc 120 with
  `rexx-exec: ~new on a subclass of Array is not implemented (Phase 5)`, stdout carrying the `~id`
  line so the refusal is `~new`'s and not `~subclass`'s. The answer would have to dispatch against
  the subclass's behaviour and a `Body::Array` carries none -- adding a class handle to that variant
  is a representation decision beyond this task. Covered by `new_on_a_subclass_of_array_is_loud`.
* **`Array~dimensions` (plural) stays a loud refusal.** `getDimensionsRexx` is a neighbour of what
  this task built and is not needed by the row; leaving it loud is deliberate, and the differential
  batch that found it is recorded so the next task has the measurement:
  `.array~new(2,3)~dimensions` is oracle rc 0 printing `2` then `3`.
* **`extendMulti`'s product is bounded here and is not in the C++.** `createMultidimensional` raises
  93.959 for a product past `MaxFixedArraySize` (`classes/ArrayClass.cpp:217`-`:220`) and
  `extendMulti` has no such check, so the C++ would wrap. This crate raises the same 93.959 in both
  places. Reaching it needs dimensions whose product exceeds 10^17, where the oracle's own answer is
  an allocation it cannot make; **not measured against the oracle**, and stated here rather than
  claimed to agree.
* **A subscript list that is a lone array with a leading empty slot is loud on the
  single-dimensional path** (`Loud::array_index_hole`, unchanged) and takes the 93.925/93.926 count
  check on the multidimensional path, which is where the C++ puts it -- `validateIndex` spreads
  before it chooses the path, and only `validateSingleDimensionIndex` dereferences the hole. **The
  multidimensional arm is not measured against the oracle and will not be**: it is one step from
  `corpus/oracle-crashes.txt` entry 6 and the reasoning is worth less than the risk. It follows the
  C++ by construction.
* **I did not run the phase gate at BASE.** Section 5 says what the BASE count rests on instead.
* **The `Array` row set in gate table C was not re-derived**; I read the per-phase counts the tables
  print, and the only 5b row I changed is `methodsbyclass`.
* **No performance sitting.** This task lands code in `rexx-exec/src` and `rexx-core/src`, which the
  5a constraints file's performance guard would cover; the 5b constraints file does not carry that
  guard, and project memory records the 5a pin as retired. **Not run, and not claimed.**

## 9. Citation audit

Every `interpreter/` line number this task added was read back with `sed -n "Np" <file>` against the
worktree's own `interpreter/` (byte-identical to `/home/moritz/dev/repos/ooRexx/interpreter/` for
`ArrayClass.cpp` and `ArrayClass.hpp`, checked with `cmp`). **Three were wrong and are fixed:**

* `newRexx`'s zero-size branch cited as `:124`-`:127`; the `if (totalSize == 0)` is at `:125` and the
  block ends at `:128`.
* `extendMulti`'s two `positionArgument(index[i], i + 1)` calls cited as `:2456` and `:2513`; they are
  at `:2457` and `:2515` -- both citations landed on the comment line above the call.

The other twenty-odd resolve to the function, macro row or statement they name.

**One pre-existing citation was wrong and is corrected**: `dispatch.rs`'s `Array~"[]"`/`AT` comment
named `array_index.rex` as a corpus program. No such file exists; the positive counterpart of
`array_index_refusals.rex` is `array_list_expression.rex`, which is what it now names.

`cargo doc -p rexx-exec -p rexx-core --no-deps` exits 0 with no broken intra-doc link; the two
`redundant explicit link target` warnings are in `lib.rs:405`-`:406` and predate this task.

---

## 10. Gates

Every status read from a file written by the run itself, unpiped, in the turn that commits. The run
was started once, interrupted deliberately to add the `grew` row of section 7, and **restarted from
the beginning**; the figures below are from the restarted run alone.

| gate | command | status |
| --- | --- | --- |
| 1 | `cargo fmt --all --check` | `exit=0` |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | `exit=0` |
| 3 | `cargo test --release --workspace` | `exit=0`, 1936 passed, 0 failed |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace` | `exit=0`, 1936 passed, 0 failed |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | `exit=0`, 1937 passed, 0 failed |

Gate 4's corpus harness (`tests/corpus.rs`, over
`phase-4a.txt + phase-4b.txt + phase-4c.txt + phase-5a.txt + phase-5b.txt`) prints **326 of 326
matching** in STRICT mode. `memcap` was present (`/home/moritz/.local/bin/memcap`), so gate 5 is the
`memcap` form and not the `ulimit` substitute.

**Phase gate**, the constraints file's own invocation:

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_c --test gate_table_d --no-fail-fast
exit=0
```

* table C: `5a: 135 rows, 0 not yet agree` / **`5b: 6 rows, 0 not yet agree`** / `5c: 1347 rows, 868`
* table D: `5a: 36 rows, 0` / **`5b: 2 rows, 0`** / `5c: 38 rows, 35` / `7: 1 row, 1` /
  `deferred-parse-error-rendering: 2 rows, 2`
* `gated by this run: 0 row(s)` in both tables.

**Tree hash across the gate window.** `sha256sum` over `git rev-parse HEAD` plus the **bytes** of
every tracked-modified and every untracked non-ignored path
(`git ls-files -m -o --exclude-standard`), taken before gate 1 and again after the phase gate:
`b2ae00abe15935f5f10c3961f01a17c795327bbb63f7ff707a41c47e9cbf31bc` both times, and the two path
listings `diff` empty. **What it does not cover, stated rather than implied**: paths git ignores --
`target/`, `build/`, and `.superpowers/`, so this report file itself is outside it -- and the bytes
of tracked files that match HEAD, which would enter the listing the moment they stopped matching.

