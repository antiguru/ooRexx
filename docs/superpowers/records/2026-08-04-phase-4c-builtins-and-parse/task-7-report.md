# Task 7 report: the `PARSE` template engine

Two commits, each hash read back with `git rev-parse HEAD` after committing:

* `da86c10673e07d3d619ce17002158ec3b23cbab2` -- the engine and every witness.
* `26fb60d93082c02697ad841ef194590605687bc3` -- three wrong-answer claims in
  the two corpus programs' own headers, found by re-reading the headers against
  the measured output rather than by any test going red. This is the shape the
  project has measured before: behaviour is settled on the first pass and prose
  is not, so a correction round is where a false statement gets written. The
  `sourceline_oracle` expectations move with the line counts (regenerated with
  that file's own driver), and both programs still match the oracle byte for
  byte on stdout and stderr afterwards.

The working tree is clean.

Throughout, `ORACLE FILE` abbreviates

```bash
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )
```

and `RUST FILE` abbreviates `./target/debug/rexx-run FILE` run from `rust/`.
Every probe ran from a fresh empty directory,
`$SCRATCH/t7probe`, with absolute paths, and stdout, stderr and exit status
were captured as three separate descriptors.

## 1. What was built, and where

### `crates/rexx-exec/src/parse_template.rs` (new)

Two layers, and the split is what Task 8 needs.

`Cursor` is the movement rule and nothing else: the five positions
(`start`/`end` bounding the section a trigger's targets are carved from,
`pattern_start`/`pattern_end` bounding the last match, `subcurrent` tracking
the word carving) and the eleven operations that move them
(`move_to_end`, `forward`, `forward_length`, `absolute`, `backward`,
`backward_length`, `search`, `caseless_search`, `next_word`, `remainder`, plus
`new`). It holds a `Vec<u8>` and no `Interp`, so its unit tests are byte
assertions against measured oracle answers.

`Interp::exec_parse` is the driver: `parse_strings` resolves the source and
emits its `>K>` line, `next_template` steps to the next parse string applying
`UPPER`/`LOWER` and tracing it, `apply_trigger` evaluates a trigger's operand
and moves the cursor, `assign_targets` carves the section into the trigger's
targets. A new source is one new arm in `parse_strings`; nothing in `Cursor`
or in the template walk is source-aware.

Every string the engine assigns goes through `Interp::text` -> `text_owned`
-> `Interp::alloc_with`, the only allocation site wrapper.

### Modified

* `crates/rexx-exec/src/run.rs` -- an `InstructionKind::Parse(parse)` arm
  before the loud fallthrough, calling `exec_parse`. `Assignment`'s target
  dispatch was extracted into `assign_expr_target`, now shared with `PARSE`;
  `shape_of`/`NameShape`/`whole_nonneg` widened to `pub(crate)`.
* `crates/rexx-exec/src/lib.rs` -- `mod parse_template`, a `program_path`
  field on `Interp` (filled by `execute`, empty otherwise) for `PARSE SOURCE`,
  `Loud::parse_source` and `Loud::parse_trigger_operand`, and `Parse` removed
  from `instruction_owner`'s `Some("4c")` group (`Arg`, `Pull` and `Address`
  stay).
* `crates/rexx-exec/src/trace.rs` -- `Interp::trace_dummy`, `>.>`, gated on
  `intermediates`.
* `crates/rexx-exec/tests/owners.rs` -- `Parse` is `Owner::InScope`; its
  `EXPECTED_OUT_OF_SCOPE` row deleted; three counts moved (in-scope
  instructions 31 -> 32, `4c` rows 4 -> 3).
* `crates/rexx-exec/tests/loud.rs` -- the `tag: "Parse"` witness deleted, and
  the two totals it feeds moved (12 -> 11 out-of-scope instruction witnesses,
  31 -> 32 in-scope).
* `crates/rexx-exec/tests/coverage.rs` -- `EXPECTED_SUBSET_4C` and
  `phase_4c_subset_matches_the_committed_list`, and `phase-4c.txt` added to
  the union `every_in_scope_variant_is_witnessed_by_the_phase_subsets` reads.
* `crates/rexx-exec/tests/trace_oracle.rs` -- a `parse_placeholder` witness
  test, its `WITNESS_PREFIXES` row, `>.>` added to `CLAIMED_PREFIXES`, its
  `PREFIX_COVERAGE` row moved from `Owned("4c")` to `Witnessed`, the two
  counts moved (13 -> 14 witnessed, 6 -> 5 out of scope) and the test renamed
  accordingly.
* `crates/rexx-exec/tests/keyword_assertions.rs` -- one prose sentence that
  carried a row count of `keyword-exempt.txt` now names the rows instead.
* `corpus/keyword-exempt.txt` -- 655 rows removed (see section 7).
* `corpus/README.md` -- a "Phase 4c subset" section and a "Phase 4c
  additions" table.

### New witnesses

* `corpus/phase-4c.txt`, `corpus/lang/parse_triggers.rex`,
  `corpus/lang/parse_sources.rex` (the subset lists
  `lang/parse_template.rex` as well, which already existed).
* `crates/rexx-exec/tests/trace_oracle/parse_placeholder.rex` and its
  `.expected`.
* `crates/rexx-parse/tests/sourceline_oracle/parse_triggers.txt` and
  `parse_sources.txt` -- required by
  `sourceline_matches_the_interpreter_for_every_corpus_program`, which reads
  every `corpus/lang/*.rex`. Generated with that file's own documented driver
  (the one sanctioned `.Package~new` exception); both programs took the
  primary `~source` path, and both counts equal `wc -l` (134 and 112 as
  committed), so the fallback caveat about CRLF/CTRL-Z/missing final newline
  does not apply.

