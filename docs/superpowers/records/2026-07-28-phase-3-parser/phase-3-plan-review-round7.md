# Phase 3 parser plan — seventh pass, reviewing the round-6 fix and the interning design

Targets: `b61d586a` ("Close the round-6 conditions on the Phase 3 plan") and `a9bbb2a1`
("Intern symbols in Task 3.3, and rule out hash-consing the AST"), both against
`docs/superpowers/plans/2026-07-28-phase-3-parser.md`.
Specification for `b61d586a`: `.superpowers/sdd/phase-3-plan-review-round6.md`, findings N1–N8.
`a9bbb2a1` had no specification and no prior review.
Reviewed: 2026-07-28. Oracle: `build/bin/rexx`, `build/bin/rexxc`, and the C++ tree.
Every finding is tagged with how it was established. Nothing below is reasoned from memory.

---

## Verdict

**GO** for dispatching Task 3.2 and beyond.

Task 3.2 is `ProgramSource` and touches none of the interning surface, so nothing here blocks
it.
Five **Important** findings must land before Task 3.3, which is where the interning design is
built, and one of the five is a consequence of the parse-error scope decision taken during this
review.
No Critical.

Round 6's two GO conditions are both closed, and closed well: N1's fix inverts the false claim
instead of hedging it, and N2's fix corrects not just the list of three but the *reason*, which
round 6 also found wrong for two of the three keywords it named.
Three of the four follow-up edits landed correctly and the fourth landed in the wrong task.
No third place states any of the four claims; I checked each inside its owning task rather than
document-wide.

The interning design is **sound in principle and wrong in one specific place**.
Sound: the C++ scanner already interns symbol strings (`commonString`, `Scanner.cpp:1512`), the
upcasing is byte-exact against the oracle's own table, and an uninitialised symbol's *value* is
the upcased spelling, so the interned name is not merely an identity but the value the executor
needs.
Wrong: `Program::labels: BTreeMap<SymbolId, usize>` cannot represent a **literal label**, which
is legal, reachable, and case-sensitive. That is finding I1 and it is four probes deep.

The recurrence recurs twice, both inside the task the commit edited.
`a9bbb2a1` gave `TokenKind::Symbol` a payload and left Task 3.3's own five scanner tests
constructing it as a unit variant, so they no longer compile.
And it introduced `Keywords` into `ParseCtx` without ever saying what it is.

---

## 1. Round 6's findings, one verdict each

