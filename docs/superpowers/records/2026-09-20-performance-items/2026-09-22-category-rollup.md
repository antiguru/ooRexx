# The inclusive category rollup, ours against the oracle, 2026-09-22

Read-only. Nothing in the repository was edited and nothing was committed; the
measuring worktree is
`/home/moritz/dev/repos/ooRexx-rust-rewrite/.claude/worktrees/agent-a5014588856118785`,
reset to `c418d4555` (confirmed `c418d4555` on top of `f3974036a` on top of
`3b850d885`). Scripts and derived tables are in the session scratchpad under
`catrollup-2026-09-22/`.

Both dumps are the ones the brief names; neither was re-taken.

    sha256 cg-instr.out             7c6977fac7328e25e319584bfc9de70ec51bf1e23afc5447b360659a60c9b199
    sha256 cg-oracle-rexxcps.out    7b9262b75297587e269242aa31f046e0217bb2384937ef5239700c5c8680c89d

## The table

Self cost per function, rolled up by work category. **Each column sums to its
own run's `summary:` line, difference 0** — see "The partition check" below.
20,000,000 clauses on both sides.

| category | ours Ir | ours % | ours /cl | oracle Ir | oracle % | oracle /cl | diff /cl |
|---|---:|---:|---:|---:|---:|---:|---:|
| interpretation machinery | 8,079,281,481 | 39.82% | 403.96 | 1,965,821,096 | 18.39% | 98.29 | **+305.67** |
| variable access, simple | 309,210,422 | 1.52% | 15.46 | 472,701,216 | 4.42% | 23.64 | **-8.17** |
| variable access, compound | 1,726,330,288 | 8.51% | 86.32 | 990,262,146 | 9.26% | 49.51 | +36.80 |
| arithmetic and comparison | 1,982,620,116 | 9.77% | 99.13 | 1,370,983,215 | 12.83% | 68.55 | +30.58 |
| text/number conversion, string building | 2,167,100,799 | 10.68% | 108.36 | 939,448,091 | 8.79% | 46.97 | +61.38 |
| PARSE | 1,879,081,112 | 9.26% | 93.95 | 1,046,086,894 | 9.79% | 52.30 | +41.65 |
| built-in functions | 757,282,604 | 3.73% | 37.86 | 291,525,442 | 2.73% | 14.58 | +23.29 |
| call and argument handling | 831,816,009 | 4.10% | 41.59 | 604,600,919 | 5.66% | 30.23 | +11.36 |
| allocation and collection, the interpreter's own | 955,310,696 | 4.71% | 47.77 | 2,219,050,185 | 20.76% | 110.95 | **-63.19** |
| the C allocator | 900,376,297 | 4.44% | 45.02 | 71,570 | 0.00% | 0.00 | +45.02 |
| libc string and memory | 643,083,007 | 3.17% | 32.15 | 771,870,242 | 7.22% | 38.59 | **-6.44** |
| everything else | 60,348,433 | 0.30% | 3.02 | 16,428,199 | 0.15% | 0.82 | +2.20 |
| **total** | **20,291,841,264** | **100.00%** | **1014.59** | **10,688,849,215** | **100.00%** | **534.44** | **+480.15** |

The diff column sums to **480.15**, which is the whole gap; both checks are
printed by `table.py` and are assertions in it, not hand arithmetic.

### Where we are ahead

Three categories, and two of them are large.

* **allocation and collection, -63.19/cl.** The oracle spends 20.76% of its run
  there; we spend 4.71%.
* **libc string and memory, -6.44/cl** — ours 32.15, theirs 38.59.
* **variable access, simple, -8.17/cl.** Read the caveat: this one is an
  artifact, see "Two rows that are not like-for-like".

**All-in allocation** (our GC category plus the C allocator, against theirs)
is **92.78/cl ours against 110.96/cl theirs, -18.17/cl in our favour** — and
theirs is understated, because 203,184,750 Ir (10.16/cl) of the `__memset` in
its libc row is `MemoryObject::newObject` zeroing fresh objects. Counting that
where the work is, the oracle's allocation is 121.1/cl against our 92.8.

### Which categories account for the 480.2

