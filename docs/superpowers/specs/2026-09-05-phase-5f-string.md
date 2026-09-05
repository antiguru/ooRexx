# Phase 5f — `String`'s 112 loud instance rows get bodies

**Measured 2026-09-05 at `4ff8b9683`.** The successor `2026-09-04-phase-5e-mutablebuffer.md` named in
its own §5 — *"Not the other 571 loud rows under `5c`. `String` 112 … stay as they are"* — and in its
D80 note, *"`String`'s own 112 loud rows suggest the byte machinery may already exist; that was not
checked and checking it is the first task."*

**It has now been checked, and it does.** That is the finding this phase is built on.

---

## 1. What is wrong

`String` carries 118 documented instance rows in `corpus/method-bodies.txt`. Six answer:

```text
dispatch.rs:859-876   LENGTH  MAKEARRAY  MAKESTRING  REVERSE  SIGN  UPPER
```

Those six are exactly the six `answers` rows, one to one. **The other 112 are `loud`**, measured:

```text
$ say 'abcdef'~substr(2,3)
rexx-exec: method "SUBSTR" of class "String" is not implemented (Phase 5)     rc 120
oracle:  bcd                                                                  rc 0
```

`~substr ~pos ~left ~right ~word ~verify ~changeStr ~strip ~translate`, every operator as a message,
and the base conversions are all in that set. This is ordinary Rexx that the crate refuses.

**No new state is needed, and that is what makes this phase cheap.** `MutableBuffer` cost D80's
`native: Option<Box<BufferState>>` on `Body::Instance` because a buffer has contents and a capacity.
A `String` receiver has neither: `interp.to_text(receiver)` already yields its bytes, `text_built`
and `counted` already build the answers, and `native_reverse` (`dispatch.rs:9855`) is four lines
long for that reason. Binding, not implementing.

---

## 2. The partition, which is the phase's real content

The 112 rows are four kinds of work, not one. Measured by name against
`dispatch.rs`'s `NATIVE_METHODS` (`:247`) and `builtin.rs`'s `IMPLEMENTED` (`:144`):

| n | kind | what exists already |
|---|---|---|
| 41 | a `native_mutable_buffer_*` body already implements it | `[] append pos lastPos verify word words changeStr insert overlay translate space subChar substr subWord subWords delStr delWord contains startsWith endsWith match matchChar replaceAt lower wordIndex wordLength wordPos countStr containsWord`, and the eleven `caseless*` of those |
| 25 | an implemented BIF already computes the **value**, and none of its error surface | `abbrev abs b2x bitAnd bitOr bitXor c2d c2x center centre compare copies d2c d2x dataType format left max min right strip trunc x2b x2c x2d` |
| 33 | operator as a message | `+ - * ** / // % = == \= \== < <= << <<= <> > >= >< >> >>= \< \<< \> \>> \ & && \| \|\| ?`, abuttal and blank |
| 13 | neither: a body to write | `caselessAbbrev caselessCompare caselessCompareTo caselessEquals ceiling compareTo decodeBase64 encodeBase64 equals floor hashCode modulo round` |

**`hashCode` is `Object`'s, not `String`'s** — its evidence column says
`method "HASHCODE" of class "Object"`, so it is an inherited row appearing under `String`'s arm and
implementing it moves rows on every class that inherits it. Whoever lands it says so.

**The 41 and the 25 are not equally cheap, and measuring that is what the table above hides.** A
method and its like-named builtin agree on the answer and on nothing else. Measured on `left`, all
four runs on the oracle:

```text
'abc'~left        93.903  Missing argument in method; argument 1 is required.      rc 163
left('abc')       40.3    Not enough arguments in invocation of LEFT; minimum
                          expected is 2.                                           rc 216
'abc'~left('x')   93.923  Invalid length argument specified; found "x".            rc 163
left('abc','x')   40.12   LEFT argument 2 must be a whole number; found "x".       rc 216
```

Different error number, different message, different exit status — and the argument *numbering*
shifts, because the receiver is the builtin's first argument and is not an argument of the method at
all. **So "route the receiver into the builtin" is wrong for all 25 rows.** What is reusable is the
value computation, and only where it has already been lifted out of the builtin's argument handling:
Task 3a of the 5c follow-up extracted `substr_bytes insert_bytes overlay_bytes space_bytes
changestr_bytes translate_bytes verify_bytes case_shift_bytes` into `builtin/string.rs` and
`word_count word_range subword_range delword_bytes wordpos_bytes word_slices` into `word.rs`, over
plain `&[u8]`.