| # | finding | verdict |
|---|---|---|
| **N1** (Important, GO condition) | Task 3.4 still said a clause span is derivable from a token sub-range | **fixed.** Line 784 now reads "`tokens` is a range and nothing in `Clause` is shared or interned, so a sub-range is always constructible", and a new paragraph at 787 states the opposite of the old claim outright, with the `THEN` clause's `6..8` tokens versus its token-6 end as the worked counter-example. The inversion is correct and the example matches Task 3.6's own token numbering. Round 6's suggested addendum — that a clause returned by `split_before` carries no `Eoc` — did **not** land; see M4 below. |
| **N2** (Important, GO condition) | Step 3b constrained three keywords; Task 3.6 Step 4 needed five and cited Step 3b for all five | **fixed, and better than asked.** Step 3b (203–219) now names all five, gives a per-keyword measurement, and replaces the justification round 6 showed was wrong: it no longer claims criterion 6 gates on all of them. VERIFIED against the corpus: `rust/corpus/lang/trace_output.rex` is six lines and contains `if y > 5 then say "big"` and no `else`, `otherwise`, `when` or `end`, so `THEN` is in criterion 6's scope and `END` reaches it only through probe A, exactly as the new text says. The fallback reason also checks out: `samples/` carries 718 `else`, 431 `otherwise` and 883 `when` word matches. Task 3.6 Step 4 (1212) and the Notes bullet (1951) agree, and both name the same five. |
| **N3** (Minor) | marker count wrong, and the plan's own warning missed two shapes | **partially fixed.** Both enumerations are corrected and there is no third: Task 3.9 body (1595–1603) and criterion 7 (1853–1863) are the only two, and both now cite the table. I re-read `RexxActivation.cpp:3567–3587` independently: nineteen entries, and of the eighteen non-`*-*` exactly sixteen have the `>X>` shape, `+++` and `<I<` do not. The plan's "Sixteen of the eighteen" is right. **What survives is the `243` two lines below** — see M1. |
| **N4** (Minor) | a continued clause's traced text is not a contiguous byte range | **partially fixed.** The Notes bullet at 1937 is new, correct and independently VERIFIED: `x = 1 ,` / newline / `    + (a` traces as `x = 1 ,    + (a`, newline gone, continuation blanks kept. The "`TRACE`'s `*-*` line prints `clause_span`" claim above it is now qualified with "**for an unbroken clause**". But round 6 asked for one sentence *in Task 3.9*, and Task 3.9 (1554–1702) contains no occurrence of "continu" at all. A reader of Task 3.9 alone still sees the unqualified claim. |
| **N5** (Minor) | probe A's description omitted `end` | **fixed.** Line 1653 now lists four clauses including `end`, says it is "easy to leave off the list", and adds that four clauses produce more than four `*-*` lines. |
| **N6** (Minor) | `split_before`'s doc comment dropped an invariant it still relies on | **not fixed.** The body still does `..cur.span.end` for the remainder and the doc comment still does not say why. |
| **N7** (Minor) | "tiles" used in two senses | **not fixed.** The Notes bullet at 1945 still opens "**Clause spans do not tile the source**" while criterion 1 at 1745 uses "tiles" for containment-plus-ordering. The bullet's *second* sentence says "partition"; the heading that collides was not changed. |
| **N8** (Minor, discretionary) | the `assert!` permits the overlap C1 was about | **not applied.** Line 1002 is unchanged. Round 6 called this a strengthening rather than a defect, so this is a legitimate declination. |

**2 of 2 GO conditions closed. 3 of 6 Minors fixed, 2 partially, 1 declined.
No round-6 fix introduced a defect of its own.**

### The four self-sweep edits

All four were checked in place, and then each claim was re-searched inside its owning task only.

* **Task 3.6 Step 4's three-versus-five wording** (1212–1218) — landed, and now cross-references
  Step 3b as naming "the same five and gives the measurement for each", which is true.
* **The Notes bullet** (1951–1954) — landed, with a genuinely useful addition: "only the first
  three end a clause *mid-line*, which is Task 3.4's rule 4 and a different property". That
  distinction is correct and is the thing a reader would otherwise conflate.
* **The second marker enumeration** (criterion 7, 1849–1863) — landed. There is no third
  enumeration. `grep` for every number word and for each marker string returns hits in two
  places only.
* **Probe A's missing `end`** (1653) — landed. There is no third statement of probe A's clause
  list; the only other mentions of `nop;` are Task 3.4's rule 2 and its test, which are about
  the semicolon and not about probe A.

---

## 2. Findings on the interning design

### Important

#### I1 — `Program::labels: BTreeMap<SymbolId, usize>` cannot represent a literal label

Task 3.7b, line 1350:

> ```rust
> pub labels: BTreeMap<SymbolId, usize>,
> ```

and Task 3.3, line 417:

> /// A symbol's identity: the upcased spelling, interned.

A label may be a **literal**, and a literal label's name is stored **verbatim**, not upcased.
`SymbolId` is by construction the upcased spelling, so this type cannot hold the key and
`intern` destroys it.

VERIFIED, four probes, each read rather than skimmed:

| program | result |
|---|---|
| `'MiXeD': nop` under `rexxc` | rc 0 — a literal label is legal |
| `signal value 'MiXeD'` + label `'MiXeD':` | reaches it |
| `signal value 'MIXED'` + label `'MiXeD':` | **error 16.1, `Label "MIXED" not found`** |
| `signal MiXeD` + label `'MiXeD':` | **error 16.1, `Label "MIXED" not found`** |
| `signal value 'MIXED'` + label `mIxEd:` (symbol) | reaches it |
| `signal value 'mIxEd'` + label `mIxEd:` (symbol) | error 16.1 |

