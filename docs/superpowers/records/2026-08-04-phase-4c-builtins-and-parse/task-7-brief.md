### Task 7: the `PARSE` template engine

**Files:**
* Create: `crates/rexx-exec/src/parse_template.rs`
* Modify: `crates/rexx-exec/src/run.rs`, `crates/rexx-exec/src/lib.rs` (`instruction_owner`, `:758`), `tests/owners.rs`, `tests/loud.rs`, `tests/coverage.rs`, `tests/trace_oracle.rs`, `rust/corpus/phase-4c.txt`

**`rust/corpus/phase-4c.txt` does not exist yet. This task creates it**, and `tests/corpus.rs` does not read it until Task 15 Step 4 -- so this task's witness is committed but inert, and that is expected rather than a defect.

**The AST is complete and this task writes no parser code.**
`rexx_parse::ast::Parse` carries `source`, `upper`, `lower`, `caseless`, and `template: Vec<Option<ParseTrigger>>` where `None` is the comma fence between templates.
`ParseTrigger` carries `kind`, an optional `value`, and `targets: Vec<Option<Expr>>` where `None` is a `.` placeholder.
`TriggerKind` has all eight variants: `End`, `Plus`, `Minus`, `Absolute`, `MinusLength`, `PlusLength`, `String`, `Mixed`.

**Sources in this task:** `Var`, `Value`, `Arg`, `Source`, `Version`. `Pull` and `LineIn` are Task 8's.

**`PARSE ARG` here means the argument strings of the *current activation*, which 4b built.
A top-level program's own argument string does not exist yet and is Task 8's.**

- [ ] **Step 0: The template semantics, measured -- read before writing any engine code**

Surveyed and re-verified 2026-08-05. Source is `'abcdefghij'` throughout.

**(a) `Minus` and `MinusLength` are not variants of each other, and conflating them is the likeliest engine bug.**

```
parse value 'abcdefghij' with p 5 q -2 r   ->  [abcd][efghij][cdefghij]
parse value 'abcdefghij' with p 5 q <2 r   ->  [abcd][cd]    [cdefghij]
```

Same movement, unrelated assignment for `q`. **`<n` means "the `n` characters ending at the current position"; `-n` means "move back `n`, then assign to the end".**

**(b) `Absolute` and `Plus`/`Minus` share ONE rule: if the new position is greater than the current, the target gets `[current, new)`; otherwise it gets `[current, END]`.**
Backward movement never assigns the null string.

```
p 5 q      -> p='abcd'         q='efghij'
p 1 q      -> p='abcdefghij'   q='abcdefghij'    1 is not > 1, so remainder
p 11 q     -> p='abcdefghij'   q=''              past the end
p 5 q 5 r  -> r='efghij'                          equal counts as backward
p 5 q -99 r-> r='abcdefghij'                      clamped at 1, never before
p +0 q     -> both the whole string
```

**`>n`/`<n` do NOT follow it** -- they assign an exact slice, clamped at the ends: `p >0 q` and `p <0 q` both give `p=''`.

Errors: a fractional or non-numeric positional is **26.4**; `p =x q` is **38.2**.

**(c) Pattern triggers.** An **absent pattern matches at END**: `p 'z' q` gives `p` the whole string and `q=''`. The **empty pattern behaves as absent**. A pattern at position 1 gives `p=''`. Searches are non-overlapping and the next search starts after the previous match.

**The subtle one: after a string pattern the next *target* starts after the match, but a following *relative* trigger measures from the match START.**
`p 'c' +1 q` and `p 'c' q` are identical; `p 'c' -1 q` gives `q='bcdefghij'`.

**(d) `CASELESS` folds ASCII only, verified against a byte alphabet.**
Pattern `'e9'x` against a source containing `'c9'x` does **not** match; `'c9'x` matches itself exactly. **Every byte `>= 0x80` matches only itself.**
`CASELESS` preserves the original case in assignments; `UPPER`/`LOWER` transform the **source** before parsing.
`upper caseless` and `caseless upper` are both legal and order-independent; `upper lower` is **25.12**.

**(e) The comma fence assigns the null string to every unmatched target -- it never leaves one unset.**
`parse value 'a b' with p , q` gives `q=''`. An omitted *middle* argument empties that template only and does not shift the others.
Extra targets in one template also get `''`, and **only the final target keeps its leading blanks**: `p q r` on `'a  b  c'` gives `r=' c'`.

**(f) `PARSE SOURCE` field 2 varies by CONTEXT, not by call depth.** `LINUX COMMAND` at top level, in an internal subroutine and in an internal function alike; `LINUX METHOD` inside a `::method`.
*Flagged as inference: which fields vary across **hosts** was not measured, since one machine cannot show it.*

**(g) The trace shape, all eight kinds under `trace i`. All trace output is on stderr.**

| construct | lines, in order |
|---|---|
| source `VALUE` | `>L> "<expr>"` · `>K> "VALUE" => "<src>"` · `>>> "<src>"` |
| source `VAR` | `>V> NAME => "<src>"` · `>K> "VAR" => "<src>"` · `>>> "<src>"` |
| source `SOURCE`/`VERSION` | `>K> "<kw>" => "<src>"` · `>>> "<src>"` |
| source `ARG` | **no `>K>` at all** -- straight to `>>>` |
| any positional trigger | `>L> "<n>"` · `>>> "<n>"`, **before** the preceding target's assignment |
| `String` and `Mixed` | `>L>` · `>>>` -- **identical; caseless is not distinguishable in the trace** |
| target | `>=> NAME <= "<value>"` at `trace i`; `>>> "<value>"` at `trace r` -- see below |
| `.` placeholder | `>.> "<consumed>"`, **emitted even when it consumes nothing** |
| `End` | nothing |
| comma fence | `>>> "<next template's source>"` |

