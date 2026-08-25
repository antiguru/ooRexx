## Task 17: the environment search order, the Directory entry methods, and `.METHODS`

**Goal.** M6 and the reachable half of M5: the environment-symbol order this phase can observe, the
Directory entry-method mechanism, and the `.METHODS` join.

**What this task does not build, and why M5 is split.** The spec's migration item M5 says this order
"gains the package-local directory as a step above `.LOCAL` and `.ENVIRONMENT`". **The spec's own
enumeration puts that step in 5c**, three separate times: the eight-step row reads "5a; steps 3 and 5
with 5c", the `.Package` row reads "5a; `~local` 5c", and 5c's contents list names `Package~local` and
the two package-crossing steps. Read at the file, `rexxpg/classes.xml`'s `searchord` makes the
package-local directory **step 5**. So M5 and the enumeration disagree, and **the disagreement is not
free to resolve in M5's favour: the step would be unobservable.** The only route that puts anything in
that directory is `Package~local` -- measured, `.context~package~local~MYTHING = "from package local"`
then `say .MYTHING` is oracle rc 0 `from package local`, crate rc 120 -- and `~local` is 5c's, with
Task 21 building `Package` without it. Building a step nothing can populate is unverifiable work, and
an earlier draft of this task built it and deferred it in the same bullet.

**So M5 splits, and both halves are owned:** its Directory entry-method half is 5a's and is below; its
package-local step is **5c's**, recorded there with this measurement so 5c inherits a decision rather
than a gap. If a ruling moves `Package~local` into 5a, this step comes with it and must be sequenced
*before* this task, not after.

**Build:**

* **The order this phase can observe**: the running package's own class table first (landed with the
  old plan's Task 6), then `.LOCAL`, then `.ENVIRONMENT`. Measured, and it is a live divergence rather
  than a formality: with `MYTHING` set in both directories, `say .MYTHING` is oracle rc 0 `from local`
  against **crate rc 120** today, because setting the entry is itself a send to one of the
  interpreter's own objects. It needs Task 11's Directory `~put`, which precedes this task.
* **The Directory entry-method mechanism.** Measured, `.environment~local~class` is
  `The Directory class` while `.environment~hasMethod("LOCAL")` is **0** -- the entry answers through
  `DirectoryClass`'s own `methodTable`/`unknownValue`, not through the behaviour. Crate today: rc 120.
* **The `.METHODS` join** `dire.xml` states three times: a floating `::METHOD`/`::ATTRIBUTE`/
  `::CONSTANT` is reachable through `.METHODS`, which `CoreClasses.orx:73` depends on, inside the loop
  at `:69`-`:74`. Measured: `.methods~class` is `The StringTable class`, **and a StringTable answers an
  entry name sent as a message just as a Directory does** -- `.methods~z` and `.methods["Z"]` both
  answer `a Method`, an absent name answers `The NIL object` rather than raising, and
  `.methods~hasMethod("Z")` is `0`. Crate today: rc 120 on both forms.
* **The agreement to preserve:** a package with no floating directive at all leaves `.METHODS` and
  `.ROUTINES` resolving to their own literal spellings, rc 0 and byte-identical on both engines today.
  That is a row to keep green, not a gap.

**Verification, runnable now.** Oracle-differential on each of the above, both engines, plus the two
existing `VALUE` routes staying green. The `.methods~z` program is what Task 21's REXX_DEFINED probe
sends, which is why that task follows this one.

**The seam's second site is this task's to keep.** D45 and restated criterion 5 require two asserted
chokepoints, and `tests/environment_seam.rs` is site two: it pins one `env_seam::admit(` call, one
`env_seam::directory(` call, and a bound on what lives inside `mod env_seam`. **This is the task that
changes that module**, so it is the task that can break the assertion. It fails loudly rather than
silently, which makes this an obligation to state rather than a hole to close.

**What it cannot see.** Nothing here exercises a package-crossing lookup: steps 3 and 5 have no
program in this phase, and step 5 has none in any 5a shape at all, for the reason above.

**Done when** the `.LOCAL`-over-`.ENVIRONMENT` program, the Directory entry-method program and both
`.methods` forms match byte for byte on both engines, both seam assertions still hold, and two
controls are recorded: resolving `.MYTHING` from `.ENVIRONMENT` first reddens the shadowing program,
and answering `.environment~local` through the behaviour -- which would make `hasMethod("LOCAL")` `1`
where the oracle says `0` -- reddens the entry-method program. Sitting required.

---