That is the real ranking inside the phase, cheapest first:

* **the 41** — the method-side argument layer *and* the core both exist, because
  `native_mutable_buffer_*` wrote them against this same 93.9xx surface. **Only 30 of them share a
  body, though.** Measured on the oracle: `'abcdef'~insert('XY',2)` answers `abXYcdef` and leaves the
  receiver `abcdef`, where `.MutableBuffer~new('abcdef')~insert('XY',2)` returns *the receiver* and
  the buffer is `abXYcdef` afterwards. So the eleven whose `MutableBuffer` native reaches
  `buffer_state_mut` — `append caselessChangeStr changeStr delStr delWord insert lower overlay
  replaceAt space translate` — share the core and the argument layer and **not** the return contract;
* **the 33** — `apply_binary` computes them and `operator_argument` is the argument layer;
* **the 25** — the core exists for some and the method-side argument layer exists for none. Each row
  is a small piece of hand work measured against the oracle;
* **the 13** — nothing exists.

The first 74 are still the argument for doing `String` before anything else in the 540. What is
missing is the receiver, and for a quarter of them, the argument layer.

---

## D-numbers

**D83 — how the 33 operator rows reach an answer.** `eval::apply_binary(op, left, right)`
(`eval.rs:1872`) already computes every one of them, and `Object` already has native operator methods
built on `operator_argument(args)` (`dispatch.rs:4666`) — so the machinery is there twice. Two shapes:
one native per operator name forwarding to `apply_binary`, or one native that carries its `Operator`
in the table row. **Not decided here, and the risk is on stderr rather than stdout.** Measured, the
two forms of the same operation differ by one traceback line and agree on everything else:

```text
$ say 'abc' + 1                    $ say 'abc'~"+"(1)
                                          *-* Compiled method "+" with scope "String".
   1 *-* say 'abc' + 1                1 *-* say 'abc'~"+"(1)
Error 41.1 ... rc 215              Error 41.1 ... identical, rc 215
```

A route through `apply_binary` that does not push a method frame drops that line, and drops it only
on a *failing* operand — so a program of successful operator sends cannot witness it. §3's programs
have to make each of the 33 raise as well as succeed.

**The crate already emits that frame for a native method, on both engines.** Measured, and the three
outputs are byte-identical:

```text
b = .MutableBuffer~new('abc'); say b~substr
       *-* Compiled method "SUBSTR" with scope "MutableBuffer".
Error 93.903 ... rc 163      REXX_ENGINE=ir, REXX_ENGINE=tree-walker, oracle
```

So the frame comes from the send machinery and not from any body, and the risk is narrowed to an
operator native that leaves that machinery. That is what makes D83 a shape question rather than a
feature question.

**D84 — what the gate is, because the verdict column cannot see a stub.** `Arity::Fixed(n)` is a
**maximum**: it refuses more than `n` with 93.902 and *pads a shorter list with nulls*
(`dispatch.rs:219-225`). So the required-argument check is each body's own code, and measured, the
oracle's answer to a no-argument send is a raise:

```text
$ say 'abcdef'~substr
Error 93.903:  Missing argument in method; argument 1 is required.       rc 163
```

`method-bodies.txt` sends **no arguments**. A row therefore moves `loud` → `answers` as soon as the
crate raises 93.903 correctly — **with no body at all**. Measured here: of `MutableBuffer`'s 51
instance rows, **32 are `answers rc 163`**, 13 are `rc 0` and 6 are `rc 168`.

**This was already found, and the criterion was already retired.** `2026-09-04-phase-5c-followup.md`'s
gate section says so, and its plan review went further than measuring — it *built* the stub: 51 rows
bound to seven functions, about 90 lines, the constructor still discarding its argument, all 51 rows
agreeing on all three descriptors on both engines. Its words are the ruling this phase inherits:
*"A criterion a stub satisfies is not a criterion, and this is the third time the same idea has failed
here"* — `hasMethod` readbacks, then "the row answers", then "the row answers a zero-argument send".

