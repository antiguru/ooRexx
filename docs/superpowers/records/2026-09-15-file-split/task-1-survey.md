# Task 1 survey: which files hold more than one responsibility

BASE: `ec7824e4a`, working tree clean at the time of measurement. Nothing in
`rust/` was edited to produce this document.

## How the candidates were enumerated

Two enumerations, because the trigger and the question are not the same thing.

**By length**, the plan's own command, re-run at BASE:

```
cd rust && find crates -name '*.rs' | xargs wc -l | sort -rn | awk '$1 > 1000 && $2 != "total"'
```

and its decomposition, which reproduces the plan's own figures at `5d84dd8cb`:

```
cd rust && find crates -name '*.rs' | xargs wc -l \
  | awk '$1>1000 && $2!="total"{print $2}' \
  | awk -F/ '{for(i=1;i<=NF;i++) if($i=="src"){print "src"; next}} {print "tests"}' \
  | sort | uniq -c
       40 src
        8 tests
```

so no file crossed the trigger between `5d84dd8cb` and BASE. The largest are
`rexx-exec/src/dispatch.rs`, `rexx-exec/src/run.rs`,
`rexx-exec/src/run/tests.rs` and `rexx-exec/src/lib.rs`, in that order, the same
ones the plan names.

**By responsibility**, over every `.rs` from 400 lines up, reading each module
doc header and a column-0 item outline:

```
cd rust && for f in $(find crates -name '*.rs' | xargs wc -l \
    | awk '$1>=400 && $1<=1000 && $2!="total"{print $2}'); do
  echo "$(wc -l < $f)  $f :: $(sed -n '11,20p' $f | grep '^//!')"
done
```

The outline tool is a grep for column-0 item keywords plus four-space `fn`,
which is exact for this tree because rustfmt puts every top-level item at
column 0. It is at `outline.sh` in this session's scratchpad, beside the two
measurement scripts named below.

## The measurement that reorders the list

A physical line count does not say whether a file holds one responsibility,
because an inline `#[cfg(test)] mod tests` is a second thing in every file that
has one. Splitting total into inline-test lines and the rest changes which files
are worth looking at, and for several it changes the answer entirely.

A top-level `#[cfg(test)] mod X {` starts at column 0 and ends at the first line
that is exactly `}` at column 0. Only attribute and comment lines may sit
between the gate and the `mod`, which is what keeps a `#[cfg(test)]` on a
*function* from being read as a test module. The script is `testsize3.py` in the
same scratchpad directory. An earlier version without that restriction reported
`lib.rs` as 5713 test lines because it scanned forward from a `#[cfg(test)] fn`
at 1411 to the `mod tests` at 6149; every figure it disagreed with was checked
back against `grep -n '^#\[cfg(test)\]'` on the file itself before this table
was written.

```
     total  tests   code  path
     10710   1792   8918  crates/rexx-exec/src/dispatch.rs
      9229      0   9229  crates/rexx-exec/src/run.rs
      8328      0   8328  crates/rexx-exec/src/run/tests.rs
      7123    976   6147  crates/rexx-exec/src/lib.rs
      3338   1750   1588  crates/rexx-exec/src/eval.rs
      2992      0   2992  crates/rexx-exec/src/dispatch/hash.rs
      2571    259   2312  crates/rexx-exec/src/environment.rs
      2549      0   2549  crates/rexx-exec/src/ir/drive.rs
      2490      0   2490  crates/rexx-parse/src/instruction.rs
      2452    265   2187  crates/rexx-exec/src/error.rs
      2352    418   1934  crates/rexx-exec/src/ir/compile.rs
      2279      0   2279  crates/rexx-exec/src/dispatch/collection.rs
      2222     31   2191  crates/rexx-exec/src/dispatch/string.rs
      2114      0   2114  crates/rexx-api/tests/values.rs
      2092   1083   1009  crates/rexx-exec/src/builtin/convert.rs
      2084    914   1170  crates/rexx-exec/src/builtin/string.rs
      2084      0   2084  crates/rexx-parse/src/instruction/tests.rs
      2046    959   1087  crates/rexx-exec/src/value.rs
      2001    838   1163  crates/rexx-exec/src/builtin/datetime.rs
      1988      0   1988  crates/rexx-api/src/values.rs
      1925      0   1925  crates/rexx-exec/src/dispatch/stream.rs
      1869      0   1869  crates/rexx-exec/tests/gate_table_c.rs
      1865      0   1865  crates/rexx-classes/tests/native_classes_wiring.rs
      1798   1001    797  crates/rexx-exec/src/plan.rs
      1647      0   1647  crates/rexx-exec/src/ir/golden_tests.rs
      1514    230   1284  crates/rexx-bench/src/bin/rexx-bench-suite.rs
      1509      0   1509  crates/rexx-exec/tests/coverage.rs
      1505      0   1505  crates/rexx-parse/src/directive/tests.rs
      1391    617    774  crates/rexx-exec/src/dispatch/library.rs
      1362    310   1052  crates/rexx-exec/src/trace.rs
      1352      0   1352  crates/rexx-parse/src/ast.rs
      1337    190   1147  crates/rexx-extract/src/docs/classes.rs
      1330    729    601  crates/rexx-exec/src/builtin/numeric.rs
      1319    356    963  crates/rexx-exec/src/builtin.rs
      1288    492    796  crates/rexx-exec/src/parse_template.rs
      1272     56   1216  crates/rexx-exec/src/activation.rs
      1212      0   1212  crates/rexx-exec/tests/method_bodies.rs
      1211      0   1211  crates/rexx-exec/tests/ir_recorded.rs
      1144    945    199  crates/rexx-api/src/invoke.rs
      1117      0   1117  crates/rexx-parse/src/directive.rs
      1073      0   1073  crates/rexx-parse/src/scanner.rs
      1051      0   1051  crates/rexx-num/tests/format.rs
      1046    344    702  crates/rexx-exec/src/dispatch/native.rs
      1043      0   1043  crates/rexx-parse/src/expr.rs
      1041    193    848  crates/rexx-api/src/ffi.rs
      1032      0   1032  crates/rexx-exec/src/dispatch/package.rs
      1023      0   1023  crates/rexx-parse/src/block.rs
      1007      0   1007  crates/rexx-parse/tests/scanner.rs
```