Ranked, behind first:

    +305.67  interpretation machinery          (63.7% of the gap)
     +61.38  text/number conversion, string building
     +45.02  the C allocator
     +41.65  PARSE
     +36.80  variable access, compound
     +30.58  arithmetic and comparison
     +23.29  built-in functions
     +11.36  call and argument handling
      +2.20  everything else
      -6.44  libc string and memory
      -8.17  variable access, simple
     -63.19  allocation and collection

**Interpretation machinery is almost two thirds of the gap on its own**, and
nothing else is a quarter of it. Decomposed:

| ours | Ir | /cl |
|---|---:|---:|
| `run_ops_from` x3 | 5,043,544,989 | 252.18 |
| loop machinery | 1,104,464,279 | 55.22 |
| operator dispatch (`apply_binary` and friends) | 1,044,202,242 | 52.21 |
| remainder | 887,069,971 | 44.35 |
| **total** | **8,079,281,481** | **403.96** |

| oracle | Ir | /cl |
|---|---:|---:|
| `evaluate` family + `callOperatorMethod` | 637,165,689 | 31.86 |
| `RexxInstruction*::execute` family | 538,655,765 | 26.93 |
| `RexxActivation::run` x2 | 533,966,788 | 26.70 |
| loop machinery (`DoBlock`/`ControlledLoop`/`ForLoop`) | 168,509,035 | 8.43 |
| remainder | 87,523,819 | 4.38 |
| **total** | **1,965,821,096** | **98.29** |

Our driver alone, at 252.18/cl, is **2.57x the oracle's entire machinery
category**. Our loop machinery alone (55.22) is more than twice the oracle's
whole `RexxActivation::run` (26.70).

## Caveats

### The partition check

`parse_cg2.py` sums every cost line that is **not** the line immediately
following a `calls=` line (that one carries a call's inclusive cost and belongs
to the callee). Run as `python3 parse_cg2.py <dump> <out>`:

    summary 20291841264  selfsum 20291841264  diff 0  fns 1150     rc=0   (ours)
    summary 10688849215  selfsum 10688849215  diff 0  fns 1733     rc=0   (oracle)

`classify.py` asserts the category sums equal those totals; `classify ours
rc=0`, `classify oracle rc=0`, `table rc=0`. A second, independently written
parser (`parse_cg.py`) produced byte-identical cost columns on both dumps
(`diff <(cut -f1 ours-self.tsv) <(cut -f1 ours-fn.tsv)` empty, same for the
oracle).

### Two rows that are not like-for-like

1. **`variable access, simple` — our -8.17 is an artifact, not a win.** Our
   simple-variable *read* is a real function (`read_at`, 309,209,311) but our
   simple-variable *write* has no function of its own: it is inlined into the
   driver's `Op::Store` arm and is charged to interpretation machinery. The
   oracle's write side is `RexxSimpleVariable::assign`, 210,252,426 = 10.51/cl,
   and it sits in this row. Move it to machinery and the two rows read
   **VSIM ours 15.46 against theirs 13.12, +2.34** and **MACH +295.16**. I did
   not move it, because I cannot measure our inlined half without per-address
   work the brief did not ask for; take the row as read-only-ours against
   read-and-write-theirs.

2. **Operator dispatch is the largest judgement call.** `apply_binary`
   (874,321,446) is a dispatcher — it picks the operator receiver, then
   delegates to concat / compare / logical — so it is machinery by the rule
   below, matching `RexxBinaryOperator::evaluate` and
   `RexxObject::callOperatorMethod` on the other side. Moving operator dispatch
   to `arithmetic and comparison` on **both** sides gives
   **MACH ours 351.75, theirs 74.60, +277.15** and
   **ARITH ours 151.34, theirs 92.24, +59.10**. Machinery stays the headline
   either way.

### The two asymmetries, stated and not corrected for

* The oracle is a **shared library** at `-O2 -g -DNDEBUG`: its cross-library
  calls go through the PLT and it gets no cross-TU inlining. We are a static
  binary at `-O3` with fat LTO and one codegen unit. **Both favour us**, so
  480.15/cl understates the structural gap rather than overstating it.
* `build/` is `-O2`. **No ratio taken from the oracle run is a sanctioned
  figure**, this table's included. What it supports is shape, not magnitude.

