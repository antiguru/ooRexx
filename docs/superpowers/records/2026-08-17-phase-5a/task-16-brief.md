## Task 16: `REPLY` and `GUARD` inside a method

**Goal.** D55: a `REPLY` or `GUARD` inside a Phase 5 method **runs**, rather than being refused.

**Measured, both programs.** `guard on` inside a class method is oracle rc 0 against our rc 120. And
`reply "replied"` followed by `say "after"` and `return "returned"` is **oracle exit status 0**, stdout
`replied` then `after`, stderr a `98.936 RETURN cannot return a value after a REPLY` traceback --
against our rc 120. **Note the transcript shape**: exit status 0 *with* a traceback on stderr, because
the raise happens after the main program has finished. A gate case must reproduce that pair, not just
the status.

**Build.** In a single-threaded Phase 5, `GUARD ON` on an uncontended object is a no-op and `REPLY`
hands its value to the sender and lets the body carry on; a `RETURN` carrying a value afterwards
raises `98.936`, which is a run-time legality check in D32's shape. Bare `GUARD` keeps its syntactic
check. **Phase 6 replaces the scheduling, not the legality**, and the task says which of its code is
which so Phase 6 knows what it may rewrite.

**Verification, runnable now.** Both programs above, both engines, compared raw. Measured beforehand:
a `guard on` body that is never sent to is rc 0 on both sides already, so the refusal is at invocation
and the bootstrap's `guard`/`reply` bodies (`CoreClasses.orx:1554`, `:1663`) are not blocked by this
gap -- which is why this task sits here rather than before Task 23.

`phase-4-exclusions.txt:3304`'s `GUARD` limb moves out in this commit.

**Done when** both transcripts match on both engines including the exit-status-0-with-traceback shape,
and a control returning a value after `REPLY` without raising reddens it. Sitting required.

---