The files whose non-test code is already under the trigger, so that moving the
test module out is the whole of what the trigger was pointing at:
`rexx-api/src/invoke.rs`, `builtin/numeric.rs`, `dispatch/native.rs`,
`dispatch/library.rs`, `parse_template.rs`, `plan.rs`, `builtin.rs`.

## What the file-granular records actually pin

Read at BASE rather than taken from the plan. The plan's Global Constraints need
correcting in two places, and there are records it does not name.

### `rexx-core/tests/unsafe_sites.rs`

`only_the_granted_module_may_say_unsafe` asserts two separate lists against
literal path vectors:

* the files carrying `allow(unsafe_code)` or `expect(unsafe_code)`:
  `crates/rexx-api/src/ffi.rs`, `crates/rexx-api/src/load.rs`,
  `crates/rexx-core/src/lib.rs`;
* the files containing `unsafe {`, `unsafe fn`, `unsafe impl` or
  `unsafe trait`: `crates/rexx-api/src/ffi.rs`, `crates/rexx-api/src/load.rs`,
  `crates/rexx-core/src/bytes.rs`.

The plan says "D-U1 grants `unsafe` to exactly `rexx-api/src/ffi.rs` and
`src/load.rs`". `rexx-core/src/bytes.rs` is missing from that, and
`rexx-core/src/lib.rs` is missing from the opt-in half. No candidate of this plan
is in `rexx-core`, so nothing is blocked by it, but as written the constraint
would let a task treat an `unsafe` block in `bytes.rs` as a violation.

The scan walks every `.rs` under `crates/`, so a **new file** holding an
`unsafe` block turns this red, whatever module it is declared from.
`rust/CLAUDE.md` says so directly: an opt-in on a module reaches every submodule
of it, and the two assertions are separate for that reason.

`the_scan_reaches_the_whole_workspace` asserts the scan finds
`crates/rexx-core/src/bytes.rs`, `crates/rexx-exec/src/run.rs`,
`crates/rexx-num/src/lib.rs` and `crates/rexx-parse/src/lib.rs` by exact
relative path. Task 3 must leave `run.rs` in place as a file, which the plan's
architecture already does.

### `rexx-exec/tests/dispatch_seam.rs`

Pins, only one of which the plan describes.

1. `the_protected_question_is_asked_only_inside_the_seam` requires
   `method_is_protected(` to occur exactly twice and every occurrence's path to
   contain the literal `dispatch.rs`. At BASE
   (`grep -rn 'method_is_protected(' crates/rexx-exec/src/`):

   ```
   crates/rexx-exec/src/dispatch.rs:60      the call, inside `mod seam`
   crates/rexx-exec/src/dispatch.rs:1975    the definition
   ```

   A child file at `src/dispatch/<name>.rs` does **not** contain the substring
   `dispatch.rs`. So `mod seam`, `Interp::method_is_protected` and
   `Interp::check_protected_method` stay in `src/dispatch.rs`.
2. `the_seam_module_holds_one_struct_one_enum_and_one_function` finds
   `mod seam {` by text search in `src/dispatch.rs` and brace-matches its body.
   The seam module stays in that file, spelled that way.
3. `seam::clear(` must occur exactly once across the crate. At BASE
   (`grep -rn 'seam::clear(' crates/rexx-exec/src/`) it is at
   `dispatch.rs:2134`, inside `Interp::invoke`. `Interp::invoke` stays in
   `src/dispatch.rs`.

`CLEARANCE_CONSUMERS` is the file allowlist the plan does describe. Every native
method body names `Cleared` in its signature, so any new child file holding a
native body has to be added to it. Code lines naming the token, per file at BASE
(`grep -rn 'Cleared' crates/rexx-exec/src/ | grep -v '^\S*: *//' | cut -d: -f1 | sort | uniq -c | sort -rn`):

```
    161 crates/rexx-exec/src/dispatch.rs
    111 crates/rexx-exec/src/dispatch/string.rs
     66 crates/rexx-exec/src/dispatch/collection.rs
     47 crates/rexx-exec/src/dispatch/hash.rs
     35 crates/rexx-exec/src/dispatch/package.rs
     27 crates/rexx-exec/src/dispatch/rexx_info.rs
     25 crates/rexx-exec/src/dispatch/files.rs
     24 crates/rexx-exec/src/dispatch/stream.rs
     24 crates/rexx-exec/src/dispatch/context.rs
     17 crates/rexx-exec/src/dispatch/executable.rs
      9 crates/rexx-exec/src/dispatch/introspection.rs
```