## 2. The question the brief asks, answered at the call site

The brief's Step 1 asks whether the existing `trace.rs` keyword path emits the
`=>` continuation, and its parenthetical claims 4a/4b `>K>` lines carry a bare
value.

**Confirmed at the call, not from the doc comment.**
`Trace::trace_keyword` (`crates/rexx-exec/src/trace.rs`) is

```rust
push_tagged(&mut self.trace, ">K>", indent, true, keyword.as_bytes(), " => ", value)
```

`quote_tag` is `true` and the separator is `" => "`, so it already emits
`>K>   "KW" => "value"` -- exactly the shape `PARSE` needs. The parenthetical
is wrong, as the controller said: `keyword_while.expected` (committed, oracle
bytes) contains `>K>     "WHILE" => "1"`. No second emitter was added; `PARSE`
calls this one.

## 3. The question the brief does not ask: the `trace r` gates

**This is the finding of the task.** Every line in the brief's Step 0(g) table
was measured under `trace i`, which sets both `results` and `intermediates`.
Under `trace r` the shape is different, and not by omission of intermediate
lines: **a target's own value line is a choice between two prefixes, not two
independent gates.**

Measured with one program under both modes. `gate_r.rex` is `trace r` followed
by ten clauses; `gate_i.rex` is the identical file with `trace i`:

```bash
$ for m in r i; do sed "1i trace $m" body.inc > gate_$m.rex; done
$ for m in r i; do ORACLE $PWD/gate_$m.rex >out_$m.txt 2>err_$m.txt; echo "mode=$m status=$?"; done
mode=r status=0
mode=i status=0
```

Both wrote nothing to stdout. The first three clauses of each transcript,
verbatim:

```text
--- trace r (err_r.txt) ---
     3 *-* parse value 'abcdefghij' with p1 5 q1
       >K>   "VALUE" => "abcdefghij"
       >>>   "abcdefghij"
       >>>   "5"
       >>>   "abcd"
       >>>   "efghij"
     4 *-* parse var srcv p2 5 q2 -2 r2
       >K>   "VAR" => "abcdefghij"
       >>>   "abcdefghij"
       >>>   "5"
       >>>   "abcd"
       >>>   "2"
       >>>   "efghij"
       >>>   "cdefghij"

--- trace i (err_i.txt) ---
     3 *-* parse value 'abcdefghij' with p1 5 q1
       >L>   "abcdefghij"
       >K>   "VALUE" => "abcdefghij"
       >>>   "abcdefghij"
       >L>   "5"
       >>>   "5"
       >=>   P1 <= "abcd"
       >=>   Q1 <= "efghij"
     4 *-* parse var srcv p2 5 q2 -2 r2
       >V>   SRCV => "abcdefghij"
       >K>   "VAR" => "abcdefghij"
       >>>   "abcdefghij"
       >L>   "5"
       >>>   "5"
       >=>   P2 <= "abcd"
       >L>   "2"
       >>>   "2"
       >=>   Q2 <= "efghij"
       >=>   R2 <= "cdefghij"
```

The `.` placeholder and the comma fence, same two files:

```text
--- trace r ---                        --- trace i ---
     8 *-* parse var srcv p6 . q6           8 *-* parse var srcv p6 . q6
       >K>   "VAR" => "abcdefghij"            >V>   SRCV => "abcdefghij"
       >>>   "abcdefghij"                     >K>   "VAR" => "abcdefghij"
       >>>   "abcdefghij"                     >>>   "abcdefghij"
       >>>   ""                               >=>   P6 <= "abcdefghij"
                                              >.>   ""
    11 *-* parse value 'x y' with a1 , b1    >=>   Q6 <= ""
       >K>   "VALUE" => "x y"
       >>>   "x y"                       11 *-* parse value 'x y' with a1 , b1
       >>>   "x y"                          >L>   "x y"
       >>>   ""                             >K>   "VALUE" => "x y"
       >>>   ""                             >>>   "x y"
                                            >=>   A1 <= "x y"
                                            >>>   ""
                                            >=>   B1 <= ""
```

### The answers, per prefix, for the `PARSE` construct specifically

| prefix | gate | evidence |
|---|---|---|
| `>K>` | `results` | present under both `r` and `i` above. `trace_keyword` is reusable unchanged and needs no differently-gated sibling. |
| `>>>` for the source string | `results` | present under both. |
| `>>>` for a trigger's operand | `results` | `>>> "5"` and `>>> "2"` present under both. |
| `>>>` for the comma fence's next source | `results` | present under both (clause 11). |
| `>L>` for a `VALUE` expression or an operand | `intermediates` | absent under `r`, present under `i`. |
| `>V>`/`>C>` for a `VAR` source | `intermediates` | absent under `r`, present under `i`. |
| `>=>` for an assigned target | `intermediates` | absent under `r`, present under `i`. |
| **`>>>` for an assigned target** | **`results` AND NOT `intermediates`** | present under `r`, absent under `i`. |
| `>.>` for a `.` placeholder | `intermediates` | absent under `r`, present under `i`. |

The last two rows are the finding. Under `trace r` each assigned target emits
`>>> "<value>"`; under `trace i` it emits `>=> NAME <= "<value>"` **in its
place**, never both. Nothing in the brief's table says so, and nothing in a
`trace i`-only survey could.

Confirmed in the C++ afterwards rather than derived from it
(`interpreter/instructions/ParseTrigger.cpp:271`-`286`), which is why the
whole thing is an else-if and not two gates:

