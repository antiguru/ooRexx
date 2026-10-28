# Task 8.7a review: crash, builtins, collections

Reviewed `d6a9ad129..a82f917f5` (code BASE `81f4bf250`, code HEAD `030af45a1`).

**Spec compliance: COMPLIANT.** **Quality: CHANGES REQUESTED** (Important 1: the closed rooting
item's own backtrace is still reachable under TRACE I through a method that returns a reference).

## What I ran

* Builds: `git archive` of `81f4bf250` and `030af45a1` (`rust/` and `interpreter/`, which
  `rexx-classes/build.rs` and `rexx-lib/build.rs` read) into
  `t87a-review-scratch/{base,head}-src`, each built with `CARGO_INCREMENTAL=0 memcap 8G cargo build
  --release -j 4 --bin rexx` into `/tmp/claude-1000/t87a-review-target-base` and `-head`, exit 0.
  The `.text` section of each (`objcopy -O binary --only-section=.text`) is byte-identical to
  `ooRexx-p9-pin/cg-9d/t8.7a/rexx-base-81f4bf250` and `rexx-head-030af45a1`, so the pinned binaries
  carry the named commits.
* A collect-on-every-allocation runner: an untracked `examples/stress.rs` in each archived copy
  calling `rexx_exec::run_program_collect_every_alloc` (the collect_stress harness's entry point)
  on one file, printing stdout, stderr and the collection count.
* Probes: every one in its own directory under `t87a-review-scratch/`, run by `bin/run3` (oracle
  under the standard wrapper, base and head under `memcap 2G timeout -k 5 20`), stdout, stderr and
  rc compared separately.
* Tests at head, from the archived copy: `cargo test --release -p rexx-exec --test collect_stress`:
  `test result: ok. 74 passed; 0 failed`. `--lib`: `1133 passed; 67 failed`, and every failure
  panics with `the worktree's build/lib is three directories above this crate` (the archive has no
  `build/lib`); none is about this task.
* Perf: `bench-programs/callgrind.sh -r 1 -j 4 -B 0.5 -p "intcall heapshape define alloc startup
  churn sayloop interpret strings dispatch" base=<my base> head=<my head>`, exit 0.
* Memory: one loop over the changed paths at two counts, and the item's `tr4.rex`.

## Each item: witness red at BASE, matching at HEAD

Every witness, run from a fresh directory (`wit/`):

| witness | oracle rc | base vs oracle | head vs oracle |
|---|---|---|---|
| trace_reference_value_rooted | 0 | same (normal binary); stress rc 101 `dispatch.rs:1831:9: a live value` | same; stress rc 0, stdout equals the oracle's |
| native_routine_argument_marshalling | 168 | rc 0, stdout and stderr differ | identical |
| class_side_defaultname | 0 | stdout differs | identical |
| string_translate_empty_input_table | 0 | stdout differs | identical |
| array_sort_leading_items | 0 | rc 158, stdout and stderr differ | identical |
| method_source_rexx_package | 0 | stdout differs | identical |
| index_argument_string_value | 0 | stdout differs | identical |
| string_equals_string_value | 0 | stdout differs | identical |

This agrees with the report's `witnesses/summary.tsv`. The `red/collect-stress-trace-reference.txt`
record is the harness's panic `dispatch.rs:1831:9: a live value`; its attribution to the witness is
by elimination, as the report says, and my direct stress run of the witness at base gives the same
panic, so the red holds.

The item's own probes: `tr2.rex` under stress panics at base (rc 101, `a live value`) and runs at
head (rc 0). `tr4.rex` in the normal binary: base rc 101 after 0.11 s; head rc 0, `end` on stdout,
70.33 s, 42044 KB maximum RSS.

Probe-agrees closings: `queued-probes/imeth/imeth.rex` (`0 Method` / `Method`) and
`queued-probes/dimeth/dimeth.rex` (`1`) are identical on all three descriptors on the oracle, base
and head. The report's ampersand probes `probes/amp/a1..a3.rex` are identical on all three at head
(`Error 35.1:  Incorrect expression detected at "~".`, `Error 16.1:  Label "S1" not found.`,
`Error 43.1:  Could not find routine "S1".`), and so is the 47.1 label insert
(`interpret 's1: say 1'`: `Error 47.1:  INTERPRET data must not contain labels; found "S1".`).

Records: every item file's first line is `**CLOSED 2026-10-28 by Phase 9d Task 8.7a (030af45a1).**`
(the ampersand item names its probe instead), and each inventory row has `closed:030af45a1` (or
`closed:probe agrees`) in columns 4 and 6. `t8.7a/closed.tsv` holds
`gate:phase-9a-gate.md:480	closed:probe agrees`, the gate line being
`.D~new~instanceMethod('M') == .C~method('M')`, which `dimeth.rex` tests.

Entry 62: the shape in `oracle-crashes.txt` run once on the oracle from a fresh directory:
`Segmentation fault`, rc 139. Base and head: rc 120, `rexx-exec: a message send to a value that is
not an array is not implemented`.

## Hunting results that held

* **TRACE siblings.** Under the stress runner, head runs rc 0 on every one of these, and base
  panics on most of them: `trace r` and `trace i` over `q = >y`; `a.1 = >y` and `b. = >y`; `parse var`
  of a reference; `call f >y` with `use arg r` and with `use arg >w`; `trace a` over a reference
  argument; a routine returning a fresh Array; `interpret "q = >y"`; `DO OVER` an array of
  references; `q~value = 3`; references in a directory entry; a `.traceOutput` destination written
  in Rexx that allocates on every line, looping over reference assignments, compound assignments,
  calls and `.array~of(>y)`; `x = .array~new~~append(1)~~append(2)`, stem `[]=`, `parse value`,
  `interpret`, `signal value`, `call value`. Head's stress stdout and stderr equal its normal run on
  each.
* **FILESPEC and DIRECTORY** (`arg/f1.rex`): identical to the oracle, including a String subclass
  whose `STRING` is overridden (oracle and head both read the string itself: `c.d`, `y.z`), an
  Array argument (`q.r`, `/p/`), objects with and without `STRING` (88.909 at the right position),
  `''` and a NUL-first option (`found ""`), NUL inside the name, and the arity errors.
* **BEEP** (`arg/b1.rex`): identical to the oracle on omitted, non-numeric, fractional, `1e30`,
  out-of-range-native, out-of-range-body (`L 25` versus `L The NIL object` as on the oracle), an
  object whose `STRING` answers `50`, an Array duration and `' 4E1 '`. Base differs on 13 lines.
* **Index and whole-number arguments** (`arg/i1.rex`): identical to the oracle on String,
  MutableBuffer, Array, List and Queue methods with an object whose `STRING` answers `2`, an Array
  of one line, an empty Array, an Array of two lines, a String subclass, a `MAKESTRING` object, an
  object whose `STRING` answers an Array (`found "a V"`), `.nil`, a Stem, a class, an object whose
  `STRING` raises (`93.900 boom.`), `section`, `insert` and `new`. NOSTRING is raised where the
  oracle raises it (`arg/e2.rex`, `e3.rex`: `NOSTRING an Object` on both; base rc 163).
* **equals** (`arg/e1.rex`, `e4.rex`): identical to the oracle, including `.nil`, an object's
  default name, a String subclass, an Array, a class with a class-side `DEFAULTNAME`, numbers,
  a MutableBuffer, a raising `STRING`, the arity errors and NOSTRING under `call on` and
  `signal on`.
* **DEFAULTNAME** (`dn/d1.rex`, `d3.rex`, `d5.rex`): identical to the oracle for a class method,
  one that uses `self~id`, an inheriting subclass, a user metaclass, a plain class, a class made by
  `subclass`, a set name, `defaultName` sent directly, an Array subclass with and without its own,
  a class-side `OBJECTNAME`, a numeric answer, an instance of the class, a class with both `STRING`
  and `DEFAULTNAME`, concatenation, and a metaclass made by `.class~subclass` with `define`.
* **Method~source** (`ms/m1.rex`): library methods answer 0 lines as on the oracle; a method and a
  routine from a `::requires`d file, the program's own methods and `.method~new`/`.routine~new`
  answer 1 on both. `library_programs` holds only the image's `.orx` programs (`rexx-lib/build.rs`
  `SOURCES`), so a required file is not caught by the new arm.
* **Perf.** My run reproduces the report's figures: `intcall 750028327 720780674 -3.8995 ...
  -3.1204`, `heapshape ... -0.7450 ... -0.7867`, `define ... +0.0265 ... -0.0040`, `alloc ...
  +0.1064 ... +0.0000`, `startup ... +0.0013 ... +0.0020`, `churn ... -0.0313 ... -0.0948`,
  `sayloop ... +0.0009`, `interpret ... +0.0001`, `strings ... +0.0000`, `dispatch ... +0.0000`, no
  OVER or SPREAD. Every program in `callgrind-final.txt` is within +0.05% libc subtracted (largest
  `startup` +0.0017); pingguard +0.0001 and pingsem +0.0002 with no SPREAD flag. The `cmd:` lines of
  the raw outputs in `cg-9d/t8.7a/final2` name only `rexx-base-81f4bf250` and
  `rexx-head-030af45a1`.
* **Memory** (`mem/m.rex`, one loop over `substr`, `[]`, `equals` with a `STRING` object,
  `filespec`, a class-side `DEFAULTNAME` render, `translate('', '')` and `sort` after `remove`):
  `n=50000 0.22 s 26844 KB`, `n=200000 0.82 s 26756 KB`. A traced reference loop
  (`st2/c4loop2.rex`, 30000 iterations): 41872 KB.

## Issues

### Critical

None.

### Important

1. **The closed item's backtrace is still reachable: under TRACE I, a method result that is a
   reference is dead before its `>M>` line.** `st2/c4o.rex`:

       trace i
       v = .array~new
       o = .k~new
       w = o~r(>v)
       say w~class
       exit
       ::class k
       ::method r
         use strict arg >x
         return >x

   Oracle rc 0, `The VariableReference class`. Head under the stress runner: rc 101,
   `panicked at crates/rexx-exec/src/dispatch.rs:1831:9: a live value`, backtrace
   `not_in_arena <- to_text <- string_value_text <- result_text <- assign_evaluated`, the item's own.
   `say o~r(>v)~class` (`c4f.rex`) and the same with `v = 1` (`c4n2.rex`) end rc 120,
   `rexx-exec: a message send to a value whose object is no longer live is not implemented`. Base
   panics on all three. The same shape passes under `trace r` (`c4o_r.rex`), with no trace
   (`c4o_n.rex`, `c4k.rex`), as a routine (`c4m.rex`), and with `use strict arg x; return x`
   (`c4g.rex`). With `eprintln!` checks added to a scratch copy of head (not the tree), the
   assigned handle is already absent from the heap when `trace_intermediate_event` is entered for
   the `>M>` line, and again at `assign_evaluated`'s entry, so the rooting the fix adds inside the
   event begins after the value is gone: the collection falls between the method's return and the
   `>M>` event. Not reproduced in the normal binary (`c4loop.rex` with a Rexx trace destination
   and `c4loop2.rex` without one, rc 0 on both binaries), so this is a stress-mode witness, as
   `tr2.rex` is for the item. The fix belongs where a traced message result is handed back to the
   caller, before anything on the TRACE I path allocates; the witness is `c4o.rex` in
   `trace_reference_value_rooted.rex`.

### Minor

1. **A doc block now describes the wrong function.** `value.rs` inserts `requested_number`
   between `operand_number`'s doc comment and its `fn`: at `030af45a1`, lines 777-779 ("[`Interp::to_number`]
   for the receiver of an operator: a `String` subclass instance is `NotNumeric` here ...") sit on
   top of `requested_number`'s own doc, and `operand_number` has none
   (`git show 030af45a1:rust/crates/rexx-exec/src/value.rs | sed -n 777,808p`).
2. **`equals` with a reference argument is neither witnessed nor stated in Deviation 72.**
   `eqr/r1.rex`, oracle versus head: `'kmake'~equals(>k) 'a K'~equals(>k)
   'KMAKE'~caselessEquals(>k)` is `0 1 0` versus `1 0 1`; `'abc'~equals(>y) 'Y'~equals(>y)` is
   `1 0` on both; with `signal on nostring` the oracle raises `NOSTRING a K` and head answers `1`.
   The behaviour follows the "Rule everywhere" ruling, but Deviation 72's text says the oracle
   "raises 88.909 for a reference" in a String method's argument (true for `pos` and `compareTo`
   in the same probe), while for `equals` the oracle compares the reference's default rendering and
   raises NOSTRING when trapped. The 8.5 ruling ("witness and state them") asks for a sentence in
   the entry and a crate-only test; `grep -rn "equals(>" crates/ corpus/` finds none.
3. **The TRANSLATE KNOWN GAP now reaches the String method, and its text names the builtin only**
   (concern 3). `arg/x2.rex` line 10, `'12'~translate(left('',0), left('',0))`: oracle `[  ]`,
   base `[  ]`, head `[12]`. Base agreed on that shape by reading every empty table as omitted;
   head follows the rule for every supplied table, which is right, but the departure is recorded
   only in the report until `phase-4-exclusions.txt`'s entry is extended.

## The report's concerns

1. **Confirmed, and a fair queue item.** `probes/dn/dn4.rex` and `dn5.rex`: the trace lines, the
   97.1 insert and the Array `makeString`/`toString` join print `The WITHNAME class` and
   `a WITHNAME` where the oracle prints `DN`; base is byte-identical to head on both. The item
   named `native_string` and a class's rendering; the remaining renderings go through
   `string_value_text` for instances and classes alike, which predates the task.
2. **Confirmed.** Oracle rc 139 on entry 62's shape; the crate's loud refusal is unchanged.
3. **Confirmed**, see Minor 3.
4. **Confirmed** for `Method~source`. Queue candidate below for `Package~source`.
5. **Confirmed** as within Deviation 72's ruling, see Minor 2.

The report's new oracle defect (entry 62) is confirmed by one run.

## Queue candidates (predate the task)

* **`Package~source` of the REXX package.** `ms/m1.rex`:
  `.stream~method('INIT')~package~source~items` and `~sourcesize` answer 0 on the oracle and 1010
  here, and `sourceline(1)` is empty there and the `.orx` banner here (base the same). The task
  made `Method~source` empty for these packages; the package still exposes the source.
* **TRACE I's `>K>` line for `PARSE VALUE` of an object.** `tr/a14.rex`,
  `parse value .array~new(2) with a`: oracle `>K>   "VALUE" => "an Array"`, crate
  `>K>   "VALUE" => ""` (base the same). The same value's `>M>` line prints `"an Array"` on both.
* The internal renderings of concern 1.

## New oracle defects

* **TRACE I over `~value=` on a variable reference: SIGSEGV.** `tr2/n5.rex`:

      q = >y
      trace i
      q~value = 3
      say y

  rc 139 after `>A>   "3"`, 4 of 4 runs of the variant `tr/a17.rex` under the standard wrapper
  from fresh directories, and once each for `n4.rex` (`y = 1` first) and `n5.rex`. Without TRACE
  (`n1.rex`) and under `trace r` (`n2.rex`) the oracle answers `3`, rc 0. A `.directory` receiver
  (`n6.rex`) does not crash. The crate answers `3`, rc 0, with the `>V>`/`>>>` lines for `say y`.
  For `oracle-crashes.txt` and `defects.json`.