### Cross-cutting slices — do not add these to the table

Each is a slice through several categories and is already inside them.

| slice | Ir | share of own run | /cl |
|---|---:|---:|---:|
| ours: bounds checking (inherited, `2026-09-21-bounds-check-share.md`) | ~200,239,889 | 0.9868% | 10.01 |
| ours: `roots.rs`, the register file | 578,359,223 | 2.850% | 28.92 |
| ours: surviving trace ops, 38,460,944 x 8 (inherited, re-profile) | 307,687,552 | 1.516% | 15.38 |
| oracle: `ExpressionStack.hpp` + `.cpp` | 402,749,563 | 3.768% | 20.14 |
| oracle: `ProtectedObject.hpp` + `.cpp` | 543,595,568 | 5.086% | 27.18 |
| oracle: `ProtectedBase` ctor+dtor, by function self | 312,508,220 | 2.924% | 15.63 |

`roots.rs` measured with `fileslice.py`, which reproduces callgrind's own file
view; it comes to 578,359,223 against the brief's 578,375,148, a difference of
15,925 (0.003%), so it reproduces.

### "Everything else"

Under the brief's 5% threshold on both sides, so no naming is owed, but: ours
(0.30%) is startup — `rexx_parse::*` compiling the program,
`rexx_classes::method_dict`/`class_graph` building the class tree. The oracle's
(0.15%) is the dynamic loader (`_dl_lookup_symbol_x`, `do_lookup_x`,
`_dl_relocate_object_no_relro`), `RexxDateTime`, and 3,924,230 Ir at addresses
with no symbol. Neither is interpretation.

## The three findings the brief asked me to check

### 1. The C allocator — half confirmed, half corrected, and the conclusion flips

**The oracle's C-allocator figure is 71,570 Ir in the whole run, not 286.**
Enumerated, every function whose source file is `malloc.c` plus the global
`operator new`/`delete`: `free` 29,592, `malloc` 25,020, `_int_malloc` 11,135,
`__libc_malloc2` 1,318 and eighteen more under 700 each. It is still
**0.00067% of the run and 0.0036 Ir per clause** — the qualitative claim is
right, the number is 250x off. The oracle calls `malloc` **890 times** and
`free` **1,658 times** in the whole run.

**Ours is 900,376,297 (4.437%, 45.02/cl), not 868,687,634.** Same enumeration
on the same dump; the difference of 31,688,663 is the smaller malloc.c entries
the earlier count left out (`realloc` 27,127,521, `calloc` 14,078,008,
`_int_realloc` 12,768,122, `alloc_perturb` 7,250,295 and the rest). We call
`malloc` **5,232,180** times and `free` **5,772,216** times.

**"Allocation volume looks like a wash" is wrong, and it is wrong in our
favour.**

| | ours | oracle |
|---|---:|---:|
| objects allocated | **6,140,867** | **15,682,531** |
| Ir per allocation, inclusive | 184.5 | 113.4 |
| allocator inclusive total | 1,133,293,243 (56.7/cl) | 1,777,963,643 (88.9/cl) |

The oracle allocates **2.55x as many objects as we do**. `newObject`'s self
cost is 1,424,894,453 = **71.24 Ir/clause**, which confirms the brief's "about
71 per clause" exactly and shows the seven-file spread was one function. Its
own `__memset` arc is a further 203,184,750 over 15,682,531 calls.

So: we win allocation all-in by 18.17/cl on the category rows, or ~28/cl
counting the oracle's zeroing where the work is. The brief's *shape* claim —
"the difference is where it is paid" — holds: they pay in a bump allocator and
a sweep, we pay 45.02/cl to glibc that they pay nothing for.

### 2. Bounds checking — confirmed as a slice, not double counted

0.9868% of our run, ~200,239,889 Ir, 10.01/cl. It is inside machinery,
compound access, PARSE, arithmetic and the allocator rows and is **not** a row
of its own. Inherited from `2026-09-21-bounds-check-share.md`; I did not
re-derive it.

### 3. `roots.rs` — the figure confirms, the comparison beside it does not

`roots.rs` re-measures at 578,359,223 (2.850%, 28.92/cl), reproducing the
brief's 578,375,148.