```cpp
if (variable != OREF_NULL)
{
    variable->assign(context, variableValue);
    if (!context->tracingIntermediates())
    {
        context->traceResult(variableValue);
    }
}
else
{
    context->traceIntermediate(variableValue, RexxActivation::TRACE_PREFIX_DUMMY);
}
```

and the whole loop sits inside `if (context->tracingResults())`
(`:248`). `traceIntermediate` is
`{ if (settings.intermediateTrace) traceValue(v, p); }`
(`RexxActivation.hpp:339`), which is `>.>`'s gate.

`PARSE ARG` emits no `>K>` at all under either mode, measured separately
(`batt8.rex`/`batt8r.rex`, a routine called with two arguments):

```text
--- trace i ---                        --- trace r ---
     5 *-*   parse arg p1 p2 , q1           5 *-*   parse arg p1 p2 , q1
       >>>     "aa bb"                        >>>     "aa bb"
       >=>     P1 <= "aa"                     >>>     "aa"
       >=>     P2 <= "bb"                     >>>     "bb"
       >>>     "cc dd"                        >>>     "cc dd"
       >=>     Q1 <= "cc dd"                  >>>     "cc dd"
```

Both transcripts are reproduced byte for byte by this crate (section 6).

## 4. The rest of the measurements

### 4.1 Movement: `-n` against `<n`, and the shared backward rule

`batt1.rex`, source `'abcdefghij'` throughout, oracle status 230 (its last
clause is a deliberate 26.4):

```text
A1 [abcd][efghij][cdefghij]     p 5 q -2 r
A2 [abcd][cd][cdefghij]         p 5 q <2 r
B1 [abcd][efghij]               p 5 q
B2 [abcdefghij][abcdefghij]     p 1 q
B3 [abcdefghij][]               p 11 q
B4 [abcd][efghij][efghij]       p 5 q 5 r      (equal counts as backward)
B5 [abcd][efghij][abcdefghij]   p 5 q -99 r    (clamped at 1)
B6 [abcdefghij][abcdefghij]     p +0 q
B7 [][abcdefghij]               p >0 q
B8 [][abcdefghij]               p <0 q
B9 [ab][cdefghij]               p =3 q
B10 [abcdefghij][abcdefghij]    p 0 q
C1 [abc][de][fghij]             p +3 q +2 r
C2 [abc][defghij][cdefghij]     p +3 q -1 r
C3 [abc][de][fghij]             p >3 q >2 r
C4 [abc][bc][bcdefghij]         p >3 q <2 r
C5 [][][abcdefghij]             p <3 q <2 r
C6 [abcdefghij][]               p +20 q
C7 [abcdefghij][]               p >20 q
C8 [][abcdefghij]               p <20 q
C9 [cd][efghij]                 3 p 5 q
C10 [abc][efghij]               p 4 5 q
```

Every row confirms the brief's Step 0(a)/(b) and adds the `>n`/`<n`
combinations it does not list.

### 4.2 Where the two match positions become necessary

`batt1.rex`'s first version got E6/E7 wrong in my own model and re-reading the
output is what caught it. Under `trace i`, `e6.rex`:

```text
     3 *-* parse value d with p 'c' -1 q
       ...
       >=>   P <= "ab"
       >L>   "1"
       >>>   "1"
       >=>   Q <= "bcdefghij"
     4 *-* parse value d with p 'c' -1 q r
       ...
       >=>   Q <= "bcdefghij"
       >=>   R <= ""
```

`Q` starts at index 1, not 2. That is only consistent with `q` belonging to a
following **`End`** trigger rather than to `-1`, which is how `rexx-parse`'s
own `parse_template` groups them: targets attach to the trigger that follows
them, and a trailing `End` trigger is emitted for the ones with no trigger
after them. `p 'c' -1 q` is therefore `('c', [p])`, `(-1, [])`, `(End, [q])`.
This is not visible in the trace at all, because an `End` trigger emits no
line -- it was settled by value, and it is written into the module doc for
that reason.

### 4.3 Expression triggers, and what a bare symbol is

`batt1d.rex` (each counter reset before use, after a first version of this
probe clobbered `nn` with a target assignment and produced three misleading
rows):

```text
D2 [abcdefghij][]        p (nn) q     nn=6 -- a bare (expr) is a PATTERN, searching for "6"
D3 [abcde][fghij]        p =(nn) q    nn=6 -- =(expr) is the absolute column
D4 [abcdef][ghij]        p +(nn) q    nn=6
D5 [abcd][efghij][cdefghij]  p 5 q -(nn) r   nn=2
D6 [abcd][cd][cdefghij]      p 5 q <(nn) r   nn=2
D7 [abc][defghij]        p >(nn) q    nn=3
D8 [abc][fghij]          p (pat) q    pat='de'
D9 [abcdefghij][][]      p nn q       nn=6 -- NN is a TARGET, assigned ''
```

D9 is the one that matters: a bare *variable* symbol in a template is a
target, not a positional pattern. `TriggerKind::Absolute`'s "a bare numeric
symbol" means a numeric literal token.

### 4.4 Patterns, words, fences, placeholders

`batt2.rex`, oracle status 0:

