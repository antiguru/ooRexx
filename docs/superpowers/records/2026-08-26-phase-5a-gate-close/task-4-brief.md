## Task 4: `.RexxInfo`

**Goal.** `corpus/gate-tables/classes/rexxinfo.rex` agrees.

**Measured, and it is smaller than "build the RexxInfo class".** Oracle **rc 159**: it prints
`entry a RexxInfo` and `class-of-entry RexxInfo`, then raises
`97.1 Object "a RexxInfo" does not understand message "ID"` on the third line -- because
**`.RexxInfo` is an instance, not a class**. Also measured: `.RexxInfo~class~id` is `RexxInfo`,
`.RexxInfo~class~superClass~id` is `Object`, and `.RexxInfo~string` is `a RexxInfo`.

So the row needs a `RexxInfo` **class object that no environment symbol reaches**, and a pre-built
**instance** of it under `.RexxInfo` -- which is exactly what the spec says twice and what
`native_classes.rs`'s `DEFERRALS` records: `RexxInfo` is `addToSystem`-only
(`EndSpecialClassDefinition`), and only the instance is `addToEnvironment`'d (`memory/Setup.cpp:1737`).

**Build.** The class, off the registry's environment-reachable path; the instance in `.environment`;
and nothing else. The instance answers `~class` and `~string` and refuses everything else with the
oracle's own 97.1.

**Done when** the row agrees on both engines, `DEFERRALS`' `RexxInfo` entry is retired or narrowed to
what still stands, and a control is recorded: registering the class under `.RexxInfo` instead of the
instance makes `entry` read `The RexxInfo class` and the row reddens.

---


## Controller addendum, measured at `637d0dd64` before dispatch

**Treat the brief's "what is already there" paragraph as a claim to re-measure, not a premise.** It
has been wrong twice on this plan, both times in the direction of making the task look smaller.
Correct the plan file where you find it wrong, not this brief and not your report.

### The Build paragraph's last sentence is false, and following it introduces the exact defect this plan warns about

It says the instance "answers `~class` and `~string` and refuses everything else with the oracle's
own 97.1." **The oracle does not refuse everything else.** Measured, oracle rc 0:

```
cands = 'PACKAGE DIGITS FORM FUZZ LANGUAGELEVEL VERSION INTERNALDIGITS ARCHITECTURE MAJORVERSION',
      ' RELEASE REVISION DATE PLATFORM FILESEPARATOR PATHSEPARATOR ENDOFLINE LIBRARYPATH',
      ' CASESENSITIVEFILES DEBUG EXECUTABLE ID STRING CLASS OBJECTNAME COPY'
```
of those candidates the instance answers `hasMethod` **1** for all but `ID` and `FILESEPARATOR`, and
they are live rather than merely present -- oracle rc 0:

```
say .RexxInfo~languageLevel   ->  6.06
say .RexxInfo~digits          ->  9
say .RexxInfo~form            ->  SCIENTIFIC
say .RexxInfo~fuzz            ->  0
say .RexxInfo~internalDigits  ->  18
say .RexxInfo~objectName      ->  a RexxInfo
```

`ID` is the one the row asks for and the one that raises, which is why the row is rc 159.

**So answering 97.1 for `~version` or `~digits` would be a silent wrong answer** -- the oracle
understands those messages perfectly well, and 97.1 says it does not. That is the worst defect class
on this project, no gate sees it, and the brief as written instructs you to build it.

**Ruling: the unbuilt part of the surface refuses LOUDLY, at `NOT_IMPLEMENTED_EXIT` rc 120, never
with 97.1.** A loud refusal is an honest "not built here"; a 97.1 is a false claim about the
language. Scope is unchanged -- the plan's "and nothing else" stands, and you are not being asked to
build the other methods -- only the refusal's shape is corrected. `ID` keeps its genuine 97.1,
because there the oracle really does not understand the message.

### The version identity is already committed and already matches

`parse version v` is **byte-identical** on the oracle and `ir` today, rc 0 both:
`REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026`. So if you find that a handful of these read off the
same constants the crate already holds, say so in your report with the measurement -- that is a
finding for the consolidated review to size, not a licence to widen this task.

### Measured, for the row itself

```
say 'string'   .RexxInfo                    ->  a RexxInfo
say 'classid'  .RexxInfo~class~id           ->  RexxInfo
say 'superid'  .RexxInfo~class~superClass~id->  Object
say 'isa'      .RexxInfo~isA(.Class)        ->  0
```

`.RexxInfo~isA(.Class)` being **0** is the row's own check that you built an instance and not a
class. Note the row file is **re-derived by `gate_table_c.rs` on every run and compared in both
directions**, so it is not yours to edit by hand -- if its content must change, the derivation is
what changes.

**The hazard, restated.** Every task on this plan turns a loud refusal into an answer, which is
exactly where a silent wrong answer at rc 0 is introduced, and no gate sees one. Probe past the row.
