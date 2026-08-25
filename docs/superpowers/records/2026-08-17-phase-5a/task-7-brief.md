## Task 7: `::CLASS ... MIXINCLASS` and `::CLASS ... INHERIT`

**Goal.** Both keywords install, on both engines, with the documented merge order.

**Why here.** The class graph is what everything else in the phase hangs on, `CoreClasses.orx` uses
both (`DateTime`, `TimeSpan`), and D44's witness in Task 8 needs `MIXINCLASS Class`.

**Build:**

* `ClassKind::Mixin` reaching the directive: `install_class` hard-codes `Regular`, and
  `ClassGraph::inherit` with its validity assertions and `update_sub_classes` cascade already exists,
  so the graph work is largely present and the wiring is the shape of `d988b2632`.
* **`install_class_at` orders on `subclass` alone.** An `INHERIT a b c` target is equally a
  dependency, so the dependency **set** grows to the inherit list or a forward `INHERIT` target does
  not resolve. Cycle detection generalises unchanged.
* **Merge order**: a class's own methods precede its superclasses' **and its mixins'**; among several
  `INHERIT`s, leftmost first (a reverse walk with `addFront`).
* **M10.** The narrowing must be written against the mixin flag, not `subclass.is_some()`.
* **`~baseClass`**, which belongs to Task 9's protocol by subject and to this task by reachability: it
  is the mixin's own observable and table D's `MIXINCLASS` discriminator, and a differential written
  against it in this task cannot run unless this task lands it. Task 9's list does not repeat it.

**Verification, runnable now.** Measured, oracle rc 0 and crate rc 120 today, so both sides reach the
comparison the moment the refusal lifts:

* the spec's merge-order program -- `::CLASS P` with `::METHOD m CLASS`, `::CLASS M1 MIXINCLASS
  Object` with its own `m`, `::CLASS K SUBCLASS P INHERIT M1`, and **`K` defining no `m`**. `say .K~m`
  is **`parent`**. The mixin winning is its negative control; the variant where `K` defines its own
  `m` answers `own` under *any* merge order and is green against a build that has this backwards.
* `.M~baseClass` is `The Object class`, `.P~baseClass` is `The P class`.
* a class inheriting two mixins where the order is observable, and the diamond.
* the refusal ladder, all three shapes: `::class d inherit c` with no `c` is `98.909 Class "C" not
  found`; with `::class c` declared it is `98.942 ... must be a MIXINCLASS for INHERIT`; and
  `::CLASS K INHERIT M PUBLIC` is `98.909` -- `dire.xml`'s "INHERIT must be last" is **not** a syntax
  check, the arm consumes to end of clause and reads `PUBLIC` as a class name. Re-measure each on a
  current build rather than inheriting the old plan's numbers.
* **the frame line on the `98.942` shape, which is this task's and not Task 6's.** Measured, that
  refusal opens `       *-* Compiled method "INHERIT" with scope "Class".` -- a native method invoked
  by the **install machinery**, not by an expression or an operator. Task 6 owns the operator half and
  the old plan's Task 5 landed the explicit-send half; a directive-initiated send is a third caller and
  it lands here, because this is the task whose own acceptance needs those bytes.

* **The UNINIT propagation flags.** The spec's cell reads "5b, flags carried by 5a's code", and the
  code it names -- `subclass`, `mixinClass`, `inherit` -- is this task's. So **this task carries the
  flags**, and 5b owns everything that reads them, `UNINIT` firing included. **The reason this task
  takes no differential is that the firing is 5b's, not that a flag cannot be observed** -- it can be,
  class-side, with no `~new` anywhere, because a class object is an instance and is destroyed like
  one. Measured, one process each, three descriptors separate:

  * `::CLASS K` carrying a `::METHOD uninit CLASS` that says its `self~id` -- oracle rc 0, `main` then
    `uninit on K`; **crate rc 0, `main` alone, which is a silent divergence today**;
  * the same method on `::CLASS P` with `::CLASS K SUBCLASS P` -- oracle `uninit on K` then
    `uninit on P`, crate the same silent `main`;
  * the same method on `::CLASS M MIXINCLASS Object` with `::CLASS K INHERIT M` -- oracle
    `uninit on K` then `uninit on M`, crate rc 120 at `MIXINCLASS`. That third one is this task's own
    code path: `ClassClass.cpp:1364` is
    `if (mixin_class->hasUninitDefined() || mixin_class->parentHasUninitDefined())` inside `inherit`.

  So **the instrument here is an in-crate test that the flag propagates through all three
  constructors, and nothing else** -- no differential, no gate row -- and **the three programs above
  travel with the row to 5b**, named in Task 24's handover, so 5b starts from a witness rather than
  hunting for one. Named here so the split has both halves owned rather than being handed whole to
  5b, which is the D46 violation the spec exists to prevent.

**What it changes about refusals, and the instrument.** `MIXINCLASS`'s and `INHERIT`'s current
refusals are held by `every_directive_this_crate_cannot_install_refuses_before_the_first_clause`, and
**a corpus program can never hold them** -- our rc 120 against the oracle's rc 0 is not expressible as
a differential. When those rows move, table D's rows are what can fail in their place, and the task
records that the replacement can fail the same way the deleted rows could.

**And it converts one loud refusal into a silent wrong answer, which is a debt this task creates
rather than one it inherits.** A class-side `::METHOD uninit` on a mixin the program `INHERIT`s is
`rc 120 ::CLASS MIXINCLASS is not implemented (Phase 5)` today -- loud, and impossible to mistake for
an answer. After this task the directive installs and the program runs to `rc 0` printing nothing,
where the oracle prints `uninit on K` and `uninit on M`: **5b owns the firing, so the gap is correct,
but its shape changes from a refusal to silence.** Record it in this task's report so the 5a gate's 5b
handover carries all three programs rather than the two that are silent before this task lands.

**Done when** the merge-order program answers `parent` on both engines, the diamond is green, all
three refusal shapes match byte for byte including the `98.942` frame line, `METACLASS` still refuses
loudly with a message naming only what it still refuses, the UNINIT flag propagation has its in-crate
test, and **three controls are recorded as run**:

* walking the chain instead of merging reddens the diamond;
* **table D's mutation 1, relocated here** because this is where its discriminator first answers: the
  M10 narrowing written against `subclass.is_some()` makes `.M~baseClass` answer `The M class` where
  the oracle says `The Object class`, so the `MIXINCLASS` row goes from `agree` to `diverge-stdout`;
* dropping the flag propagation from one of the three constructors fails the in-crate test -- the only
  thing that can see it.

Sitting required.

---