```text
E1 [abcdefghij][]      p 'z' q   -- an absent pattern matches at END
E2 [abcdefghij][]      p '' q    -- the empty pattern behaves as absent
E3 [][bcdefghij]       p 'a' q
E4 [ab][defghij]       p 'c' q
E5 [ab][defghij]       p 'c' +1 q   -- identical to E4
E6 [ab][bcdefghij]     p 'c' -1 q
E7 [ab][bcdefghij]     p 'c' <1 q
E8 [ab][defghij]       p 'c' >1 q
E9 [ab][abcdefghij]    p 'c' =1 q
F1 [a][b][c]           'aXbXc' with p 'X' q 'X' r   -- non-overlapping
F2 [a][b][]            'aXXb' with p 'XX' q 'X' r
F3 [a][bXc]            'aXbXc' with p 'X' q
G1 [a][b][ c]          'a  b  c' with p q r   -- only the last keeps blanks
G2 [a][ b  c]          'a  b  c' with p q
G3 [a][b  ]            '  a b  ' with p q
G4 [a][b][][]          'a b' with p q r s
H1 [a b][]             'a b' with p , q
H2 [a][b][]            'a b' with p q , r
H3 [a b][]             'a b' with p , , q
I1 [one][]             'one two' with p . q
I2 [two]               'one two three' with . p .
I3 [cdef]              'abcdef' with . 3 p
I4 ok                  'a b' with . . .
J1 [MIXED][CASE]       parse upper value 'MiXeD case' with u v
J2 [mixed][case]       parse lower value 'MiXeD case' with u v
J3 [a][bxc]            parse caseless value 'aXbxc' with p 'x' q
J4 [a][b][c]           parse caseless value 'aXbxc' with p 'X' q 'X' r
J5 [MI][ED]            parse upper caseless value 'MiXeD' with p 'x' q
J6 [MI][ED]            parse caseless upper value 'MiXeD' with p 'x' q
K1 []                  parse value with p       -- legal, parses ''
K2 [a][b]              'a'||d2c(9)||'b' with p q  -- a tab is whitespace
```

### 4.5 `CASELESS` over a byte alphabet

`batt3.rex`, oracle status 0, values shown through `c2x`:

```text
L1 [61C962][]      source 'a'||'c9'x||'b', pattern 'e9'x  -- no match
L2 [61][62]        source 'a'||'c9'x||'b', pattern 'c9'x  -- matches itself
L3 [a][b]          source 'aZb',            pattern 'z'
L4 [617B62][]      source 'a'||'7b'x||'b',  pattern '5b'x -- 0x20 apart, not letters
L5 [615F62][]      source 'a'||'5f'x||'b',  pattern '3f'x -- likewise
L6 [61E962][]      source 'a'||'e9'x||'b',  pattern 'c9'x -- the other direction
```

L4/L5/L6 are the dimension the brief only half-covers: it names the high-byte
direction, and the two non-letter ASCII pairs exactly `0x20` apart are the
control I added. `CASELESS` folds ASCII letters and nothing else.

`UPPER`/`LOWER` are ASCII-only too, `batt7.rex`:

```text
R6 [E0C17B41]   parse upper value 'e0'x||'c1'x||'7b'x||'61'x
R7 [E0C17B61]   parse lower value  (same source)
```

### 4.6 `PARSE ARG`

`batt3.rex`, from `call sub 'one two', 'three four', , 'five'`:

```text
M1 [one][two]                            parse arg a1 a2
M2 [one two][three four]                 parse arg b1 , b2
M3 [one two][three four][][five]         parse arg c1 , c2 , c3 , c4
M4 [one two][]                           parse arg d1 , , d3
M5 [ONE]                                 parse upper arg e1 .
M6 [two]                                 parse arg 5 f1
```

M3/M4 are the omission holding its place rather than closing up.

At the top level with no program arguments, `parse arg t1` gives `t1 = ''` and
still traces the source (`edge.rex`, `trace r`):

```text
     2 *-* parse arg t1
       >>>   ""
       >>>   ""
     3 *-* say '[' || t1 || ']'
       >>>   "[]"
```

### 4.7 `PARSE SOURCE` and `PARSE VERSION` on this host

`batt4.rex`, oracle status 0:

```text
N0 [LINUX COMMAND /…/t7probe/batt4.rex]     top level
N1 [LINUX COMMAND /…/t7probe/batt4.rex]     inside an internal subroutine
N2 [LINUX COMMAND /…/t7probe/batt4.rex]     inside an internal function
N3 [REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026]
N4 [REXX-ooRexx_5.3.0(MT)_64-bit][6.06][30 Jul 2026][]
N5 []                                       parse arg at top level
```

(Paths elided here only for width; they are the probe file's own absolute
path, verbatim on both sides.)

Field 2 varies by context, not by call depth -- three contexts, one answer.
N4 shows `PARSE VERSION` has exactly three fields: interpreter name and
version, language level, build date.

**Flagged as inference, not measurement**, carried over from the brief and
narrowed: which words other *hosts* use for fields 1 and 2 was not measured,
because one machine cannot show it. `PLATFORM` and `CONTEXT` are constants
with that caveat in their doc comments.

### 4.8 Error 26.4

`err264.rex`, oracle status 230:

```text
     1 *-* parse value 'abc' with p 2.5 q
Error 26 running /…/err264.rex line 1:  Invalid whole number.
Error 26.4:  Positional pattern of PARSE template must be a whole number; found "2.5".
```

Five operand shapes, each its own one-clause file:

```text
+(-1)     status=230  found "-1"
=(0)      status=0    T [abcdefghij][abcdefghij]
=(-1)     status=230  found "-1"
+(2.5)    status=230  found "2.5"
+('x')    status=230  found "x"
```

And the conversion is bounded by the **active** `NUMERIC DIGITS`, not by a
fixed width:

```text
numeric digits 2 ; parse value d with p +(100) q     status=230  found "100"
numeric digits 2 ; parse value d with p +(1e2) q     status=230  found "1E2"
parse value d with p +(99999999999999999999999999) q status=230  found "99999999999999999999999999"
parse value d with p +(1e30) q                       status=230  found "1E30"
```

`found "1E2"` is D15 showing through: the substitution is the operand value's
own rendering, not a re-rendering. The implementation reuses `run.rs`'s
`whole_nonneg`, which already converts under `self.activation().settings
.digits()`, rather than adding a second conversion rule.