**"The oracle's `ExpressionStack` push and pop are raw pointer moves that
inline to nearly nothing" is not what the dump says.** They inline, but the
inlined code is priced: `ExpressionStack.hpp` carries 278,416,082 and
`ExpressionStack.cpp` 124,333,481, **402,749,563 together = 3.768% of the
oracle's own run, 20.14/cl** — a *larger* share of its run than our register
file is of ours (2.850%). And that is only its operand stack. Its second
rooting mechanism, `ProtectedObject`, costs a further 543,595,568 by file slice
(5.086%, 27.18/cl), or 312,508,220 (15.63/cl) counting only the `ProtectedBase`
constructor and destructor as functions. `ProtectedBase::ProtectedBase` alone
runs from PARSE (5,600,015 calls), function evaluation (2,520,217), compound
reads (2,500,000), arithmetic (1,400,199) and compound writes (1,380,000).

**On rooting we are ahead, not behind.** Ours 2.850% of our run against
theirs 3.768% + 5.086% = 8.854% of theirs. Do not spend here on the strength
of finding 3.

## Method, and the changes I made to the category list

**Functions, not files** — the brief's reason holds and I hit it directly:
`write_small_int` is declared to callgrind under
`library/core/src/num/int_macros.rs` and `small_int_operand` under
`rexx-core/src/handle.rs`. Both are ours; a file rollup puts them in the
standard library.

**The rule.** A function is categorised by the work it performs. A function
that is a dispatcher — it decides which of several operations to run and
delegates — is machinery. Where a function is used by exactly **one** category
on this program, checked against the call arcs, it is charged to that category,
because the other side may inline the same work and the rows must stay
comparable. Four placements were decided that way rather than by name:

* `assign_expr_target` → compound. Called 1,660,000 times, **all** of them the
  Compound (1,380,000) or Stem (280,000) arm; no simple store reaches it.
* `read_by_name_at` → compound. Sole caller `append_tails`, 5,040,000 calls.
* `RexxString::upper` + `checkLower` → PARSE. Sole hot caller
  `RexxTarget::next`, 560,006 calls, 205,852,794 Ir; ours is inlined in
  `exec_parse`.
* `hash_one::<&[u8]>` + `sip::Hasher::write` (111,852,756, 5.59/cl) → call and
  argument handling. Sole hot caller `resolve_fixed_call`, 560,013 calls. This
  is *not* the stem tail map, which uses `FxBuildHasher`; it is SipHash on a
  routine name, about 196 Ir per call resolution.

**Two categories widened, both symmetrically:**

* "arithmetic and numeric comparison" → **"arithmetic and comparison"**. String
  comparison is decided in the same operator path as numeric on both sides
  (`rexx_num::compare::compare_strings` sits beside `numeric_order`;
  `RexxString::stringComp` beside `NumberString::comp`). Splitting them would
  have put the two halves of one decision in different rows.
* "string and number conversion" → **"text/number conversion, string
  building"**, to take concatenation (`concat_values`; `RexxString::concatRexx`
  and `concatBlank`). Concatenation is not conversion, but it is the same
  render-and-copy work, and it had no other home.

## What I did not do

* **I did not re-take either dump.** Both are the brief's, verified by their
  `summary:` lines and hashed above.
* **I did not run anything against the oracle**, so `rust/corpus/oracle-crashes.txt`
  never came up.
* **I did not split our driver's self cost by op arm.** That is the one
  measurement that would make the machinery row a like-for-like comparison, and
  it needs per-address work against the two jump tables, not a function rollup.
* **This is one program.** `rexxcps` spends 9.26% of our run in PARSE and 8.51%
  in compound variables; a program that does neither would rank these
  categories differently. Every figure here is `rexxcps` only.
* **The 20,000,000 clauses is checked, not inherited**: `rexxcps.rex` has
  `count=200`, `averaging=100` and a 1000-clause timed body
  (`do i=1 to averaging` / `do count` / 1000 clauses), last modified at
  `d90de68e3`, well before both dumps, so both runs ran the same program.
  `c418d4555` is documentation only, so the binary measured at `f3974036a` is
  still this tree's.