### `rexx-exec/tests/environment_seam.rs`

`env_seam::admit(` and `env_seam::directory(` must each occur exactly twice and
every occurrence's path must contain `src/environment.rs`. The call sites are
`Interp::directory_lookup` (the read path) and `Interp::set_directory_entry`
(the write path). The seam module is found by text search in
`src/environment.rs` and brace-matched. `mod env_seam`, `directory_lookup` and
`set_directory_entry` stay in that file.

### Records the plan does not name

* **`corpus/refusal-sites.tsv` column 3 is derived from file paths**, not only
  column 4. `tests/refusal_sites.rs:276` tags a construction site `send` when
  its path ends with `dispatch.rs` **or contains `/dispatch/`**, and `ir` when it
  ends with `ir.rs` or contains `/ir/`. Moving a construction site from
  `dispatch.rs` into `dispatch/<child>.rs` therefore leaves column 3 unchanged,
  which is what makes a dispatch split safe for this table. Moving one out of the
  `dispatch/` subtree changes column 3, and the plan calls that a finding rather
  than a refresh.

  Column 4's distribution over files at BASE
  (`awk -F'\t' '!/^#/{print $4}' corpus/refusal-sites.tsv | sed 's/:[0-9]*$//' | sort | uniq -c | sort -rn`):

  ```
      180 crates/rexx-exec/src/error.rs
       45 crates/rexx-exec/src/lib.rs
       19 crates/rexx-exec/src/run.rs
        2 crates/rexx-exec/src/trace.rs
        1 crates/rexx-exec/src/dispatch/native.rs
  ```

  A `lib.rs` split that moves `impl Loud`, and a `run.rs` split that moves the
  `raised_*` constructors, each rewrite a large block of column 4. That is a
  refresh, but the commit's table diff will be large and a reviewer should be
  told to expect it rather than read it as a finding.

  The table's own header prose says "`send` is dispatch.rs ... and
  dispatch/native.rs", which is narrower than the rule the test applies. A
  dispatch split that puts a refusal construction site in a new child makes that
  sentence incomplete. It is prose in a derived file, so `REFRESH=1` does not fix
  it; the task that creates such a site corrects the sentence by hand.

* **`corpus/docs/class-set.txt` and `corpus/docs/class-methods.txt` each carry a
  header line naming their deriving module**:

  ```
  # module is `src/docs/classes.rs`. Re-derived and compared in both
  ```

  built at `rexx-extract/src/docs/classes.rs:1079` and `:1141`, and compared in
  both directions by `tests/extract_docs.rs`. A split of `docs/classes.rs` must
  either keep that sentence true, by leaving the entry point in `classes.rs`, or
  change the string and re-derive both committed files.
  `corpus/docs/provide-sections.txt`, `corpus/docs/hierarchy-edges.txt` and
  `corpus/docs/directive-options.txt` have the same shape for `docs/provide.rs`,
  `docs/hierarchy.rs` and `docs/directives.rs`, none of which is a candidate.

* **Prose that names a file by path and would go stale**, found with
  `grep -rn -E 'src/[a-z_]+(/[a-z_]+)*\.rs'` over `crates/*/tests`,
  `crates/*/src`, `crates/*/benches` and `corpus/`:
  * `tests/builtin_status.rs:494` and `:546` document `STRING_FAMILY` and
    `STATE_FAMILY` as "the names `src/builtin/string.rs` runs" and "the names
    `src/builtin/state.rs` runs". A split of `builtin/string.rs` (Task 7) makes
    the first sentence incomplete. Neither is an assertion, so nothing goes red.
  * `corpus/errors/parse-errors.tsv` lines 53 to 76 record the table's
    provenance in `src/instruction/tests.rs`, `src/directive/tests.rs`,
    `src/block/tests.rs`, `src/expr/tests.rs`, `tests/scanner.rs` and
    `tests/program.rs`. Provenance is historical, so a later split leaves it
    true and a task should not "correct" it.
  * `corpus/gate-tables/README.md` names `rexx-parse/src/instruction/tests.rs`,
    which Task 9 targets, and `corpus/oracle-crashes.txt` names
    `rexx-exec/src/builtin/string.rs`, which Task 7 targets.
  * `corpus/lang/push_queue.rex` names `rexx-exec/src/queue.rs`. Per
    `rust/CLAUDE.md:15`, editing any byte of a `corpus/lang/*.rex` reddens
    `sourceline_matches_the_interpreter_for_every_corpus_program`, whose
    recorded file holds that program's line count and every line verbatim.
    `queue.rs` is not a candidate, so there is no conflict here; a task that
    ever wants to split a file one of those programs names has to stop.
  * `tests/support/mod.rs:73` names `rexx-exec/src/trace.rs`'s `push_clause`.
    Under the proposal below `push_clause` stays in `trace.rs`.

## Recommendations

Ranked by what the split buys, with risk noted separately. Line ranges are at
BASE and say where the items are, not what order they should land in.