`38.2` (`p =x q`) and `25.12` (`parse upper lower`) are **parse-time** errors
that `rexx-parse` already raises (`instruction.rs`'s `trigger_position` and
its `parse` sub-keyword arm). No engine code was needed for either.

### 4.9 Compound and stem targets

`batt5.rex`, oracle status 0:

```text
P1 [hello][world]      ii=2 ; parse value 'hello world' with aa.ii bb.
P2 [one][two]          ii=3 ; parse value 'one two' with aa.ii cc.
```

and under `trace i`, clause 7:

```text
     7 *-* parse value 'one two' with aa.ii cc.
       >L>   "one two"
       >K>   "VALUE" => "one two"
       >>>   "one two"
       >C>   AA.II => "AA.3"
       >=>   AA.II <= "one"
       >=>   CC. <= "two"
```

A compound `PARSE` target resolves its tail and announces `>C>` exactly as
`a.i = value` does. That is what made sharing `Assignment`'s dispatch the
right move rather than writing a second one.

### 4.10 `PARSE VAR` is a variable read

`batt6.rex`/`batt6b.rex`, oracle status 0:

```text
Q1 [kk][ll]              ii=1 ; aa.1='kk ll' ; parse var aa.ii p q
Q2 [mm][nn]              bb.='mm nn' ; parse var bb. r s
Q4 novalue fired         signal on novalue ; parse var zzunset t
```

```text
     4 *-* parse var aa.ii p
       >C>   AA.II => "AA.1"
       >V>   AA.II => "kk ll"
       >K>   "VAR" => "kk ll"
       >>>   "kk ll"
       >=>   P <= "kk ll"
     6 *-* parse var bb. r
       >V>   BB. => "mm nn"
       >K>   "VAR" => "mm nn"
```

So `read_parse_var` mirrors `eval_node`'s three arms including the
`NOVALUE` check for a simple and a compound name and its absence for a bare
stem.

### 4.11 Dimensions varied deliberately

`batt9.rex`, chosen for axes the earlier batteries held fixed (a builtin call
as an operand, a stem *and* a compound target in one template, a 200-byte
source, a `PARSE` inside a `DO` under `trace i` so the traced indent is not
column zero):

```text
S1 [cdef][ghij]        . 3 p . 7 q
S2 [abc][defghij]      p +(length('abc')) q
S3 [][]                parse arg t1 t2 at top level
S4 [x][][]             'x' with a1 , b1 , c1
S5 [MI][ED WORDS]      parse upper caseless value 'MiXeD wOrDs' with p1 'x' p2
S6 [mi][ed words]      parse lower caseless value 'MiXeD wOrDs' with p3 'X' p4
S7 [][]                jj=4 ; 'w1 w2' with st.jj , st.
S8 [99][eabcd][96]     copies('abcde',40) with q1 100 q2 +5 q3
S9 [p2]                two passes of a PARSE inside a DO, traced
```

S7 is worth a note because it looks wrong and is not: the first template
assigns `st.4`, then the second template's `st.` assignment *replaces the
whole stem object*, so both reads come back as the new stem's `''` default.
Both interpreters agree.

## 5. Guards fired deliberately

Six mutations, each applied, run, and reverted from a copy (never
`git checkout`).

| # | mutation | result |
|---|---|---|
| 1 | `trace_dummy` gated on `results` instead of `intermediates` | `trace_oracle`: `parse_placeholder … FAILED`, "stderr" |
| 2 | the target `>>>` emitted unconditionally (`if true`) | `trace_oracle`: `parse_placeholder … FAILED`, "stderr" |
| 3 | `TriggerKind::MinusLength` dispatched to `Cursor::backward` | `parse_template` unit tests: **8 passed, 0 failed** |
| 4 | a placeholder that consumed nothing emits no `>.>` | `trace_oracle`: 22 passed, 1 failed |
| 5 | 26.4 replaced by `whole_nonneg(...).unwrap_or(0)` | `parse_template` unit tests: 10 passed, 1 failed |
| 6 | `lang/parse_sources.rex` deleted from `phase-4c.txt` | `coverage`: `phase_4c_subset_matches_the_committed_list … FAILED` |

**Mutation 3 is the important one and it exposed a real gap in my own
tests.** The `Cursor` unit tests call the movement methods by name, so
swapping which method `apply_trigger` dispatches to left all eight of them
green. That is exactly a test that cannot catch the defect it looks like it
covers. I added
`every_trigger_kind_and_source_reaches_its_own_operation` and
`source_and_version_carry_their_own_strings`, which run programs through
`crate::run_program` -- so the `step` arm and the dispatch are inside the
subject -- against oracle stdout, with the rows chosen so no two kinds agree
on their own row (`+n` and `>n` coincide for a non-zero offset, hence the
`+0`/`>0` pair; `-n` and `=n` coincide on the second field, hence the third).
Re-run after adding it, three dispatch swaps each go red:

```text
applied: MinusLength => cursor.backward_length -> MinusLength => cursor.backward
test result: FAILED. 10 passed; 1 failed
applied: Plus => cursor.forward(offset) -> Plus => cursor.forward_length(offset)
test result: FAILED. 10 passed; 1 failed
applied: TriggerKind::Mixed => cursor.caseless_search -> TriggerKind::Mixed => cursor.search
test result: FAILED. 10 passed; 1 failed
```

On "can fail is not adds coverage": for mutations 1, 2 and 4 the
`parse_placeholder` witness is the only guard in the workspace at all --
`corpus/lang/parse_sources.rex` covers the same ground but `tests/corpus.rs`
does not read `phase-4c.txt` until Task 15, so that program is committed and
inert by design. For mutation 3 the check was run the other way round, as
above: the pre-existing tests do not catch it.

