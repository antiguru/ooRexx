## Task 14: required string values

**Goal.** The protocol `provide.xml` `reqstr` specifies: `request("STRING")`, then the receiver's
`makeString`, then the NOSTRING condition if trapped, else `defaultName` (D52).

**Why this task exists at all.** It is the other mechanism the spec was rewritten for, and it is the
worse of the two: **matching exit status, empty stderr on both sides, different stdout.** Measured,
`say .k` with `::METHOD makeString CLASS returning "K says hello"` is oracle rc 0 `K says hello`
against crate rc 0 `The K class`. No refusal names an owner and no harness reddens.

**Build.** The protocol itself, plus `~string`, `~request` and `~objectName`/`~objectName=`, which are
its Rexx-level face. Measured: `.K~request("STRING")` is `The NIL object`, `.K~string` and
`.K~objectName` are both `The K class`. Then **every context `reqstr` names becomes a dispatch site**:
`SAY`; `DO`'s `exprr`/`exprf`; substituted compound-variable tails; commands and `ADDRESS`;
`ARG`/`PARSE`/`PULL`; parenthesised `CALL` targets; `DROP`/`EXPOSE`/`PROCEDURE` lists; `INTERPRET`;
`NUMERIC`'s three values; `OPTIONS`; `PUSH`/`QUEUE`; `SIGNAL VALUE`; `TRACE VALUE`; builtin arguments;
and dyadic operators with a string on the left -- plus the two different rules for method arguments
(String's arithmetic, comparison and concatenation methods fall back to `~string`; every other method
raises).

**The re-derivation, which is the migration half.** `eval.rs`'s `object_operand_tests` and
`corpus/lang/message_send_argument_object_not_a_string.rex` were derived from a coercion audit, not
from `reqstr`. Re-derive them from the section: a context the audit happened to miss is exactly what
this task exists to find, and a context the audit covered that `reqstr` does not name is a question
to answer rather than to delete.

**Verification, runnable now.** The `makeString` program above, both engines; one program per
`reqstr` context with a receiver that defines `makeString` and one that does not; the NOSTRING
condition trapped and untrapped; `apply_binary`'s object check, which this protocol makes
unavoidable.

**The two controls this task owes table C, and Task 5 assigns both here.** Delete the `makeString`
limb and record that the `reqstr` concept row reddens -- **the second of D50's two required
controls**, runnable only here. And table C's **mutation 3**, which Task 5 also owns to this task:
answer `makeString` with the *wrong string* rather than not at all, and the same row reddens at
**rc 0 with empty stderr on both sides**, on stdout alone. That second one is the whole reason this
task exists -- it is the silent-divergence shape a matching status and a matching stderr hide -- and
naming only the deletion control here would have left it owned by nobody.

**What it cannot see.** A context `reqstr` does not name is outside the row set. The section is the
denominator and the task says so.

**Done when** the `makeString` program agrees at rc 0 on both engines, every named context has a
program, and **both controls above are recorded as run** -- the deletion and table C's mutation 3.
Sitting required.

---

