STATUS: DONE

# Task 6 fix re-review: the exhaustive resolution pass

Reviewing `3cf3f03c`, "Make Plan::note/build exhaustive over
ExprKind/InstructionKind, no catch-all", against the defect I found in the
original review. Clean export of the commit; the working tree carries live
Task 8 work in `eval.rs` and was not used.

## Verdicts

**Spec compliance: PASS.** The defect is closed. The pass now visits every
instruction and expression form, and D16's "built by one upfront pass" is true
of stem and compound bodies for the first time.

**Code quality: PASS.** The wider registration is correct rather than merely
wider, the helper extraction changed no existing behaviour, and the reasoning
for going past the minimum is sound.

**0 Critical, 0 Important, 2 Minor.**

---

## The acceptance bar: met, and re-run rather than accepted

Neutering `Plan::build` to `Plan::default()` in my own export:

```
test plan::tests::build_registers_every_name_a_stem_or_compound_touches ... FAILED
test plan::tests::a_fragments_plan_resolves_against_the_enclosing_frame ... FAILED
test result: FAILED. 36 passed; 2 failed
```

Exactly the reported numbers, and the right two tests. Before the fix the same
mutation left 19 of 20 passing with only the fragment test failing; the pass now
has a test that fails when it is deleted, which is what it lacked.

**The new test is a real content test, not a non-emptiness check.** It asserts
specific names per program across six cases, including `say a.b` -> `A.`, `B`;
`drop a.b.c` -> `A.`, `B`, `C`; and `do i = 1 to 5` -> `I` — each of which was
dropped by a *different* `_ => {}` arm before. It also keeps `say v` as an
explicit control, so a fix that broke what already worked would be
distinguishable from one that left something unfixed.

## The wider registration is correct, not merely wider

This was the thing most likely to be a new defect in a fix's clothing, so I
measured what `build` actually produces rather than reading the arms:

```
"say a.b"        -> ["A.", "B"]          not ["A.B"]
"a.1 = 'x'"      -> ["A."]               a digit tail is a constant (D15a)
"q. = 1"         -> ["Q."]
"drop a.b.c"     -> ["A.", "B", "C"]
"say a.b.c"      -> ["A.", "B", "C"]
"if x then say y"-> ["X", "Y"]
"select case s"  -> ["S"]
"numeric digits 5" -> []
"trace value tv" -> ["TV"]
"exit rc"        -> ["RC"]
"leave lbl"      -> []
"say .nil"       -> []
```

The two rows that matter most are the empty ones. **`leave lbl` registers
nothing** — a `LEAVE` label is not a variable, and registering it would have
been exactly the "name that is not a variable" defect. **`say .nil` registers
nothing** — an environment symbol is not a slot. Both are the cases a
mechanically-wider pass gets wrong, and both are right.

`a.1 = 'x'` giving only `A.` is the other good sign: a bare digit tail is a
constant piece, so it needs no slot, and the pass distinguishes that from a
letter-led piece.

## The `Compound` claim: confirmed by reading the resolution path

The registration for compounds depends entirely on a compound's own id never
being looked up, and that holds. `eval.rs`:

```rust
ExprKind::Variable(id) | ExprKind::Stem(id) => {
    let (value, _novalue) = self.read(code, *id);   // -> code.slots.get(&id), i.e. by_symbol
}
ExprKind::Compound(id) => {
    let (stem_name, _tails) = compound_parts(code.symbols.name(*id));
    let key = self.tail_key(code, *id);
    Ok(self.stem_get(stem_name.as_bytes(), &key))   // resolves by NAME
}
```

The `Compound` arm decomposes the interned dotted spelling and resolves through
`stem_get`/`tail_key` by name; it never reaches `by_symbol`. So registering the
decomposed names and *not* the compound's own id is right, and registering the
id would have been the error.

**The name/id split is coherent, measured rather than inferred**, by printing
`by_symbol`'s size alongside:

```
"say a.b"          names ["A.", "B"]   by_symbol 0
"q. = 1"           names ["Q."]        by_symbol 1
"say v"            names ["V"]         by_symbol 1
"drop a.b.c"       names [3 names]     by_symbol 0
"if x then say y"  names ["X", "Y"]    by_symbol 2
```

An id is registered exactly where evaluation looks one up: `ExprKind::Stem`
goes through `read(code, id)` so its id is registered; a compound's is not, and
a tail piece has no id at all, since `compound_parts` hands back borrowed text
rather than a token. Nothing over- or under-registers on that axis.

## The helpers changed no existing arm

Checked against the three `build` arms and three `note` arms that existed
before:

```
"v = w"      -> ["V", "W"]        slots [(0,"V"), (1,"W")]
"interpret x"-> ["X"]             slots [(0,"X")]
"say a b c"  -> ["A", "B", "C"]   slots [(0,"A"), (1,"B"), (2,"C")]
"a = b + c"  -> ["A", "B", "C"]   slots [(0,"A"), (1,"B"), (2,"C")]
```

Assignment still walks target then value, `Say` and `Interpret` still walk their
expressions, and `Binary`/`Blank` still recurse. **Slot numbers are still
assigned by first mention in source order**, which matters because it is the
property `bind` guaranteed and `slot_for` now sits underneath — a helper that
renumbered would be invisible in a name-set assertion but would move every
slot.

## m1 (Minor). The new test asserts presence, never absence

`build_registers_every_name_a_stem_or_compound_touches` checks each expected
name is in the plan. It never checks that nothing *else* is. So the exact
failure mode this fix was most at risk of — registering a name that is not a
variable — would pass it.

There is no over-registration today; I verified that by measuring the exact
contents, including the `leave lbl` and `say .nil` cases above. The gap is in
the test, not the code, and it is the same shape as the original defect: a test
that cannot distinguish "correct" from "wider than correct".

Cheapest fix is an `assert_eq!` on the sorted key set for two or three of the
existing cases rather than `contains_key` per name. `leave lbl` and `say .nil`
would be the two worth adding, since an empty expected set cannot pass
vacuously.

## m2 (Minor). Nine helpers, not five

The commit adds `note_instruction`, `note_loop`, `note_parse`, `note_call`,
`note_variable_ref`, `note_compound_name`, `note_opt`, `note_args` and
`slot_for`. Cosmetic, noted only because the count is quoted as five and
someone re-reading the diff against that number will wonder what they missed.

---

## On going past the minimum

The implementer registered every name an instruction's fields could name rather
than only the kinds 4a executes, arguing that matching current kinds and
writing "Task N will handle this" is the self-promising-comment pattern that
caused the original defect. I agree, and I would go further: those comments
were *mine*, written in the spike as a handover and carried verbatim into Task 6
where they became a task promising itself. The exhaustive match removes the
place such a comment can live, which is a better fix than the comment being
more carefully worded.

The cost is that names are registered for instructions 4a cannot execute — a
`CALL`'s arguments, a `PARSE`'s targets. That is harmless: an unused slot costs
one `None` in the frame, and the alternative is another `_ => {}` waiting to be
the next defect.

## Method

Clean export of `3cf3f03c` via `git archive`. The mutation was re-run by me
rather than taken from the report. Plan contents and `by_symbol` sizes were
measured with a throwaway probe in the export, not inferred from the arms. The
export was discarded; nothing in the repository was modified.
