# Task 2, fix round 2 — rulings on the re-review

Re-review: `.superpowers/sdd/2026-08-17-phase-5a/task-2-rereview.md`. **All nine findings CLOSED**,
and the ruling worked where it was aimed: **54 of 54 counts re-run reproduce**, and both rows you
found on your own verified. Seven new defects — one high, three medium, three low.

---

## The meta-defect, and it is the ruling's own shadow

The round's job was to put the procedure beside every claim. **Three of the seven new defects are
procedures that were written down without being run as written.**

* **N2** — the row's own `grep` has no `-E`, so under BRE `[[:space:]]+` wants a literal `+`. Run
  exactly as stated it returns **zero files**, and the intersection is empty before `activate` is
  ever consulted. The conclusion survives; the stated reason for it does not.
* **N3** — the derivation's `awk` prints `NR`, which is cumulative across files, not `FNR`. Run as
  written it emits `StreamClasses.orx:4703` for a file of 1010 lines. **It produces strings shaped
  exactly like citations, naming lines that do not exist.**
* **N7** — every `|` in a mapping-table pattern is written `\|`. That is correct GFM table escaping
  and renders right, **but a later task reads this document with `sed` or `cat`, not a renderer**, and
  `/bin/grep -E` treats `\|` as a literal pipe. Pasted from the raw file, three counts return 0 and
  exit 1 — reading as upstream drift when nothing moved, which is the exact misreading this document
  exists to prevent.

**Ruled: a procedure is not written down until it has been run verbatim, from the raw file, and its
output pasted or reproduced.** If the output is in front of you, the command ran. That is the whole
test, it is cheap, and it converts "I stated the method" into "the method works" — which were quietly
the same sentence last round and are not.

**For N7 specifically:** patterns must be pasteable **from the raw text**. Move any pattern containing
`|` out of a table cell into a fenced code block beside the row, where no escaping is needed. Do not
solve it by explaining the escaping — a reader who pastes does not read the explanation first.

---

## Finding by finding

**N1 — HIGH. Fix, and it is the most valuable thing in this round.** `ootest/ooRexx/base/class/class.testgroup.cls`
is a file upstream wrote **for nothing but this mechanism**, driven by `Class.testGroup:962`
`test_activate`. It pins, with its own failure messages: every class object existing before any
`activate` runs (asserted three times over, strictly stronger than `REQUIRES.testGroup:320`);
**activate order following the dependency graph** — `class2`, then `class3`, then `class1` which
subclasses `class3`; an instance being constructible and sendable inside `activate`; and the package
prolog running after every `activate`.

**Activate ordering is pinned upstream and no row in the ledger mentions it.** Add it.

**The mechanism of the miss is `--include='*.testGroup'`** — `ootest/` also holds `.rex`, `.cls`,
`.testUnit` and `.oodTestGroup`, and these assertions live in the `.cls`. **This is the third
instance of "the search was narrower than the sentence it licensed", and it is in the rows you
rewrote to fix the first two.** Widen the extension set wherever a row searches `ootest`, re-run
every surviving negative under the wider set, and state the extensions in the pattern. Row `:137`'s
"no test group carries both an `::method init class` and an `::method activate`" is true only by file
extension and must go.

**N2 — MEDIUM. Fix.** Add `-E`. Say that the conclusion survives and under what restriction, since
with `-E` it returns 26 files and the intersection over those is still empty.

**N3 — MEDIUM. Fix.** `FNR`, not `NR`. Also correct the third file's path: it is
`interpreter/platform/unix/PlatformObjects.orx`, not under `RexxClasses/`, so the file list as
written cannot be pasted from either directory. The finding-8 fix itself is right — the `awk` does
find all three double-quoted directives, they are all `File`'s, the conclusion is unchanged.

**N4 — MEDIUM. Fix.** Row `:120`'s new `test_issubclassof (:167)` is **the mis-attribution finding 3
corrected, re-introduced two rows away.** The right line is `:146`; `:167` is
`test_issubclassof_non_class`, which never mentions `AmphibianVehicle`. Beware the decoy: a second,
byte-similar `::method test_issubclassof` at `:1248` sits after `::CLASS "WasserFahrzeug"` and is
that class's method, not the test. The substantive claim holds.

**N5 — LOW. Fix.** Row `:153`'s enumeration of what its search returns omits three hits in
`base/rexxutil/Macrospace.testGroup`. A third subject, so the conclusion is unaffected — but the
enumeration is a checkable claim and it is wrong, and a re-runner has to decide whether the extra file
means the row is stale.

**N6 — LOW. Fix.** List (a)'s criterion sentence says its members "resolve to the definition line of
the named function or the named declaration", and its `Setup.cpp` members are statements and macro
invocations inside the image build. Say what the list actually is.

**The observation — fix it, it is not merely an observation.** `Setup.cpp:325-329` is cited by the
spec at `:1359` and appears in **neither** list. The C++ row's stopping point is *"every `file:line`
the spec cites, verified"*, so a cited line in no list is a hole in the stopping point, not a
bookkeeping nit. **The conservation check ran over the old list rather than over the spec** — re-run
it against the spec's citations, which is what the stopping point names, and place every one.

---

## Verification

Every command you state, run verbatim from the raw file first. The five gate commands, each with its
own exit status; corpus **106 of 106**. `REXX_PHASE_GATE=5a …` stays unrun, per the standing answer.

Append to `task-2-report.md` under "Fix round 2".

**Return only:** status, commit SHAs, one line per finding, and anything you could not close.