An earlier draft of this spec proposed the verdict column as an instrument that covers the
missing-argument path *"which nothing else in this phase provides"*. **That is wrong and the same
plan says why**: the raiser is one shared implementation, already byte-identical to the oracle, so 87
rows exercising it exercise one code path 87 times. It is not coverage. It stays in §3 as a **drift**
check and carries no positive claim.

**D85 — whether the 33 operator rows are in scope.** 5e's §5 records them as a deliberate exclusion
(*"Task 3 left `String`'s operator-message rows"*), so including them reopens a recorded decision, and
D83 says they are the one part with an unmeasured risk. Splitting them into their own phase keeps the
other 78 clean. **Decided 2026-09-05 by Moritz: in scope.** All 112, operator rows included, and 5e's
Task 3 exclusion is reopened deliberately. The gate below is stated per-row so a partial landing is
visible.

**D86 — the phase's id and the `method-owner` column.** 5e faced this as D81 and the tree answers it:
`CLOSED_PHASES` is `["5a","5b","5c","5d"]` (`gate_tables/mod.rs:344`) and `MutableBuffer`'s row still
reads `5c` (`class-set.txt:92`), so 5e re-owned nothing and ran as a 5c follow-up. `String`'s row reads
`5c` too (`class-set.txt:57`). **Decided 2026-09-05 by Moritz: do the same, change no committed
table.** `class-set.txt`'s ninth column keeps saying `5c` for `String`, `CLOSED_PHASES` gains nothing,
and `5f` is this document's name rather than a value any table carries.

---

## 3. The gate

**The gate is the corpus programs. The verdict column is a drift check beside them, not a criterion.**

1. **New `corpus/lang/string_*.rex` differential programs calling every one of the 112 with real
   arguments, byte-identical to the oracle on all three descriptors, both engines.** This is the
   gate. `corpus/lang/mutablebuffer_{readers,mutators,caseless,conversion,state,instance}.rex` are
   the shape to copy — one program per family, filed by the commit that writes it, never at the end.
   Every row in §2's table appears in one of them, including each of the 41 reused `MutableBuffer`
   bodies: *reused* is a claim about the code, and `String`'s behaviour is what the oracle is asked
   about.

2. **`corpus/method-bodies.txt`: no `String` row moves to `diverge`.** Drift only. Per D84 the
   direction `loud` → `answers` is ungated progress and a stub satisfies it, so it is reported and
   not required.

**Shown to fail**: replace one landed body with a stub that raises 93.903 and nothing else.
Instrument 1 must go red. If it stays green the program set does not cover that row, and the row is
unwitnessed however the table reads — which is the only failure mode that matters here, because the
table will read `answers` either way.

---

## 4. What this phase does not do

* **Not `RexxInfo`'s 28 rows or `WeakReference`'s 1.** They need no new state either and were the
  obvious ride-along, but they would land in the same `method-bodies.txt` diff as `String`'s 112 and
  confound it. Their own phase, or a task that lands after §3's reading is taken.
* **Not the 13 collection classes' 257 rows**, nor `Package` 32 / `Message` 17 / `Method` 14 /
  `RexxContext` 14 / `Class` 11. Those are a different kind of work — receivers that hold state — and
  `String` is chosen first precisely because it holds none.
* **Not `Stream` 24, `StackFrame` 10, `Pointer` 5.** Owed by phase 7, deferred, and `unreachable`
  respectively; 39 of the 579 loud instance rows are not `5c`'s.
* **Not a change to the method-body table's drift rule.** §3 adds a second instrument beside it.

## 5. Risks

* **The 41 reused bodies are the largest block and the least examined.** They were written against a
  `MutableBuffer` receiver, and a divergence in one of them reads as a `String` defect. The corpus
  programs are what separate the two; a row that only ever ran under `MutableBuffer` has been tested
  once, not twice.
* **`hashCode` moves rows on classes this phase is not about**, per §2.
* **112 hand-written missing-argument checks is 112 chances to write the wrong error number** — and
  §3 says the verdict column cannot be trusted to see them, because the shared raiser makes every
  such row agree for free. The corpus programs have to send a short argument list as well as a good
  one, or nothing witnesses this at all.
* **D83's extra traceback line is measured on the oracle and unmeasured on this crate.** Whether
  `apply_binary` reached through a send already pushes that frame is the first thing to run, before
  any of the 33 rows are written — and it is visible only on a raising operand.
