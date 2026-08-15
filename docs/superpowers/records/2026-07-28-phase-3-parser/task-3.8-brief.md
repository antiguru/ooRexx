## Task 3.8: Errors with number, sub-number and line

**Files:**
- Create: `rust/crates/rexx-parse/src/error.rs`
- Test: `rust/crates/rexx-parse/tests/errors.rs`

**Interfaces:**
- Consumes: `ProgramSource::line_of` from Task 3.2 — note the method is `line_of`,
  not the `position` an earlier draft named, because there is no column — and
  `rexx-inventory`'s message table.
- Produces: the completed `ParseError { code: u16, sub: u16, byte: usize, subs: Vec<String> }` — the same shape Task 3.3 defined, no field added or removed —
  with `message(&self) -> String` rendered from the generated table.

**This is the phase's gate, not a finishing touch.** Model it on `rexx-num`'s error
work, which is already done and reviewed: carry the substitution *values* and render
on demand, because a rendered `String` cannot be un-spliced.

**But read the scope carefully, because it changed after this task was written.**
Byte-exact message text and substitution values are **not gated** — that was a
deliberate decision, and error 36's byte-position substitution is not produced at
all. What is gated is the number and the sub-number on a plausible line, in **both
directions**, over the corpus criterion 4 now names.

That leaves a question this task must settle rather than inherit: **`ParseError.subs`
exists and is never populated.** Every construction site passes an empty vector. A
field nothing sets reads as a contract, which is the reason Task 3.6 and Task 3.7b
were told not to ship `next` and the jump targets before they could fill them. So
either fill `subs` where the parser already has the values — which makes `message()`
render readable text and costs little, since the table is generated — or remove it and
render without splicing. **Do not leave it empty and unremoved.** Producing a message
was never dropped; only differentially testing its text was.

- [ ] **Step 1: Collect ground truth — and note the obvious recipe does not work**

**The corpus is defined by criterion 4 now, not by this step.** Build both sets it
names: the **soundness** set, being every program the crate's own tests assert an error
on (currently 385, extractable by instrumenting the `ok`/`err` helpers in
`instruction/tests.rs`, `directive/tests.rs` and `block/tests.rs`), and the
**completeness** set, being that corpus plus the 301 `samples/` files and both
bootstrap files. Report both counts. Criterion 4 was rewritten after this task's text
was drafted, because the earlier wording was a soundness condition only and a parser
that accepted everything would have satisfied it vacuously.

**Exclude every input with more than one syntax error, and do it deliberately
rather than by luck.** Task 3.3 scans eagerly while the interpreter interleaves
scanning and parsing, so on a program containing both a scan error and an earlier
parse error the two disagree on the error *number*: `say )` on line 1 with
`'unclosed` on line 3 gives the oracle 37.2 line 1 and gives us 6.2 line 3. That
is a recorded, accepted deviation (see Task 3.3), and it does not arise on real
input — zero mismatches across 2,470 oracle scanner-class errors in `ootest/`,
`samples/` and `corpus-l1`. It arises on *generated* input, at any clause boundary
including a mid-line `;`, in 144 of 4,000 adversarial programs. If this step
records our answer for such a file, it enshrines the deviation as an expected
value and the gate stops being able to see it. One error per input.

`signal on syntax` **cannot** catch a syntax error in its own file. ooRexx
parses the whole file before executing anything, so the trap is never
installed. Verified: a file containing `signal on syntax name oops` and then
`x = )` prints the error to **stderr** and exits rc 219; the handler never
runs. An earlier draft of this plan specified exactly that broken recipe.

Three routes that do work, with different trade-offs:

```bash
# (a) syntax-check the bad file with rexxc and capture stderr. rexxc with no
#     output file parses WITHOUT EXECUTING, so nothing in the file runs. It
#     reports the file's own line number. The version banner goes to stdout and
#     the error to stderr, so redirect to separate the two.
build/bin/rexxc bad.rex 2>&1 1>/dev/null; echo "rc=$?"
#   ->    1 *-* x = )
#         Error 37 running .../bad.rex line 1:  Unexpected ",", ")", or "]".
#         Error 37.2:  Unmatched ")" in expression.
#         rc=219
```

```bash
# (b) run the bad file. Same error text and same rc as (a) for a PARSE error,
#     because ooRexx parses the whole file before executing anything -- but it
#     runs the file when the file is valid, and it also reports RUNTIME errors,
#     which this phase must not gate on. Use only to confirm (a).
build/bin/rexx bad.rex 2>&1; echo "rc=$?"
```

```rexx
/* (c) INTERPRET a fragment inside an installed trap. The trap fires and
   condition('o')~code is available -- but POSITION is the line of the
   INTERPRET instruction, not a position inside the fragment. */
signal on syntax name oops
interpret "x = )"
exit 0
oops: say condition('o')~code; say condition('o')~position
```

Use **(a)** as the default, including for anything positional. Use (c) only when
you want the condition object's fields, which stderr does not expose separately.

**(a) also gives the negative direction, which the other two cannot.** `rexxc`
answering rc 0 means *this file parses*, so a case can be recorded as "must not
raise a parse error" rather than only as "must raise error N". That is what
separates a parse error from a runtime one: bare `procedure`, bare `leave` and
`x = 1/0` all give `rexxc` rc 0 and fail only under `rexx` (17, 28, 42), so none
of them belongs in this task's expectations. Task 3.6 Step 4 has the measured
table.

- [ ] **Step 2: Write the failing tests from those recordings**
- [ ] **Step 3: Implement**
- [ ] **Step 4: Verify what the oracle exposes, and what we deliberately do not reproduce**

The condition object carries exactly these:
`PROPAGATED ERRORTEXT MESSAGE STACKFRAMES POSITION INSTRUCTION CODE RC
CONDITION PACKAGE TRACEBACK PROGRAM ADDITIONAL DESCRIPTION`. `POSITION` is a
**line number**, and there is no column *field* among them. ooRexx normally
locates an error by *quoting the offending token* in the message text rather
than by offset.

**But "there is no column anywhere in the oracle" is false, and earlier drafts
of this plan asserted it.** Errors 36.901 and 36.902 substitute a position:
`Left parenthesis "(" in position 5 on line 3`. It is a 1-based **byte** offset
within the offending token's own physical line — `x = "ää" || (a` reports
position 15 where the `(` is the 13th character — and its line can differ from
the main message's, which reports the clause's start line. Three review rounds
acted on the false version of this claim, so it is spelled out rather than
quietly corrected.

**What this phase gates: number and sub-number, on a plausible line. Nothing
else.** Reproducing message text and substitution values 1:1 was dropped as a
scope decision, and error 36's position is not produced at all. This is an
*observable* deviation, not an unobservable one: a trapped syntax error hands
the program `ERRORTEXT`, `MESSAGE` and `ADDITIONAL`, so a program reading those
would see a difference. That is accepted.

So this step's job is no longer a differential capture of substitutions. It is
narrower: for each error the parser can raise, confirm the number and sub-number
against `rexxc`, and confirm the generated message table has that row so the text
comes out of the table rather than being hand-written. Do not build machinery to
match spliced text, and do not track per-line byte offsets.

Runtime errors are untouched by any of this. `rexx-num`'s numbers, sub-numbers,
message text and `ADDITIONAL` values stay byte-exact, and this relaxation must
not be read as licence to loosen them.

- [ ] **Step 5: Commit**

---