## 6. Differential verification against the oracle

Every probe file above, plus the three corpus programs, run through both
interpreters and compared on stdout, stderr and exit status separately.

```text
$ for pair in batt1 batt1d batt2 batt3 batt4 batt5 batt6 batt6b batt7 batt8 \
              batt8r batt9 batt10 gate_r gate_i edge edge2 \
              corpus/lang/parse_triggers corpus/lang/parse_sources \
              corpus/lang/parse_template ; do … diff … ; done
ALL_MATCH=yes
```

Byte-identical throughout, including:

* `batt1`'s 26.4 report with its absolute path and exit status 230 on both
  sides.
* `batt10`, the 26.4 path **under `trace r`**, where the target `>>>` for the
  first assignment appears before the operand `>>>` that then fails:

```text
     3 *-* parse value d with p 5 q +(2.5) r
       >K>   "VALUE" => "abcdefghij"
       >>>   "abcdefghij"
       >>>   "5"
       >>>   "abcd"
       >>>   "2.5"
     3 *-* parse value d with p 5 q +(2.5) r
Error 26 running /…/batt10.rex line 3:  Invalid whole number.
Error 26.4:  Positional pattern of PARSE template must be a whole number; found "2.5".
```

* `batt9`'s traced `PARSE` inside a `DO`, whose trace indent is not column
  zero.
* `batt5`/`batt6b`/`batt8`/`batt8r`/`gate_i`/`gate_r`, all of stderr.

Two deliberate non-matches, both loud disclosures rather than wrong answers:

```text
$ ORACLE pp.rex        # queue "line" ; parse pull v ; say v
status=0  stdout=[line]
$ RUST pp.rex
status=120  stderr=rexx-exec: PARSE PULL is not implemented (4c)

$ ORACLE edge3.rex     # parse value 'a b' with q~x r
status=159
Error 97 running /…/edge3.rex line 1:  Object method not found.
Error 97.1:  Object "Q" does not understand message "X=".
$ RUST edge3.rex
status=120  stderr=rexx-exec: a message send is not implemented (Phase 5)
```

The second corrected a comment I had just written. A `PARSE` target is
`parseVariableOrMessageTerm`, so `assign_expr_target`'s fourth arm **is**
reachable from `PARSE` while staying unreachable from `Assignment`, whose
targets are only what `addVariable` builds. The doc comment I moved said
"unreachable through any program that parsed"; running it is what showed that
false, and the comment now states both callers.

## 7. `base/keyword`'s PARSE group

`cargo test -p rexx-exec --test keyword_assertions` in report mode, before
`keyword-exempt.txt` was pruned:

```text
779 of 896 bodies passing, carrying 1537 of 1773 assertSame calls
  PARSE                 653/659     763/778
not passing, by what would unblock it:
  4c                                       111
  defect:compound-do-control-variable      6
first construct hit, for a body that failed loudly:
  …
  PARSE PULL                               5
  PULL                                     1
```

655 rows became stale, with **no** new failure and **no** moved blocker
(`/bin/grep -cE "is failing|blocker moved"` over the panic output: 0 matches;
`/bin/grep -c "now PASSES"`: 655). Of those 655, 653 are PARSE bodies; the
other two are `ASSIGNMENT::test_7` and `DO::test_DO_standardTest5-70`, which
used `PARSE` incidentally. I removed exactly those 655 keys from
`corpus/keyword-exempt.txt`; the file now holds 117 rows, 111 `4c` and 6
`defect:compound-do-control-variable`.

The six PARSE rows that remain are all Task 8's, read directly out of
`ootest/ooRexx/base/keyword/PARSE.testGroup`:

* `Test_614`, `Test_620`, `Test_626`, `Test_632`, `Test_638` -- each is
  `Queue '…'` followed by `Parse Pull , c`.
* `test_PARSE_variable_patterns` -- `PUSH "11/15/98"` then `pull date`, the
  `PULL` instruction.

## 8. The `PARSE VERSION` decision, flagged

`PARSE VERSION`'s three fields are the interpreter's name and version, the
language level, and the interpreter's **build date** (`30 Jul 2026` here).
Nothing in this crate can derive the third. The commit before this one
(`f322477f`) records the standing hazard directly: "a rebuilt oracle silently
reprices every differential result and nothing here stores a build identity
beside any number."

What I did: implemented the source from a single `const VERSION` recorded
verbatim from the measurement, documented as a claim about the oracle build
present on 2026-08-05, pinned by
`the_version_string_is_the_measured_oracle_string`, and reproduced
differentially today (`batt4.rex` matches byte for byte). **No corpus program
prints it**, so no live differential depends on it, and the `phase-4c.txt`
header and `parse_sources.rex`'s header both say why.

This is a judgement call and the controller may want a different one. The
alternatives, both worse in my reading: report this crate's own identity, which
guarantees a differential mismatch for a construct that otherwise works; or
leave `ParseSource::Version` loud, which makes `parse version` fail where the
whole rest of `PARSE` runs. If the oracle is rebuilt, this constant is the one
thing in the change that goes stale, and its unit test is what says so.

`PARSE SOURCE` has the same shape of problem in a milder form: field 3 is the
program's own absolute path, so it is reproduced exactly (the path
`run_program` was handed) but no corpus program prints it, matching what
`corpus/lang/source_arg.rex` already did.

## 9. Everything flagged as inference rather than measurement

1. **Which words `PARSE SOURCE`'s first two fields carry on other hosts.**
   Not measurable on one machine. `PLATFORM = b"LINUX"` and
   `CONTEXT = b"COMMAND"` carry that caveat in their doc comments. The
   *context-not-depth* half was measured (three contexts, one answer).
