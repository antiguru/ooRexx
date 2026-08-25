## Task 12: `UNKNOWN`, and the NOMETHOD condition

**Goal.** The documented method search order, complete: per-object (5b's), own class, superclasses,
**`UNKNOWN`**, NOMETHOD.

**Why this task exists at all.** It is one of the two mechanisms whose absence is the reason the spec
was rewritten. `say .k~zork(1,2)` with `::METHOD unknown CLASS` is **oracle rc 0, `unknown: ZORK`**.

**What this task starts from moved under it, and the move was Task 11's.** This paragraph said the
crate answered **rc 159, `97.1 does not understand "ZORK"`** -- a wrong error where the oracle
answers. Re-measured before dispatch, both engines: the crate is **rc 120**, `rexx-exec: the UNKNOWN
forward for message "ZORK" is not implemented (Phase 5)`. The gate is `lib.rs:828`, its test is
`dispatch.rs:2674`, and `git log -S` puts both in `f4b21eadb`, Task 11's own implementation commit.
The reason is sound rather than scope creep: once `ExprKind::List` evaluated, the send reached dispatch
for the first time, and without an `UNKNOWN` step the crate would have answered 97.1 where the oracle
answers -- a wrong answer newly reachable. Task 11 refused instead. **So this task converts a loud
refusal into the answer, and it must remove that gate rather than a wrong error.** The same holds for
the no-argument spelling: `say .k~zork` is oracle rc 0 `unknown: ZORK` against the same rc 120 here.
`corpus/lang/message_send_unknown_method.rex` is unaffected and still agrees, 97.1 at rc 159 on both
sides, because `'abc'` answers no `UNKNOWN`.

**Build.** The `UNKNOWN` step in front of the miss arm; the receiver's own behaviour consulted after
the miss (D27's amendment); the method's two arguments -- the message name and an Array of the
original arguments; the NOMETHOD condition beneath it, trapped and untrapped.

**M2.** `corpus/lang/message_send_unknown_method.rex` pins 97.1 for a name the behaviour does not
answer. That stays correct **for a receiver with no `UNKNOWN` method**, and it becomes one branch of
two: the program gains a sibling whose receiver has one.

**Verification, runnable now** -- and the argument order is why Task 11 precedes this one. Measured:
the name arm needs only `use arg n, a` and runs at rc 0 on the oracle today; the **args-array** arm
needs `a~items`, which Task 11 delivers. Both arms, both engines, plus the untrapped NOMETHOD
transcript and a trapped one.

**Where the receiver's result goes decides the arm's exit status, and the two arms differ.** Measured
all three shapes. `say .k~zork(1,2)` with an `UNKNOWN` method whose body only `say`s is oracle
**rc 165** with `unknown: ZORK` on stdout and `Error 91.999: Message "ZORK" did not return a result.`
on stderr: `say` demands a result the method never produced. The same send with the method
**returning** `'unknown:' n` is rc 0, `unknown: ZORK`. The args-array arm therefore reads best as a
statement rather than a `say` -- `.k~zork(1,2)` alone, with the method printing `'unknown:' n 'args'
a~items`, is oracle rc 0 `unknown: ZORK args 2`. Use these program shapes as written; an earlier
draft named only the message send and the surrounding statement, which does not determine the exit
status.

**The negative control this task owes table C.** Delete the `UNKNOWN` step and record that the
`unkno` concept row reddens. **This is one of the two controls D50 requires to be recorded as run**,
and Task 5 deferred it here because the step must exist before it can be deleted.

**Done when** both arms match byte for byte on both engines, the `unkno` row reads `agree`, and its
deletion control is recorded. Sitting required.

---