The model is exact and it is in the C++: `InstructionParser.cpp:153` accepts
`first->isSymbolOrLiteral()` before a colon, and `labelNew` keys the table by
`nameToken->value()` (`InstructionParser.cpp:2795–2799`).
A symbol token's value is upcased by `Scanner.cpp:1492–1511`; a literal token's is not.
`SIGNAL` static and `SIGNAL VALUE` both match that key by exact string equality.

So the plan's design is wrong in **both** directions: it would make
`signal value 'MIXED'` succeed where the oracle raises 16.1, and
`signal value 'MiXeD'` fail where the oracle succeeds.
`SymbolTable::get`'s doc comment, which exists specifically for this lookup, cannot find a
mixed-case literal label at all.

Nothing in the Phase 3 gate catches this. Criterion 5 is parse-time and 16.1 is runtime;
criteria 1–4 only require the file to parse. It lands directly in the interface Phase 4
consumes.

**Fix:** key labels by the token's *value string*, not by `SymbolId` — either
`BTreeMap<Box<str>, usize>`, or an explicit two-case key that records which token kind produced
it. Whichever is chosen, Task 3.6 Step 3's `SIGNAL`-and-labels bullet must say that a literal
label keeps its case and a symbol label does not, because that is where the label table is
built.

#### I2 — `Keywords` is named in three places and specified in none

Task 3.3, line 361:

> /// The 35 keyword spellings and the sub-keywords, interned by `scan`
> /// before it reads any source, so their ids are fixed and a keyword test
> /// is an integer comparison.

That is the whole specification. There is no struct, no field list, no constructor, and no
statement of membership.
`SymbolId`, `SymbolTable`, `TokenCursor`, `ClauseCursor` and `Clause` all get full code in this
plan; `Keywords` gets a borrow and a sentence.

The membership question is not cosmetic, because this plan counts five different tables:
35 keywords, 50 `subKeywords`, 12 `conditionKeywords`, 10 `parseOptions` (line 41), plus
9 `directives` and 40 `subDirectives` for Task 3.7 (line 1244).
"The 35 keyword spellings and the sub-keywords" names at most two of the six.
Task 3.3's `symbols` doc comment says "Tasks 3.6 **and 3.7** need it", so Task 3.7's tables are
in scope — yet Task 3.7 line 1291 still describes resolution as "the token after `::` looks up
in `directives[]`, everything after it looks up in `subDirectives[]`", in C++ table terms, with
no mention of `keywords` or of ids.
Task 3.6 line 1066 meanwhile says flatly "**Do not build a sorted string table in Rust**".
Those two instructions are not reconcilable by an implementer without inventing the type.

**Fix:** give `Keywords` a definition in Task 3.3 with the same weight `SymbolTable` gets —
which tables it covers, what the accessor looks like, and who calls it — and add one sentence
to Task 3.7 saying whether directive resolution goes through it.

#### I3 — Task 3.3's own five scanner tests no longer compile

Task 3.3, line 332, changed by `a9bbb2a1`:

> `TokenKind::Symbol` carries a `SymbolId` rather than text

Step 2's tests, unchanged, at lines 594, 604, 608, 619, 623, 635 and 640:

> ```rust
> assert_eq!(kinds(&toks), [TokenKind::Symbol, TokenKind::Blank, TokenKind::Symbol, TokenKind::Eoc]);
> ```

VERIFIED by compiling the shape: with a payload, `TokenKind::Symbol` in expression position is
`fn(SymbolId) -> TokenKind`, and `rustc` rejects the array with
`expected enum constructor, found TokenKind` (E0308).
Five of the six Step 2 tests are affected, and they are the tests that carry the
significant-blank rule — the thing this task calls "the single most important thing in this
task".

This is the plan's recurring shape, inside the task the commit edited: the interface changed in
the Interfaces block and the task's own code four hundred lines below did not.

**Fix:** either add a tag accessor (`Token::tag() -> TokenTag`) and write the tests against it,
or write the payload in every test literal. The former is smaller and is what the tests actually
want, since none of them asserts on a symbol's identity.