### Rank 1: `rexx-exec/src/dispatch.rs`, 10710 lines. SPLIT.

Responsibilities found, in file order:

1. the native method tables `NATIVE_METHODS`, `NATIVE_CLASS_METHODS` and
   `SETUP_METHODS`, 169 to 1020;
2. the security seam and the object model: `mod seam`, `ObjectModel`, `build`,
   `resolve`, `lookup`, `invoke`, the private, protected and package checks,
   `send_message`, `message_term`, `enter_method_body`, the `UNINIT` sweeps and
   the `blame_*` reporters, 33 to 2996;
3. the required-string latch and the string-conversion protocol, 2996 to 3392;
4. the `Object` protocol natives: `hasMethod`, `id`, `defaultName`, `metaclass`,
   `superclass`, `isA`, `hashCode`, `annotation` and the object operators, 3392
   to 3833, and `objectName`, `setMethod`, `copy`, `run`, `send`, `start`, the
   `Message` readers and `enhanced`, 5538 to 6350;
5. the `Class` protocol natives: `method`, `scope`, `define`, `subclass`, `new`,
   the metaclass factory, `inherit` and `uninherit`, 3833 to 4580;
6. array slot and subscript arithmetic and the `Array` natives, 4584 to 5500;
7. `MutableBuffer`: the whole class, plus the `pub(super)` argument parsers and
   search helpers that `dispatch/string.rs` shares, 6353 to 8230;
8. the constructors: `native_*_new` for each primitive class, `load_external`,
   `Pointer` and `WeakReference`, 8230 to 8720;
9. the test module, 8919 to 10710.

Item 2 cannot move: `mod seam`, `method_is_protected`,
`check_protected_method` and the one `seam::clear(` call are pinned to this file
by `dispatch_seam.rs`, and item 2 is also what the file's own doc header says
the file is. Everything else can.

Proposed children, each following the pattern this file already uses for
`string`, `collection`, `hash` and `package`, where a child owns both its
functions and the `NATIVE_METHODS` slice that points at them, chained into
`ObjectModel::build`:

| module | purpose | rough length |
| --- | --- | --- |
| `dispatch.rs` | the seam, the object model, resolve/lookup/invoke, the access checks, the send and uninit machinery | 2000 |
| `dispatch/buffer.rs` | `MutableBuffer`, and the byte-level argument and search helpers `string.rs` shares | 1880 |
| `dispatch/class_protocol.rs` | `Class`'s own methods: define, subclass, new, inherit, the metaclass factory | 750 |
| `dispatch/object_protocol.rs` | `Object`'s own methods and operators, `setMethod`, `copy`, `run`/`send`/`start` | 1250 |
| `dispatch/array.rs` | array slot and subscript arithmetic, and `Array`'s primitive methods | 920 |
| `dispatch/construct.rs` | `~new` for each primitive class, plus `Pointer` and `WeakReference` | 500 |
| `dispatch/reqstr.rs` | the required-string latch and the string-conversion classification | 400 |
| `dispatch/tests.rs` | this module's own tests | 1790 |

Each of the code children above must be added to `CLEARANCE_CONSUMERS` in
`dispatch_seam.rs` in the same commit. Each is dispatch code the list already
covered: each holds native method bodies whose signature names `Cleared`, and
each is being moved out of `src/dispatch.rs`, the list's first entry. None
introduces a new producer of the token, which stays in `mod seam`.

`refusal-sites.tsv` column 3 is unaffected, because `/dispatch/` carries the same
`send` tag as `dispatch.rs`.

Risk: the highest here, because this is the interpreter's hot dispatch path and
the plan's performance constraint bites hardest on it. Recommend several commits,
with `dispatch/tests.rs` first as a warm-up that moves no production code.

### Rank 2: `rexx-exec/src/run/tests.rs`, 8328 lines. SPLIT.

`grep -c '^#\[test\]' crates/rexx-exec/src/run/tests.rs` is 285. The file is
flat: one test at line 20, a helper block from 69 to 168, then tests already
grouped by the construct they exercise.

This buys nearly as much as Rank 1 and carries the least risk in the plan,
because no production item moves. Instruments 1 to 3 have nothing to compare
beyond the moved test bodies, and instrument 4 reduces to the same test names at
the same count in the same binary, since `run.rs`'s `#[cfg(test)] mod tests`
becomes `mod tests;` with children rather than a new binary.

Proposed children under `run/tests/`, with the helper block staying in
`run/tests.rs` as `pub(super)`: `assignment.rs` (168 to 435), `numeric.rs` (435
to 690), `branch.rs` (687 to 1430, IF and the SELECT/WHEN/WHENCASE family),
`loops.rs` (1434 to 2200), `iterate_leave.rs` (2202 to 2500), `interpret.rs`
(2500 to 3000), `result.rs` (3000 to 3460), `signal.rs` (3466 to 4100),
`scope.rs` (4100 to 5200, PROCEDURE, EXPOSE, USE ARG, USE LOCAL),
`conditions.rs` (5200 to 6200), `routines.rs` (6200 to 7100), `address.rs` (7100
to 8000), `directives.rs` (8000 to 8328).

Those boundaries come from sampling `grep -n '^fn '` every sixth name. They are
where the topics change, not exact cut points, and the task that moves should
re-derive them.

