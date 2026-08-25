## Task 23: the bootstrap milestone

**Goal.** D26 and D47. `CoreClasses.orx`, `StreamClasses.orx` and `PlatformObjects.orx` install and
`CoreClasses.orx`'s prologue runs to its `exit`, driven from `rexx-lib`.

**It is a milestone inside 5a, not the phase boundary and not the gate.** 5a is the phase that
completes every mechanism the three files need, which is why this task can be last-but-one; a phase
scoped to what those files happen to touch would leave `UNKNOWN`, Required String Values, Object
Destruction, `PACKAGE` and `DELEGATE` owned by nobody.

**Build.**

* `rexx-lib`: the three files embedded at build time from this repository's tracked copy by a
  workspace-relative path, following `crates/rexx-inventory/build.rs`, each with a recorded sha256 so
  drift is a build failure. **Never edit them.**
* The driver. **`CoreClasses.orx` is not a main program**: measured, running it directly is rc 159 on
  the oracle, `Object "REXXPACKAGE" does not understand message "ADDCLASS"`, because `use arg
  rexxPackage` with no argument leaves the symbol as its own name. The bootstrap **calls** the file
  with a Package object, as `Setup.cpp:1795` passes `TheRexxPackage`. This task says what supplies
  that argument.
* Program-name resolution for the two `CALL`s, intercepting the three bootstrap names ahead of the
  external file search, which is Phase 7's.
* **When the driver runs, which decides whether the wiring half can ever be green.** The classes
  `CoreClasses.orx` and `StreamClasses.orx` declare are most of the documented class set, and a
  wiring probe is an ordinary program: unless the bootstrap runs at interpreter start, `.Alarm~id`
  answers nothing in a program that did not ask for it. So the driver runs at start, as the C++ does
  from its image -- and since D26 builds no image, that cost lands on every run, which is what makes
  Task 24's cold-start measurement a real number rather than a formality.

**Verification, and its limit is structural rather than incidental.** **There is no oracle transcript
for the prologue** (D39): `removeSetupMethods()` deletes two methods the prologue calls, so no
shipped-oracle run reaches its `exit`. What is checkable, and all of it is:

1. the driver's exit status, empty stdout, empty stderr, on both engines;
2. the post-bootstrap state answering Task 9's wiring questions for every class the prologue installs
   **and for every native class it mutates** -- which is table C's wiring half, re-run against a
   bootstrapped state rather than a fresh one. The mutated ones are easy to forget because no
   `::CLASS` names them: measured, `.Queue~superClasses` is `The Object class` and
   `The OrderedCollection class` and `.Stem~superClasses` is `The Object class` and
   `The MapCollection class`, from `CoreClasses.orx:99` and `:110`, and those are the halves of
   Task 21's rows that only this task can close;
3. **`.TraceObject~option` is `N`**, which separates a bootstrap that ran `ACTIVATE` from one that did
   not;
4. the donated method sets on `.Supplier`, `.Set`, `.Bag` and `.Relation`, which separate a real
   `inheritInstanceMethods` from a no-op -- measured in the old plan: `.Supplier~superClasses` is
   `The Object class` alone while `.array~of(1,2)~supplier~hasMethod("ALLITEMS")` is 1, so **a class
   graph assertion cannot see this and only a method set can**;
5. the setup methods gone: `.String~hasMethod("DEFINECLASSMETHOD")` and
   `.Supplier~hasMethod("INHERITINSTANCEMETHODS")` are 0, `.Class~hasMethod("DEFINE")` is 1, and
   `.String~defineClassMethod(...)` is rc 159 with 97.1 -- the only instrument the setup methods have.

**Which `PlatformObjects.orx`.** Both `interpreter/platform/unix/` and `interpreter/platform/windows/`
are tracked, and CI runs five platforms. This task embeds **unix's**, whose content is one comment
line, and states that the windows file is unread and unembedded -- the spec flagged the same thing
under what it could not check, and a plan that says "the three `.orx` files" without naming which
leaves a CI platform to discover it.

**Done when** the driver exits 0 with empty stdout and stderr on both engines, all five checks above
match the oracle, and a control is recorded: **suppressing the `ACTIVATE` pass** leaves
`.TraceObject~option` at `OPTION` instead of `N`, which reddens check 3 -- the check that exists
precisely because a bootstrap that skipped `ACTIVATE` otherwise looks identical. Sitting required.

---