#### I4 — `intern`'s fast path allocates on both branches, so its stated purpose is not achieved

Task 3.3, lines 435–442:

> ```rust
> // Upcase only when the text is not already upper, so the common case
> // in machine-generated and conventionally-written Rexx allocates
> // nothing on lookup.
> let upper: Box<str> = if text.bytes().any(|b| b.is_ascii_lowercase()) {
>     text.to_ascii_uppercase().into()
> } else {
>     text.into()
> };
> ```

`<Box<str> as From<&str>>::from` copies into a fresh allocation. So does
`String::into_boxed_str` on the other branch.
Every `intern` call allocates exactly once regardless of the branch taken, including on a hit.
The comment claims the opposite, and the branch buys nothing at all: `to_ascii_uppercase` is
already identity on text with no ASCII lowercase, so the condition only decides *which*
allocation happens.

That matters more than a stray comment, because "replaces about ten thousand short-string
allocations with that many hash probes plus a few hundred `Box<str>`" (line 487) is the entire
performance argument for the design, and the code as written delivers ten thousand
allocations plus ten thousand hash probes.

**Fix:** probe with a borrowed key and allocate only on insert.
`Cow<'_, str>` does it: `Cow::Borrowed(text)` when there is no ASCII lowercase, else
`Cow::Owned(text.to_ascii_uppercase())`, then `self.by_name.get(key.as_ref())` — which
typechecks because `Box<str>: Borrow<str>`.
Then the branch means what the comment says.
Worth stating in the same breath that the entry is stored twice, once in `names` and once as the
`by_name` key, so a new symbol costs two allocations; that is a fair trade for `name()` being
an index, but it should be a recorded choice rather than an accident.

#### I5 — Global Constraint 3 now carries the whole error contract, and the reason it gives is incomplete

Raised by the parse-error scope decision taken during this review, not by either commit.
Line 15:

> Error numbers **and sub-numbers** are contract. Programs trap on them.

Under the new scope that sentence is the *only* thing holding the error contract, because
byte-exact message text and error 36's position substitution are both being dropped and
criterion 5 is being relaxed to "correct number and sub-number, on a plausible line".
It needs to say so explicitly, and its justification needs correcting, because as written a
reader can draw the opposite conclusion from it in two ways.

First, the constraint does not say that numbers and sub-numbers are the *only* error property
that is contract. A reader who reaches criterion 5's current "substitution values match the
oracle" has no basis in line 15 for treating that as un-gated.

Second, and more important, "Programs trap on them" reads as *the number is what a program can
see*, and that is false. A trapped syntax error exposes the rendered text and the substitution
values to the running program as data. VERIFIED — `signal on syntax` around
`interpret "x = (a"`:

```
CODE=[36.901]   RC=[36]   POSITION=[2]
ERRORTEXT=[Unmatched "(" or "[" in expression.]
MESSAGE=[Left parenthesis "(" in position 5 on line 2 requires a corresponding right parenthesis ")".]
ADDITIONAL items= 2   ADD 1 =[5]   ADD 2 =[2]
```

So dropping the message text and error 36's position is an **observable** deviation, not an
unobservable one. That is a perfectly defensible call — no gated corpus program traps a syntax
error and reads `~message` — but it must be recorded as a deliberate deviation with its
observable surface named, otherwise the next reviewer rediscovers `~additional` and reopens it.

**Fix, in the constraint itself:** numbers and sub-numbers are contract; `ERRORTEXT`, `MESSAGE`
and `ADDITIONAL` are observable through `condition('o')` on a trapped syntax error and are
deliberately **not** reproduced byte-for-byte in this phase.