2. **`PARSE VERSION`'s string beyond this build.** Section 8.
3. **`Cursor::next_word`'s leading-blank skip being bounded by the whole
   string rather than by the section end.** This is read off
   `ParseTarget.cpp:423`-`433` (the C++ scans for a non-blank relying on the
   string's terminating NUL and only then tests the section end) and mirrored
   exactly; I did not construct a probe that distinguishes it from the
   section-bounded version, because every later read tests `subcurrent >= end`
   and every movement resets `subcurrent`, so I could not find an observable
   difference. Flagged rather than claimed closed.

Nothing else in the change rests on inference: every semantic rule in
`parse_template.rs` has an oracle transcript behind it, and the trace shape
has one per mode.

## 10. Verification, all from `rust/`

```text
$ cargo test --workspace
test EXIT=0        (no FAILED lines. rexx-exec, per binary: lib 395,
                    assertions 5, builtin_status 12, collect_stress 2,
                    corpus 9 + 1 ignored, coverage 11, keyword_assertions 7,
                    loud 8, owners 5, spike 8, trace_oracle 23)

$ cargo fmt --all --check
fmt EXIT=0

$ REXX_CORPUS_GATE=1 cargo test --workspace
42 of 42 matching
gate EXIT=0

$ rm -rf $SCRATCH/cleantarget2
$ CARGO_TARGET_DIR=$SCRATCH/cleantarget2 \
    cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.84s
clippy EXIT=0
```

Every exit status was read unpiped or through `${PIPESTATUS[0]}`; clippy ran
from a target directory created empty for it, per the phase-boundary rule.

`git status --short` after committing: empty.

## 11. Notes for whoever takes Task 8

* `parse_strings` is the only place a source is resolved. `ParseSource::Pull`
  and `ParseSource::LineIn` are two `Loud::parse_source` lines to replace with
  a queue read and a console read; nothing in `Cursor`, `next_template`,
  `apply_trigger` or `assign_targets` is source-aware, and the `PARSE ARG`
  arm already shows how a source contributes more than one string.
* A top-level program's own argument string is still absent:
  `Interp::call_context.arguments` is empty for the outermost activation, so
  `parse arg` there parses `''`, which is what the oracle does for
  `rexx prog.rex` with no arguments. Task 8 owns making a supplied argument
  visible.
* The `ARG` and `PULL` **instruction** spellings are separate
  `InstructionKind` variants, still `Owner::Phase("4c")` in `owners.rs` with
  their `loud.rs` witnesses intact. They reach `rexx_parse::ast::Parse` too,
  so `exec_parse` should serve them unchanged once their `step` arms exist.
* `corpus/phase-4c.txt` exists and is read by `tests/coverage.rs` only. Task
  15 Step 4 is what makes `tests/corpus.rs` read it; until then the three
  programs it names are committed and inert, and `REXX_CORPUS_GATE=1` stays at
  42 of 42.

---

# Fix round 1

Commit `73802e7b9715a77de858f65984a0fac7c78450c5`, read back with
`git rev-parse HEAD`. Verdicts carried in: spec compliance PASS, task quality
strong, no correctness defect. Four items, all in prose or in a test's shape,
none in the engine.

## Important 1 -- the `VERSION` pin was a self-comparison, and my report claimed it was not

The reviewer is right and the claim in section 8 of this report ("its unit test
is what says so") was false. The test was

```rust
assert_eq!(VERSION, b"REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026", ...)
```

-- the constant against a literal copy of itself. It goes red only when
someone edits one side and forgets the other, and stays green through any
number of oracle rebuilds, which is precisely the staleness I had correctly
identified as the risk. So the risk was guarded by nothing and the report said
it was guarded. That is the `rust/CLAUDE.md` Method entry "a test that cannot
fail is a defect", and it is worse than the plain form: a self-comparison with
a doc comment claiming rebuild detection is a guard that also tells you not to
look.

**Fixed by making the check consult the oracle**, in a new harness
`crates/rexx-exec/tests/parse_version_oracle.rs`, built on the machinery
Task 1 already put in `tests/support/oracle.rs` (`locate`, `Oracle::run` with
the `ulimit -v` wrapper, `descriptor_diffs`) rather than a second copy of a
subprocess wrapper. It writes `parse version v ; say v` into a directory of its
own, runs it through both interpreters, and compares all three channels, so the
assertion carries no copy of the string at all. It also asserts
`oracle.invocations() == 1`, so a version of it that compared this crate's
answer against nothing cannot pass.

Gated on `REXX_CORPUS_GATE`, as instructed. **Negative controls, both run:**

```text
$ (move the recorded build date by one day, 30 Jul -> 31 Jul)
$ REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test parse_version_oracle
test parse_version_still_answers_what_the_oracle_answers ... FAILED
PARSE VERSION disagrees with the oracle on [stdout]. …
test result: FAILED. 7 passed; 1 failed

$ cargo test -p rexx-exec --test parse_version_oracle      # same mutation, no gate
test parse_version_still_answers_what_the_oracle_answers ... ok
test result: ok. 8 passed; 0 failed
```

So it goes red on a moved build date under the gate and skips without it. The
skip prints a line saying a green run there is not evidence the constant is
current, rather than passing silently.

**One thing about the gate I have to state rather than imply, and it is in the
file's own module doc:** it does not make an offline `cargo test` green.
`tests/builtin_status.rs` invokes the oracle **66 times with no gate at all**
on a plain `cargo test` (five `#[test]`s, none `#[ignore]`d,
`support::oracle::locate()` called unconditionally), so a machine without the
oracle already cannot run this crate's default suite. The gate is followed here
because it is the convention for a check whose subject is the oracle, not
because it restores a property the workspace has.

## Minor 5 -- one copy of the string, not three

`source_and_version_carry_their_own_strings` embedded the literal a second
time. It now builds the expected line from `VERSION` itself, which is all a
test inside this crate can honestly ask (does the `Version` source reach that
constant), leaving *whether the constant is still the oracle's answer* to the
harness that can tell. The tautological unit test is deleted, and the
module-doc paragraph in the new harness that quoted the old assertion no longer
reproduces the bytes either -- it describes the shape instead, which also drops
a "the version this replaces" narration that was history in a comment.

```text
$ /bin/grep -rn "REXX-ooRexx_5\.3\.0" rust/crates rust/corpus
rust/crates/rexx-exec/src/parse_template.rs:103:const VERSION: &[u8] = b"REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026";
```

One occurrence in the tree. `VERSION`'s own doc comment now points at the
harness and says plainly that an assertion comparing the constant to a literal
copy of itself would go green through every rebuild.

## Important 2 -- "five of the seven `ParseSource` variants" was four

`corpus/README.md` said five; the subset constructs four: `Value`, `Var`,
`Arg`, `Source`. The same commit's `phase-4c.txt` header and
`parse_sources.rex` header both already said `PARSE VERSION` appears in no
program, so the README contradicted two files it shipped beside.

**How the wrong number got there, because it is a repeat of a known failure
mode.** My own enumeration was a grep over the three programs:

```text
$ /bin/grep -aoiE "parse +(upper +|lower +|caseless +)*(value|var|arg|source|version|pull|linein)" …
-- lang/parse_sources.rex
arg source value var version
```

`version` is there. It is in the file's **header comment** -- the sentence
saying `PARSE VERSION` is deliberately absent -- and there is no `parse
version` clause anywhere in the file:

```text
$ /bin/grep -ain "version" rust/corpus/lang/parse_sources.rex
36:   PARSE VERSION is deliberately absent. Every field it carries is the
```

A grep over source counted a comment as a construct. That is the
"`grep -c` on code is my most frequent wrong number" entry, and it produced a
number that agreed with a plausible expectation, which is why it survived.

**Neighbourhood re-read, in the same edit**, because a correction round is this
defect's habitat:

* The count is now the four names spelled out, with `Version` stated as
  implemented and deliberately unprinted and `Pull`/`LineIn` as loud, so the
  number explains itself instead of needing to be recounted.
* The subset section said `lang/source_arg.rex` and `lang/parse_sources.rex`
  "both project it away", which is true but invited a reader to conclude
  `source_arg.rex` is in the subset. It is not, and must not be: it calls
  `ARG` and `SOURCELINE`, both of which `corpus/builtin-status.txt` measures
  as `loud`. The sentence now says so.
* That section also said "Two things it admits that no earlier subset does"
  and, after this edit, would have listed three. The count is gone; it now
  names what it admits without counting.
* `PARSE VERSION`'s absence is now stated in the subset section too, next to
  the `PARSE SOURCE` rule it shares a reason with, so the two are not recorded
  in only one of the three files that care.

## Minor 4 -- `parse_triggers.rex` named the wrong field

The backward block said an engine assigning the null string for equal or
backward movement "prints B4's third field and B5's third field empty". The
third field belongs to the trailing `End` trigger; the defect empties the
**second**, which is the target the equal or backward trigger itself assigns.

**Both halves of the replacement were measured by mutating the engine, not
reasoned.** Making `absolute`'s backward branch and `backward` set
`end = start`:

```text
B4 [abcd][][efghij]        (correct: [abcd][efghij][efghij])
B5 [abcd][][abcdefghij]    (correct: [abcd][efghij][abcdefghij])
```

Second field emptied, third untouched -- so the old sentence was false and the
new one is true. And making `backward` leave the match position unmoved, which
is what the third field actually pins:

```text
A1 [abcd][efghij][efghij]
B5 [abcd][efghij][efghij]  (correct: [abcd][efghij][abcdefghij])
```

The block now says both: the second field is what the null-string defect
empties, and the third is where the backward branch leaves the match position,
with `-99`'s clamp to the origin as the visible consequence.

`parse_triggers.rex` gained lines, so
`crates/rexx-parse/tests/sourceline_oracle/parse_triggers.txt` was regenerated
with that harness's own driver (count 139, equal to `wc -l`), and all three
subset programs were re-run against the oracle afterwards: `parse_triggers`,
`parse_sources` and `parse_template` each identical on stdout and stderr.

## Verification, re-run from `rust/`

```text
$ cargo test --workspace
test EXIT=0        (no FAILED lines)

$ cargo fmt --all --check
fmt EXIT=0

$ REXX_CORPUS_GATE=1 cargo test --workspace
42 of 42 matching
test parse_version_still_answers_what_the_oracle_answers ... ok
gate EXIT=0

$ rm -rf $SCRATCH/ct3
$ CARGO_TARGET_DIR=$SCRATCH/ct3 cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.74s
clippy EXIT=0
```

`git status --short` after committing: empty.

## What section 8 of the original report should now be read as saying

The decision stands -- keep the `const` -- and the coordinator's ruling is
recorded. The sentence "its unit test is what says so" was false when written
and is now true of a different test: `tests/parse_version_oracle.rs` under
`REXX_CORPUS_GATE`, which has been watched to fail on a moved build date. The
generalisation worth keeping is that I stated a guard's *effect* from its
intent rather than from running it, and the fix for that is the same one this
project keeps arriving at: make the guard fire before claiming it guards.