**An assigned target's own value line is a choice of prefix, not two independently gated lines**, and this table's `trace i` measurement could not have shown it.
Measured during Task 7 and then confirmed at `instructions/ParseTrigger.cpp:274-280` (`assign` followed by `if (!tracingIntermediates()) traceResult`): under `trace r` a target emits `>>> "<value>"`, under `trace i` it emits `>=> NAME <= "<value>"` **in its place**, and never both.
So a `trace i`-only survey sees `>=>` and concludes `>>>` is absent for targets, which is false at `trace r`.

**The gates, since they do not all follow the construct.** `>K>`, the source `>>>`, the operand `>>>` and the fence `>>>` are gated on `results` and appear under `trace r`.
`>L>`, `>V>`, `>C>`, `>=>` and `>.>` are gated on `intermediates`.
**Do not assume a prefix's gate carries over from `DO`'s use of it** -- measure it for `PARSE`.

**A trigger's numeric operand is evaluated BEFORE the preceding target is assigned** -- for `p 5 q -2 r` the order is `>L>"5"`, `>>>"5"`, `>=>P`, `>L>"2"`, `>>>"2"`, `>=>Q`, `>=>R`.
The traced literal for `+3`/`-2`/`>3`/`<2` is the **bare number without the sign**.

**(h) What the suite checks.** `keyword/PARSE.testGroup` is the **only** PARSE group: 682 `::method` bodies, 792 `assertSame`, **19 `expectSyntax` -- 18× 26.4 and 1× 38.2**.
**25.12 is asserted nowhere** in `ootest/ooRexx/base` despite being reachable via `parse upper lower` (positive control: the same scan finds 26.4 eighteen times).
UPPER/LOWER/CASELESS have **7 assertions out of 792** -- thin but not zero, and concentrated at `PARSE.testGroup:235-251`.

**A false lead already killed, of the `FORMAT`-in-`DATE` kind:** `parse source` occurs **440** times across `base/`, of which **407 are the harness prologue** present in nearly every group and only **6** are in `PARSE.testGroup`. A raw count overstates coverage roughly seventy-fold.

**Asserted outside PARSE's group:** `class/RexxInfo.testGroup:120-121` requires `parse version version` to equal `.RexxInfo~name`, and `:138-139` requires `parse source platform .` to equal `.RexxInfo~platform`.

- [ ] **Step 1: Measure the trace shape, which is not what the groundwork document says**

Measured 2026-08-04 under **`trace i`**, not `trace r`:

```
     2 *-* parse value 'a b c' with p . q
       >L>   "a b c"
       >K>   "VALUE" => "a b c"
       >>>   "a b c"
       >=>   P <= "a"
       >.>   "b"
       >=>   Q <= "c"
```

* **`>.>` fires at `trace i`, not `trace r`.** The groundwork document probed `trace r`, saw nothing, and recorded the absence -- an instrument that could not have produced the answer.
* **`>.>` carries the value the placeholder consumed.**
* **`>=>` is emitted per *assigned* target**; the `.` placeholder gets `>.>` instead, so the two partition the targets.
* **`>K>` carries a `=>` continuation here, and so does every other `>K>` in this crate -- reuse the existing emitter.**
  An earlier revision of this step claimed 4a's and 4b's `>K>` lines carry a bare value and told the implementer to go and measure it.
  That claim was false: `Trace::trace_keyword` (`trace.rs:487`) already calls `push_tagged(.., ">K>", indent, true, keyword, " => ", value)`, so it emits `>K>   "KW" => "value"` with a quoted tag, and `keyword_while.expected` carries `>K>     "WHILE" => "1"`.
  **Do not add a second keyword emitter.**

Probe all eight `TriggerKind` variants under `trace i`. The four a `trace r`-only probe will miss are `Plus`, `Minus`, `MinusLength`, `PlusLength`.

- [ ] **Step 2: Measure `PARSE SOURCE` and `PARSE VERSION` on this host**

Both are host-dependent; `SOURCE` carries an absolute path.
**Their witnesses belong in the live corpus, not in a committed `.expected` file.**

- [ ] **Step 3: Write the failing engine tests, then implement**

The engine is source-independent: byte string plus template in, assignments out.
Keep it so -- Task 8 adds two sources and must add no engine code.
**The comma fence is a template boundary, not a trigger**: a `None` entry advances to the next source string.

- [ ] **Step 4: Move `Parse` in scope**

`tests/owners.rs:165` (`InstructionKind::Parse => Owner::Phase("4c")`) becomes `Owner::InScope`; its row in **`EXPECTED_OUT_OF_SCOPE`** (`owners.rs:347`, **not** a const named `SPLIT_TABLE`) is removed; `src/lib.rs:758`'s arm loses `Parse`.
Its `loud.rs` witness must be **deleted**, not left stale, or `assert_witness_set_is_complete` fails the other way.

**`Arg` and `Pull` stay `4c` until Task 8** -- separate variants, moved separately.

- [ ] **Step 5: Move the `>.>` prefix**

`trace_oracle.rs`: `>.>` becomes `Witnessed`, and **`WITNESSED_PREFIX_COUNT` (`:551`) and `OUT_OF_SCOPE_PREFIX_COUNT` (`:555`) both move by one.**
A `Witnessed` entry is chained to `CLAIMED_PREFIXES` and thence to committed `.expected` bytes, so the witness must actually exist before the constant moves.

- [ ] **Step 6: Run the shared verify block and commit**

---