### Rank 3: `rexx-exec/src/run.rs`, 9229 lines. SPLIT.

One `impl Interp` runs from 764 to 8300. Its clusters:

* the loop itself: `run_activation`, `apply_flow`, `exec_instruction`, 821 to
  1590;
* message, procedure, expose, use, say, 1590 to 2232;
* guard, reply, forward, queue, return and assignment, 2233 to 2800;
* condition traps: novalue, lostdigits, `offer_to_trap`,
  `deliver_pending_traps`, 2822 to 3366;
* RAISE and SIGNAL, 3366 to 3760;
* CALL: resolution, argument settling, the invoke paths, 3760 to 4830;
* stepped-clause bookkeeping, trace echo and failure sites, 4831 to 5248;
* SELECT, 5248 to 5420;
* loops: header plan, DO/OVER, repeating, the flat-loop path, `loop_advance`,
  controlled stepping, 5420 to 7260;
* condition and chunk evaluation, 7259 to 7570;
* INTERPRET fragments, DROP and the debug pause, 7570 to 7860;
* TRACE, 7856 to 8020;
* ADDRESS, NUMERIC and clause sites, 8023 to 8300;

and outside the impl: indent computation (8300 to 8620), name shape and
indirect-word validation (8625 to 8700), the pure IF/WHEN/SELECT target
arithmetic (8703 to 8880), and the `raised_*` constructors (8884 to 9105).

Proposed: keep `Flow`, `Ended`, `Resolved`, `run_activation`, `apply_flow` and
`exec_instruction` in `run.rs`; move `run/call.rs`, `run/condition.rs` (traps,
RAISE, SIGNAL), `run/loops.rs`, `run/select.rs` (SELECT plus the branch target
arithmetic), `run/interpret.rs` (INTERPRET, DROP, the debug pause),
`run/settings.rs` (ADDRESS, NUMERIC, TRACE), `run/indent.rs` and
`run/raised.rs`.

`run/raised.rs` moves the `run.rs` rows of `refusal-sites.tsv` column 4, counted
in the table above.

`exec_instruction` is one exhaustive match and is not cut: see the section below.

### Rank 4: `rexx-exec/src/lib.rs`, 7123 lines. SPLIT.

Distinct responsibilities:

* `Loud` and its constructors, 221 to 690, one per thing this crate does not
  implement. A catalogue with one purpose, depending on nothing else in the file
  beyond `Interp`. It is the `lib.rs` block of `refusal-sites.tsv` column 4.
* the directive-analysis free functions: `class_references`, `directive_gap`,
  `class_install_order`, the dictionary-key builders, `annotation_target`,
  `instruction_owner`, `expr_owner`, 723 to 1360.
* `struct Interp` and the types it holds, 1474 to 2320.
* directive installation: `install_directives`, requires, namespaces, 2674 to
  3560.
* class, method and attribute installation, and the executable records, 3982 to
  5060.
* variables: read, write, expose, pools, 5059 to 5400.
* garbage collection: `alloc_with`, `collect_if_due`, `object_roots`,
  `collect_now`, `mint_class`, 5401 to 5800.
* the public entry points: `run_program`, `render_ir`, `native_entry_points`,
  `execute`, 5798 to 6148.
* the test module, 6148 to 7123.

**`collect_now` cannot move.**
`dispatch_seam.rs::heap_collect_is_called_from_collect_now_alone` opens
`src/lib.rs` by path, finds `    fn collect_now(&mut self) {` in its text, and
asserts that the crate's one `heap.collect(` call is inside that body. Leaving
the whole GC block where it is, is the simplest way to satisfy that;
`object_roots` could move alone if a task wants it.

Proposed: `lib.rs` keeps the module declarations, the public surface, `Interp`
and its types, and the GC block. Move `refusal.rs` (`Loud` and its
constructors), `directives.rs` (the analysis functions), `install.rs` (directive,
class, method and attribute installation), `variables.rs` and `lib/tests.rs`.

Note the `#[cfg(test)] fn planned_code` at 1411, a test-only item outside the
test module: it has to move with whichever module names it, or stay and be
`pub(super)`.

### Rank 5: `rexx-exec/src/dispatch/hash.rs`, 2992 lines. SPLIT.

Its doc header says "the mapped collections' store, and the two classes that are
the store with nothing on top". The file also holds `Stem`, which is neither:
`stem_tails` through `native_stem_empty`, 1457 to 2130, including a balanced-tree
tail ordering (`TailNode`, `stem_order`, `move_tail`, `rebalance`) that exists
for `Stem` alone. `Relation` is a third, 2469 to 2755.

Proposed: `hash.rs` keeps the store (`Half`, `Store`, `probe`, `insert`, `take`,
`expand`, `walk`) and `Directory`/`StringTable`; `hash/stem.rs` takes the stem
half; `hash/relation.rs` takes `Relation` and `Bag`. Both children need
`CLEARANCE_CONSUMERS` entries, and both hold native bodies moved out of a file
already on the list.

### Rank 6: `rexx-exec/src/dispatch/collection.rs`, 2279 lines. SPLIT.