**And wherever the plan states the rationale, state it the new way.** The flat claim is wrong
and should not survive in any form. Task 3.8 Step 4 line 1538 ("Nothing on stderr carries a
column either"), line 1546 ("nothing in this phase can check it against the oracle") and
criterion 5 line 1819 ("Not column — the oracle exposes none") are all false as written.
VERIFIED, three shapes:

```
        q = (1          -> Error 36.901: Left parenthesis "(" in position 13 on line 1 ...
y = 1; z = y[ 3         -> Error 36.902: Square bracket "[" in position 13 on line 1 ...
x = "ää" || (a          -> Error 36.901: ... in position 15 on line 1 ...
```

`rexxmsg.xml:2950` has `position` as substitution 1 and `line_number` as substitution 2.
The third probe pins the unit: the `(` is the 13th character and the 15th byte, and the oracle
says 15. The correct replacement wording is "the oracle does substitute a byte position in
error 36's messages, and this phase deliberately does not reproduce it".

##### Withdrawn: the per-token position quadruple

I had a finding here on whether a token must carry
`(start_line, start_offset, end_line, end_offset)`, as Task 3.1's report states, versus deriving
line and in-line offset from `ProgramSource`'s line index.
**Withdrawn by the scope decision, not because it was wrong.** With error 36's position dropped,
nothing in this phase consumes a byte offset within a physical line, so the question no longer
has a consumer to be decided against.

Two facts from it are worth keeping on the record anyway, because they bear on what the plan
should not now assert. Task 3.2 already ships `ProgramSource::position(&self, byte) -> (usize,
usize)` (line 259) with a test asserting `position(4) == (1, 5)`, so the derivation exists
whether or not anything gates on it; and the two line numbers in one error genuinely differ — a
comma continuation makes 36.901 say "line 3" for a clause whose trace header says line 2,
VERIFIED. Neither obliges the plan to do anything under the new scope.

### Minor

**M1 — the `243` value-marker count survives two lines under the new warning against counting
by shape.** Line 1605 still reads "1,338 lines with **135** lines carrying `*-*` and **243**
carrying a value marker". Measured with the nineteen literal prefixes from the table: `*-*`
appears on **135** lines (138 occurrences), and **275** lines carry at least one of the
eighteen others. 243 matches nothing I can reproduce; the regex `>.>` gives 246 lines and
`>[A-Za-z=]>` gives 195. The adjacent figures also mix units without saying so: 135 is lines
while "`>L>` leads at 58 occurrences and `>>>` follows at 49" is occurrences — `>L>` is on 57
lines and 58 occurrences. Nothing gates on any of these. Either restate as "275 of 1,338 lines
carry a non-`*-*` marker, counted from the table" or drop the number, since the paragraph's
point does not need it.

**M2 — `SymbolTable::get`'s documented contract stops being true once keywords are
pre-interned.** Line 459: "`None` means no symbol with that exact spelling exists." With the 35
keyword spellings interned before any source is read, `get("SAY")` returns `Some` for a program
containing no `SAY` symbol at all. Harmless for the label lookup, which then misses in
`labels`, but the contract as written is wrong and it is the contract a Phase 4 implementer will
rely on.

**M3 — the plan claims a saving without netting off the pre-interning cost.** Pre-interning the
keyword spellings happens per `SymbolTable`, and `parse_interpret` builds a fresh one per call
(line 1447). So an `INTERPRET` inside a loop pays the whole keyword set per iteration, and
`Program::symbols` always carries names that never appear in the source. Negligible against
criterion 8, which parses two files once, but the "what this buys" paragraph should say what it
costs.

**M4 — three round-6 addenda did not land.** N1's "a clause returned by `split_before` carries
no `Eoc`" — Task 3.6's only occurrence of `Eoc` is in the worked example's token list, and the
advice is still absent even though it follows from `Clause::tokens` excluding the terminator on
both sides of the split. N6's restored invariant. N7's "partition" in the bullet heading.

**M5 — nothing records why `to_ascii_uppercase` is the right upcasing.** It *is* right, and
the reason is stronger than the plan's: `LanguageParser::characterTable` (`Scanner.cpp:60`)
maps only `!`, `.`, `0`–`9`, `?`, `A`–`Z`, `_` and `a`–`z`, and **every byte from 0x80 to 0xFF
is zero**, so a non-ASCII byte cannot be part of a symbol. VERIFIED: `bäc = 2` gives
`Error 13.1: Incorrect character in program "ä" ('C3A4'X)`, at parse time. So
`to_ascii_uppercase` is byte-identical to `translateChar` over every input `intern` can
receive, including the one surprising case — an exponent sign is inside the symbol, and
`translateChar` returns 0 for it and keeps the original, which is what `to_ascii_uppercase`
does. VERIFIED behaviourally: `say 1e+5` prints `1E+5`, upcased and not evaluated. Cite the
table, because Task 3.3 Step 4's "a DBCS or UTF-8 byte sequence must survive a round trip"
reads as licence to admit non-ASCII into symbols, which would silently under-upcase.

**M6 — symbol syntax: every claim in the commit checks out, and the plan should say which
shapes reach `intern`.** All VERIFIED against `build/bin/rexx`:

* A compound name is **one** symbol token. Decisive rather than circumstantial:
  `sTem.aBaB….aBaB…` of 283 characters raises `Error 30.1: Name or symbol exceeds 250
  characters: "STEM.ABAB….ABAB…"`, so the length check sees the whole dotted name, and the
  substitution is the upcased form.
* A symbol may begin with a digit (`say 1` prints `1`), with a period (`say .5` prints `.5`),
  may be `.` alone (prints `.`), and may end in one (`stem.` prints its stem default).
* An uninitialised symbol's value is its own **upcased** name: `abc` prints `ABC`, `.Abc` prints
  `.ABC`. This is the fact that makes the design better than the plan argues — the interned
  name is not only the identity, it is the value, so nothing has to re-derive it from the span.
* `!`, `?` and `_` are symbol characters; `+`/`-` occur inside a symbol only in an exponent.

**M7 — the interning commit half-justifies `ParseCtx::symbols` on parse-error substitutions,
which the new scope drops.** This is the answer to "does anything in my two commits depend on
byte-exact parse-error matching". One place does, and only in its reasoning:

> /// Tasks 3.6 and 3.7 need it to compare a clause's first symbol against the pre-interned
> /// keyword ids, and **Task 3.8 needs it to put a symbol's name in an error message's
> /// substitutions**. (line 356)

and the same half-reason in `Program`'s field comment at line 1351, "Phase 4 resolves names for
error substitutions and `SIGNAL`'s label lookup".
Nothing has to be removed: the table is still needed for the keyword comparison, for
`SIGNAL`'s label lookup, and for Phase 4 generally. Only the Task 3.8 clause is now moot, and
it should come out so the retained table does not look load-bearing for a thing the phase no
longer does.

Nothing else in either commit depends on error text. I checked each error claim the two commits
introduce. `SymbolTable::get`'s doc comment cites 16.1 and quotes `"target"`, but the evidence it
rests on is the *pass-or-fail* difference between `signal value 'TARGET'` and
`signal value 'target'`, not the quoted text, and 16.1 is a runtime error the scope decision does
not touch. The five keyword measurements in Step 3b, the `243`/nineteen marker figures, the probe
descriptions and the `span`-not-derivable paragraph involve no error text at all.

**M8 — literals are now the asymmetric case and the plan does not say so.** Symbols carry an
interned id; literals carry nothing stated. A literal's value is *not* a slice of its span —
Task 3.3's own test asserts `literal_text(&scan_ok("'it''s'")) == "it's"`, and the `'…'x` /
`'…'b` suffixes convert bytes — while Step 4 says "Emit spans, never copied strings". One
sentence resolving that is worth more now than before, because I1's fix needs a literal's
decoded value as a label key.

---

## 3. Is the interning design sound?

**Yes, keep it.** It is faithful to the oracle rather than an optimisation bolted onto it, and
the evidence for that is stronger than the plan's own argument.

The C++ scanner does the same thing at the same point: `Scanner.cpp:1512` calls
`commonString(value)` with the comment "so we don't keep around multiple copies of variable
names". The plan presents interning as "the one decision in this task that is not a port"; it
is closer to a port than that.

Three independent facts make the *upcased* interned name the right payload rather than merely a
convenient one, and all three are measured above: an uninitialised symbol's value is its
upcased name, `SIGNAL` resolves a symbol target upcased, and every parse-error substitution that
quotes a symbol quotes the upcased form (`"YZ"`, `"IX"`, `"BAD"`, `"NOPE"`, `"BADOPT"`,
`"STEM.ABAB…"` — six errors, six upcased).
The third of those is no longer an *obligation* under the new error scope, but it is still
evidence: it shows the oracle itself has no use for the source spelling of a symbol anywhere
except `SOURCELINE` and `TRACE`, which read the span. So `SymbolTable::name` is not a debugging
affordance; it is on the path for label resolution and for whatever Phase 4 chooses to render.

The propagation holds. Read as a sequence, Tasks 3.3 → 3.5 → 3.6 → 3.7 → 3.7b → 3.8 → 3.9 line
up: `Clause` carries token ranges and is unaffected; `Expr` and `Instruction` carry byte spans
and are unaffected; Task 3.9 reads spans; Task 3.8 needs only a number and a sub-number under the
new scope and is unaffected either way.
`Fragment` correctly has no `labels` field, which happens to match the C++ forbidding a
label inside `INTERPRET` (`Error_Unexpected_label_interpret`). The one place a symbol id sits
where text belongs is `Program::labels`, and that is I1.

The borrow discipline holds, and I did not take the plan's word for it. I compiled the Task
3.7b order as described — `ProgramSource`, `scan` into a `Scanned`, `ParseCtx` borrowing four
things, parse, drop, move source and table into `Program` — in both the scoped-block and the
explicit-`drop(ctx)` spellings, and both compile. Moving `scanned.symbols` into `Program` after
the borrow ends is a partial move out of `Scanned` and is legal. Nothing needs `&mut` after
parsing starts, because `scan` finishes interning before `ParseCtx` exists, which is exactly
why `scan` returning the table rather than taking `&mut` is the right signature.

The hash-consing decision is recorded correctly and for the right reason: distinct spans make
structurally identical subtrees unequal terms, so sharing would require moving spans into a
side table keyed by the identity sharing destroys. That reason is sound and it is the reason
that survives contact with this AST specifically.

One thing to watch rather than fix: the design makes keyword recognition an integer comparison,
and the plan is careful to keep it positional — the `ParseCtx::keywords` doc comment says
"Keywords are NOT reserved words, so this is only ever consulted positionally", Task 3.6's
Step 2 section is unchanged and still the strongest statement of it in the plan, and
`keyword_as_variable.rex` still gates it. Nothing in either commit weakens that. Verified
unchanged: `-2 ** 2` is 4 (Task 3.5 line 865), `f (x)` versus `f(x)` (Task 3.3 line 618),
keywords not reserved (lines 1096, 1915).

---

## 4. Fix order

1. **I1** — re-key `Program::labels` off the token's value string, and say in Task 3.6 Step 3
   that a literal label keeps its case. Before Task 3.6; cheapest now.
2. **I2** — specify `Keywords`, including which of the six tables it covers, and reconcile Task
   3.7's directive lookup with Task 3.6's "do not build a sorted string table". Before Task 3.3.
3. **I3** — give the Step 2 tests a tag accessor. Before Task 3.3.
4. **I4** — `Cow<'_, str>` in `intern`, and fix the comment. Before Task 3.3.
5. **I5** — scope Global Constraint 3 to say that numbers and sub-numbers are the only error
   property that is contract, and that the rendered text and substitutions are observable through
   `condition('o')` and deliberately not reproduced. Restate the three "no column" passages
   (Task 3.8 Step 4 twice, criterion 5 once) as "the oracle does substitute a byte position in
   error 36 and we do not reproduce it". Do **not** add a per-token line/offset quadruple — that
   question is withdrawn. Land the constraint edit with the criterion-5 relaxation, in the same
   pass, so no reader sees one without the other.
6. **M1–M8** in place, any order. **N6**, **N7** and N1's addendum with them. **N8** remains the
   implementer's call.

No eighth review round is warranted. Everything above is a stated edit; nothing needs
investigation, and every number, citation and probe in this file was run in this session.
