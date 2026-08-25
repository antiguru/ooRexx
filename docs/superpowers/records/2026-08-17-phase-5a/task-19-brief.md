## Task 19: `::CONSTANT`'s instance method and class method

**Goal.** M7 and D38. A `::CONSTANT` creates **both** an instance method and a class method, and the
value is readable.

**Measured.** `.K~c` with `::CONSTANT c 42` is `42` on the oracle and **97.1 rc 159 here**; with
`::CONSTANT c (1+1)` it is `2`, and `.K~hasMethod("C")` is `1`. The old plan's Task 4 evaluated the
expression and stored nothing readable, so the directive installs and answers nothing.

**And the instance-side method M7 owes is observable class-side, which decides what this task's
control can be.** Measured, `.K~method("C")` is **`a Method`** at rc 0 -- `~method` reads the class's
own *instance* dictionary (`ClassClass.cpp:984`, Task 9), so the getter installed on the instance
behaviour answers there with no `~new` anywhere. The contrast is measured on the same build:
`.K~method("M")` for a class-**only** method raises `97.1` with the
`Compiled method "METHOD" with scope "Class".` frame. Crate today answers `0` to `.K~hasMethod("C")`
and then rc 120 at `~method`.

**Build.** The constant getter on both behaviours (`createConstantGetterMethod`); the parenthesised
form running **as a method against the class object with `self` bound** (`resolveConstants`'
`setScope`), in Task 18's second pass; the refusal of forward references to *calculated* constants;
and the floating-form rule, which already agrees at `99.906` rc 157 -- **and it is the
*parenthesised* floating form, measured**: `::CONSTANT c (1+1)` with no `::CLASS` is rc 157 on both
sides, byte-identical, `99.906 A ::CONSTANT directive with an expression requires a matching ::CLASS
directive.`, while a plain floating `::CONSTANT c 42` is **rc 0 on both** and refuses nothing.

**Verification, runnable now.** Oracle-differential on both constant forms read back through `.K~c`
and through `.K~new`... **no**: the instance arm needs `~new` and is 5b's. Here the readback is
`.K~c`, `.K~hasMethod("C")` and `~method("C")`'s own-scope answer; **the instance-side reading is
recorded as 5b's debt** beside the old plan's Task 7 debt, not implied to be covered. Plus the two
existing failure shapes staying green: the expression raising (rc 159, 97.1) and the class-less
structural refusal (rc 157, 99.906), and the blame-the-last-installed-class rule the old plan's Task 8
fix round pinned with two programs that only work as a pair.

`phase-4-exclusions.txt:517`'s KNOWN GAP -- a `::CONSTANT` expression cannot send to a class -- is
this task's starting point and moves to CLOSED DEFECTS in its own commit.

**Done when** both constant forms read back on both engines, both failure shapes stay byte-identical,
and the control is recorded **on the probe that can see it**: creating only the class-side getter
leaves `.K~method("C")` raising `97.1` where the oracle answers `a Method`, so that program reddens.
**`.K~c` cannot be the control's subject** -- a class-side getter alone answers it `42`, and the
control would then do the same thing whether or not M7's instance half was built, which is this
plan's own definition of decoration. Sitting required.

---