`Array` (205 to 1360, of which the merge sort is 930 to 1210), `Supplier` (404
to 560), `Queue` (1366 to 1545) and `List` (1545 to 2130) are separate classes
sharing one file. Proposed `collection/array.rs`, `collection/sort.rs`,
`collection/supplier.rs`, `collection/queue.rs`, `collection/list.rs`, with each
chained `NATIVE_METHODS` slice moving with its functions, and a
`CLEARANCE_CONSUMERS` entry per child.

### Rank 7: `rexx-exec/src/eval.rs`, 3338 lines. SPLIT, tests only.

The `#[cfg(test)]` modules are `tests` (1586 to 2719) and `object_operand_tests`
(2723 to 3338). Moving both to `eval/tests.rs` and `eval/object_operand_tests.rs`
leaves 1588 lines of one thing: an `impl Interp` that evaluates expressions, plus
the operator classification predicates that feed it. The impl is a single
responsibility and should not be cut.

### Rank 8: `rexx-parse/src/token.rs`, 648 lines. SPLIT. The trigger misses it.

See "Files the trigger misses". This is the strongest case in the tree for
splitting by responsibility rather than by length.

### Rank 9: `rexx-exec/src/ir/drive.rs`, 2549 lines. SPLIT.

The driver's op loop is 354 to 2302: helper methods around one match over `Op`.
Lines 2328 to 2549 are the instrumentation counters (`count_run_chunk_entry`,
`clause_op_entries`, `trace_op_echoes`, `arith_hint_skips`, `call_site_hits`,
`const_builds`, `load_constant_builds`, `frame_floor_high_water`, and
suspend/resume), which are a separate thing: they are read by tests and by the
bench harness, not by the driver. Proposed `ir/counters.rs`. The op loop is not
cut.

### Rank 10: `rexx-bench/src/bin/rexx-bench-suite.rs`, 1514 lines. SPLIT.

Measurement (149 to 675: `main`, `measure_interleaved`, `Stats`, the median
interval, `fingerprint`, `loop_count`) and report rendering (675 to 1286:
`write_provenance`, `write_offset`, `write_counters`, `write_axes`,
`write_rexxcps`, `write_self_timed`, `write_blocked`) are separate
responsibilities sharing only the row types. Proposed `rexx-bench-suite/report.rs`, which Cargo
resolves for a `src/bin/<name>.rs` binary. Lowest risk in the plan after the test
splits: it is a tool, so no interpreter code moves and the performance constraint
does not reach it.

### Rank 11: tests-only moves, mechanical

For each of these the non-test code is already under the trigger, and the whole
of the split is `#[cfg(test)] mod tests` becoming `mod tests;`. Lines moved and
lines left are in the table above: `rexx-api/src/invoke.rs`, `plan.rs`,
`builtin/numeric.rs`, `dispatch/library.rs`, `parse_template.rs`,
`dispatch/native.rs`, `builtin.rs`, `value.rs`, `builtin/convert.rs`,
`builtin/string.rs` (1170 to 2055 and 2057 to 2084), `builtin/datetime.rs`,
`ir/compile.rs`,
`environment.rs`, `error.rs`, `trace.rs`, `activation.rs`, `docs/classes.rs`.

`activation.rs` (56 lines out), `docs/classes.rs` (190) and `environment.rs`
(259) are small enough that the move buys nothing on its own; they should ride
along with a real split of the same file or be skipped.

### Rank 12: marginal, and worth a ruling because they are close calls

* **`rexx-exec/src/environment.rs`**, 2312 code lines. The seam, the
  `.environment`/`.local` model and `.NAME` resolution are one thing, and pinned
  to this file. Two clusters are not: output routing and trace-line delivery
  (`local_route`, `output_route`, `route_ends_at`, `route_trace_line`,
  `deliver_trace_line`, 628 to 850), and the minting of the interpreter's own
  object identities (`record_packageless_class`, `package_object_for`,
  `method_object`, `define_method_object`, the annotation tables, `find_routine`,
  `library_routine_object`, 1413 to 2100). Splitting those out is legitimate and
  leaves the seam untouched. Rank 12 only because the file reads coherently as
  it stands.
* **`rexx-exec/src/parse_template.rs`**. The PARSE template engine is one thing.
  The interpreter's **version constants** are another, and they are here:
  `PLATFORM`, `LINE_END`, `VERSION`, `VERSION_NUMBER`, `MAJOR_VERSION`,
  `RELEASE`, `MODIFICATION`, `LANGUAGE_LEVEL`, `BUILD_DATE`, `BIT_WIDTH` and the
  `const fn seek`/`const fn part` that slice them, lines 25 to 93. That block has
  nothing to do with PARSE. The file barely trips the trigger and the move is
  small, which is why it is worth doing while the file is open.
* **`rexx-exec/src/trace.rs`**, 1052 code lines. The format half (mode parsing,
  prefix layout, `push_clause`, `push_value`, `push_tagged`, 30 to 538) is pure
  functions on bytes; the emission half is one `impl Interp`, 538 to 1039. Two
  responsibilities. `tests/support/mod.rs` names `push_clause` in prose, and
  `push_clause` stays in `trace.rs` under this split.
* **`rexx-api/src/values.rs`**, 1988 lines with no test module. The types, the
  `Host` trait, `Activation` and `TABLE` are one thing; the per-code converter
  functions, 1212 to 1988, are a second, and they are a flat list of one-screen
  functions. `values/convert.rs` is a clean cut.
* **`rexx-parse/src/instruction.rs`**, 2490 lines. One `impl Inst` recursive
  descent, with self-contained sub-grammars inside it: the loop header (775 to
  1160), ADDRESS and redirection (1161 to 1330), and the PARSE template (2045 to
  2330). Splitting those out with the cursor primitives raised to `pub(super)` is
  defensible; so is leaving it.
* **`rexx-parse/src/ast.rs`**, 1352 lines of node declarations in groups that do
  not overlap: expression (26 to 425), instruction (425 to 1035), directive (1036
  to 1352). I lean leave: it is one declaration catalogue and the groups
  reference each other.
* **`rexx-exec/src/ir/compile.rs`**. The `assert_*` family, 1635 to 1926, checks
  op-stream shape invariants and is called from `compile` at 1104 and 1111 in
  production, not only from tests. `ir/compile/invariants.rs` is a real
  responsibility boundary, but the code half of the file is 1934 lines and drops
  to roughly 1650 without it, so the move is for clarity rather than length.
* **`rexx-extract/src/docs/classes.rs`**, 1147 code lines: class rows (407 to
  650), method rows (648 to 1000), header rendering (1020 to 1149). Splitting it
  needs the `module is` header decision recorded above.

## Files the trigger misses

Enumerated over every `.rs` from 400 lines up, by reading each module doc header
and outline, with the command at the top of this document.

* **`rexx-parse/src/token.rs`, 648 lines. SPLIT.** Its own doc header names the
  subjects: "Tokens, symbol interning, the keyword tables, and the context every
  `parse_*` function is handed." All of them are there, alongside `ParseError`:
  * `ParseError`, 27 to 50;
  * `SymbolId` and `SymbolTable`, 50 to 115;
  * `Operator`, `SymbolClass`, `TokenKind`, `Tag` and `Token`, 115 to 332;
  * `KeywordSet`, `Keywords` and the keyword tables `INSTRUCTIONS`,
    `SUB_KEYWORDS`, `CONDITIONS`, `PARSE_OPTIONS`, `DIRECTIVES` and
    `SUB_DIRECTIVES`, 332 to 558;
  * `ParseCtx` and `TokenCursor`, 558 to 648.

  Proposed `token.rs` (the token types), `token/symbols.rs`,
  `token/keywords.rs`, `token/cursor.rs`, with `ParseError` staying where its
  users expect it. Nothing names this file by path.

* **`rexx-exec/src/parse_template.rs`'s version constants**, described above. The
  file trips the trigger on length, but the split worth making in it is not the
  one the length points at.

* **`rexx-num/src/lib.rs`, 963 lines.** `ArithError`, the error-text
  substitution and an inline `mod substitute_tests`, 53 to 350, sit in front of
  `impl Number`, 366 to 959. Two responsibilities under the trigger.
  `unsafe_sites.rs` asserts this file's path exists, so a split has to leave
  `lib.rs` in place, which it would. Marginal: the error half is small and the
  file reads as one crate root.

* **`rexx-exec/src/redirect.rs`, 918 lines.** The doc header names the
  configuration and the per-command context, but the second is one `impl Interp`
  of 670 lines and the first is the types it operates on. Leave.

Nothing else from 400 lines up named more than one subject in its doc header and
then turned out on reading to have more than one.

## Files where a split would make things worse

Named so a later task rules them out once rather than rediscovering them.

* **`rexx-exec/src/error.rs`'s `impl Raised`**, 116 to 1795.
  `awk 'NR>=116 && NR<=1795' crates/rexx-exec/src/error.rs | grep -c '^    \(pub(crate) \)\?fn '`
  is 185, almost all under fifteen lines, each constructing one Rexx condition.
  It is an alphabet, not a program: any partition into families is opinion, the
  order is not by family, and `refusal-sites.tsv` names most of them. The
  `Failure`, `FailureSite` and `ClauseSite` block, 1838 to 2153, is a genuinely
  separate responsibility and is the only cut in this file worth considering.
* **`rexx-exec/src/run.rs`'s `exec_instruction`**, 980 to 1590: one exhaustive
  match.
  `awk 'NR>=980 && NR<=1590' crates/rexx-exec/src/run.rs | grep -c '^            InstructionKind::'`
  is 33. Cutting it hides the exhaustiveness.
* **`rexx-exec/src/ir/drive.rs`'s op loop**, 354 to 2302: one match over `Op`
  with helpers around it.
  `awk 'NR>=354 && NR<=2302' crates/rexx-exec/src/ir/drive.rs | grep -cE '^\s+Op::'`
  is 51. Same reason.
* **`rexx-exec/src/builtin.rs`'s `IMPLEMENTED` table**, 78 to 619: one table
  whose rows are the crate's answer to which builtins exist and with what arity.
  The rest of the file's code is the resolution and argument coercion that reads
  it.
* **`rexx-api/src/values.rs`'s `TABLE`**, 907 to 1165, and
  **`dispatch/native.rs`'s `LIBRARY_REXX_METHODS`**, 115 to 431: tables that must
  stay beside the `const fn` row builders that construct them.
* **`rexx-parse/src/scanner.rs`**, 1073 lines: one `Scanner` state machine with
  two literal packers after it.
* **`rexx-parse/src/expr.rs`**, 1043 lines: one recursive-descent expression
  parser plus its precedence table.
* **`rexx-parse/src/directive.rs`**, 1117 lines: one `impl Dir` parser plus its
  keyword index constants.
* **`rexx-num/tests/format.rs`**, 1051 lines: the cases for one function, which
  the plan's own Global Constraints already name as a reason to stay large.
* **`rexx-exec/src/ir/golden_tests.rs`**, 1647 lines: golden transcripts for one
  subject, `ir::compile`.
* **`rexx-exec/tests/coverage.rs`**, 1509 lines: dominated by the
  `EXPECTED_SUBSET*` lists, which are committed data the tests beside them
  compare against. Moving the data one module away separates an assertion from
  the thing it asserts.
* **`rexx-exec/tests/method_bodies.rs`**, 1212 lines: one table (D76) and the
  probe that fills it.
* **`rexx-classes/tests/native_classes_wiring.rs`**, 1865 lines: recorded method
  sets for the native class bootstrap, asserted in the test bodies that hold
  them.
* **`rexx-api/src/ffi.rs`**, 1041 lines. This is the one the plan asks about
  directly, and the answer is that moving safe code out is not enough, because
  there is no safe code to move. Measured with the predicate `unsafe_sites.rs`
  itself applies, comments stripped first:

  ```
  awk -v a=467 -v b=848 'NR>=a && NR<=b {sub(/\/\/.*/,""); print}' crates/rexx-api/src/ffi.rs \
    | grep -c -E 'unsafe \{|unsafe fn|unsafe impl|unsafe trait'     ->  15
  awk -v a=850 -v b=1041 'NR>=a && NR<=b {sub(/\/\/.*/,""); print}' crates/rexx-api/src/ffi.rs \
    | grep -c -E 'unsafe \{|unsafe fn|unsafe impl|unsafe trait'     ->   6
  ```

  Lines 467 to 848 are `#[cfg(test)]`-gated stubs (`reading_stub`,
  `dropping_stub`, `thread_table_stub`, `instance_stub`, `refusing_stub`,
  `nesting_stub`, `cstring_stub`, `arglist_stub`, `name_result_stub`,
  `numeric_stub` and their type arrays). Both they and `mod tests` contain
  `unsafe` in exactly the forms that test matches, so moving either to a child
  file puts a new path into the `uses` list and turns the test red. An opt-in on
  the parent does not help: the test scans files, and `rust/CLAUDE.md` says the
  two assertions are separate for that reason. Even if the stubs could move,
  `mod tests` alone leaves 848 lines, still over the trigger.
  **Recommend: leave `ffi.rs` entirely, and drop it from Task 8.**

## What the plan's later tasks assume that does not hold

1. **Task 8's `ffi.rs` instruction is not satisfiable.** "only by moving safe
   code out (the D-U1 constraint; stop and ask if that is not enough)" has its
   answer above: it is not enough, and there is no safe code to move. The stop
   is here rather than in Task 8.
2. **The D-U1 statement in Global Constraints is incomplete.** It omits
   `rexx-core/src/bytes.rs` from the `unsafe` uses list and
   `rexx-core/src/lib.rs` from the opt-in list.
3. **Items are pinned to `src/dispatch.rs` that the plan does not mention**:
   `mod seam`; `method_is_protected`, whose occurrences must both be in a path
   containing the literal `dispatch.rs`; and the single `seam::clear(` call
   inside `Interp::invoke`.
4. **`collect_now` is pinned to `src/lib.rs`**, by name and at four-space
   indentation, which Task 4 has to plan around.
5. **`refusal-sites.tsv` column 3 is path-derived**, so "column 4 only may
   change" holds for a dispatch split only because `/dispatch/` and
   `dispatch.rs` carry the same tag. That belongs in the task rather than being
   relied on.
6. **Task 10's list is mostly leaves.** `rexx-num/tests/format.rs`,
   `coverage.rs`, `native_classes_wiring.rs`, `method_bodies.rs` and
   `ir_recorded.rs` all come back as the plan's own "one set of test cases for
   one function" exception. `rexx-bench-suite.rs` and `gate_table_c.rs` are the
   ones in that task worth doing.
7. **`ir_recorded.rs` has a different defect from the one this plan treats.**
   Its `LOOP_CASES` and `BRANCH_CASES` are inline `const` case tables in Rust
   source, lines 60 to 688. A recorded preference of Moritz's is that
   inline-case tables belong in a data file the test reads rather than a `const`
   in the source. That is a behaviour-visible change, outside this plan's
   pure-move architecture, so it is raised here rather than folded in.
8. **Task 5 lists `error.rs` and `activation.rs` as split targets**, and both
   come back leave-or-nearly: `error.rs`'s only defensible cut is the
   `Failure`/`FailureSite` block, and `activation.rs` is 1216 lines of one thing
   with a 56-line test module.
9. **Task 2 through Task 10's ordering puts the riskiest file first.** Rank 2
   and Rank 10 here (`run/tests.rs` and `rexx-bench-suite.rs`) move no
   production code and would exercise the four instruments and the performance
   procedure before `dispatch.rs` depends on them working.
