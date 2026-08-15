STATUS: DONE

# Task 9 report: the instruction loop -- assignment, SAY, DROP, NUMERIC, EXIT, LABEL, NOP

Scope, per the dispatch's corrections: `rust/crates/rexx-exec/src/run.rs` (new),
`rust/crates/rexx-exec/src/lib.rs`, `rust/crates/rexx-exec/src/bin/rexx-run.rs`,
and this report. Filling this in as I go.

## Pre-flight reading

The task-9 brief in full, then the dispatch's four corrections against the
tree (all four confirmed correct on inspection): the `step` signature really
is `fn step(&mut self, code: &Code<'_>, instruction: &Instruction) -> Result<Flow, Failure>`,
not the brief's stale `fn step(&mut self, body: &CodeBody, index: usize) -> Result<Flow, Raised>`;
`Flow` already exists with `Next`/`Goto(usize)`/`Exit(Option<ObjRef>)`;
`SAY`, `Assignment`'s `Variable` target, `Exit { expression: None }` and
`Interpret` under the spike flag are already implemented; the brief's
`git add .../tests/run_basic.rs` line contradicts its own Files section and
the project's private-subject-gets-a-`#[cfg(test)] mod tests` rule.

Then `lib.rs` end to end (all 1259 lines, before any edit) to find the exact
span to move and everything `run_activation`/`step`/`step_in_temps_frame`/
`run_fragment` touch on `self`; `error.rs` end to end (`Raised`'s fields are
`pub(crate)`, `Failure`, `ClauseSite`); `activation.rs` (`Activation::settings`,
its own doc on why it is per-activation); `plan.rs` end to end (`Plan::names`
vs `by_symbol`, `slot_of`'s three-source resolution, `slot_of` does **not**
upcase); `stem.rs` end to end (`tail_key`, `stem_get`/`stem_set`/`stem_assign`/
`stem_drop`/`stem_drop_tail`, all already implemented behind a blanket
`#[allow(dead_code)]` on the whole `impl Interp` block -- stale even before
this task, since `tail_key`/`stem_get` are already called from `eval.rs`'s
`Compound` read arm, but `stem.rs` is not a file this task may write, so the
allow is left as found); `value.rs` end to end (`to_text`/`to_number`, the
`Cow<[u8]>` shape); `eval.rs` end to end, particularly `eval_prefix`'s
`PrefixOp::Minus` arm (unary minus is arithmetic, rounds to the *active*
`NUMERIC DIGITS` at creation -- this turns out to be load-bearing for `EXIT`,
below) and the existing test helpers' `activate`/`Code { slots: &HashMap::new(), ... }`
pattern, copied into `run.rs`'s own test module rather than shared, matching
every other test module in this crate. `rexx-parse`'s `ast.rs` for
`InstructionKind::{Drop,Numeric,Exit,Label,Nop}`, `NumericSetting`,
`VariableRef`, `compound_parts`; `instruction.rs::numeric` for exactly what
each `NumericSetting` variant's parser produces (in particular, that `DIGITS`/
`FUZZ` alone parse to `expression: None` rather than being a parse error, and
that `FORM VALUE` with no expression is a parse-time 35.917, never reaching
this crate as `expression: None`); `scanner.rs`'s `scan_symbol` for the exact
Stem-vs-Compound classification rule (traced via a sub-agent, `scanner.rs`
lines ~723-729 and ~856-875: `dot_count == 0` is `Variable`, `dot_count == 1
&& bytes[last] == b'.'` is `Stem`, anything else with a dot is `Compound`);
`rexx-num`'s `settings.rs` end to end (`Settings::set_digits_str`/
`set_fuzz_str`/`set_form_str`, `SettingsError`'s four variants, and that its
own `sub_code` is private, unlike `ArithError`'s, which `error.rs`'s own doc
says was made `pub` expressly for that one caller); `rexx-num/src/lib.rs`'s
`Number::whole_value` (the `checkIntegerDigits`-ported whole-number check
`EXIT`'s conversion turns out to need) and `ARGUMENT_DIGITS`/`DEFAULT_DIGITS`;
`rexx-core/src/roots.rs`'s `RootSet::clear_slot` (added expressly for `DROP`,
never `set_slot(frame, i, None)`, and never `ObjRef::NIL`, a value and not an
absence).

## Investigation

### `DROP`'s three shapes and the Direct/Indirect split

The brief says to upcase an indirect name before resolving it and never write
`ObjRef::NIL` for "dropped" -- both measured and correct, and both already
covered by tests below. What the brief does not spell out, and needed working
out from `scanner.rs` plus oracle transcripts: **`DROP` needs to tell a bare
stem (`A.`) apart from a real compound (`A.1`, `A.B`) before deciding whether
to call `stem_drop` or `stem_drop_tail`**, because `compound_parts("A.")`
returns `("A.", [Tail::Constant("")])` -- one tail, not zero -- so a drop path
that just called `tail_key` uniformly for every dotted name would tombstone a
single empty-keyed tail on a bare stem instead of replacing the whole stem
object, which is a different, wrong operation (measured: `stem_drop` resets
`say x.` to the untouched-looking `X.`; `stem_drop_tail` with an empty key
would instead leave a real object with one tombstoned tail, and `say x.` would
still find that object and (with no default) derive `X.` anyway for *this*
particular follow-up check, but `say x.` after a bare-stem drop and after a
compound-shaped, wrongly-routed one only look the same by accident of what
gets asked next -- they are not the same operation, and Task 5's own
`replace_stem` vs `stem_drop_tail` doc comments say why). So `run.rs` has its
own `shape_of`/`NameShape` (`Simple`/`Stem`/`Compound`), reproducing the
scanner's rule as a pure function of bytes, used identically for `Direct` and
`Indirect` targets.

**`Direct` and `Indirect` then resolve a compound-shaped name two genuinely
different ways, and getting this wrong is the second real trap.** A `Direct`
name came through the scanner, so a real compound's tail pieces are still
symbols to resolve (`tail_key`, exactly what reading or assigning `a.b` already
does). An `Indirect` name is a plain runtime string with nothing left to
resolve -- everything after the first period is the tail key **verbatim**,
including any further periods, never re-parsed as source. Discriminated on
the oracle:

```
v = 'A.1.2'; a.1.2 = 'x'; drop (v); say a.1.2   ->  A.1.2  (matches a direct drop a.1.2)
v = 'A.B'  ; a.b   = 'x'; drop (v); say a.b     ->  A.B
```

The second line is the one that actually discriminates "verbatim" from
"re-resolved as a second variable lookup": an unset piece variable's derived
name equals its own (upcased) spelling, so a naive test using an unset `B`
cannot tell "the key is the literal byte `B`" apart from "the key is `B`'s own
value, which happens to be `B`" -- confirmed the hard way, see "A dead end"
below. `drop_variable`'s own doc comment in `run.rs` carries this transcript.

**Upcasing.** Measured, `v = 'x'; x = 1; drop (v); say x` -> `X`: the wrapper's
*value* is upcased, matching what the scanner would have done to `x` had it
been written directly (`SymbolTable::intern`'s own `to_ascii_uppercase`, byte-
identical to `translateChar`). `to_ascii_uppercase()` on the indirect name's
bytes (not `SymbolTable::intern`, which is `rexx-parse`-private) is what
`drop_variable` uses.

### `NUMERIC DIGITS`/`FUZZ` with no expression

The parser (`instruction.rs::numeric`) accepts `NUMERIC DIGITS`/`NUMERIC FUZZ`
with `expression: None` -- not a parse error -- and the brief's own step list
says to test this. Measured what it does, since neither `settings.rs` nor the
brief says: it resets to the package default, reported **exactly as if the
default's own text had been typed**, not as a no-op or a sentinel:

```
numeric digits 3; numeric digits; say 1/3          -> 0.333333333  (DIGITS 9's rendering)
numeric fuzz 3   ; numeric fuzz  ; say (1.001 = 1)  -> 0            (FUZZ 0: exact comparison)
numeric digits 20; numeric fuzz 15; numeric digits  -> Error 33.1, ("9") vs ("15")
```

The third line is the one that pins "as if typed": if the reset were a
special no-op path that skipped `set_digits_str`'s own validation, this would
silently leave DIGITS at 20 instead of raising. It raises, with the *rejected
candidate* substitution reading `"9"`, exactly matching `NUMERIC DIGITS 9`
typed literally at that point. So the implementation is
`numeric_operand(code, expression, "9")` / `numeric_operand(..., "0")` --
evaluate `expression` if present, otherwise hand the literal default text to
the identical `set_digits_str`/`set_fuzz_str` call the explicit form uses, no
separate code path.

`NUMERIC FORM` alone (`NumericSetting::FormDefault`) resets to `SCIENTIFIC`
the same way: `numeric form engineering; numeric form; say 1e10 + 0` ->
`1E+10` (not `10E+9`). 4a has no `::OPTIONS` to move the package default away
from `Scientific`, so `FormDefault` and `FormScientific` do the identical
thing today; the arm is commented to say a later `::OPTIONS FORM` is what
should split them apart, rather than something this task invents ahead of
having the construct that would need it.

`NUMERIC FORM VALUE`'s own rule, from `set_form_str`'s doc comment and
confirmed on the oracle: the runtime `VALUE` path does no uppercasing, no
trimming, no abbreviation -- `numeric form value 'engineering'` (lowercase)
is 25.11, not accepted case-insensitively the way the keyword spelling
`NUMERIC FORM ENGINEERING` is (that one is already uppercase by the time the
token reaches this crate, since the *scanner* uppercases keywords, not
`set_form_str`).

### `EXIT`'s numeric conversion, and the asymmetry that looked like a sign bug

This is the fact the dispatch flagged as unmeasured and mine to work out.
`rexx-run.rs`'s existing comment records four data points (`exit 256 -> 0`,
`257 -> 1`, `-1 -> 255`, `255 -> 255`) and says the fix is `value mod 256`.
Measuring past that comment's own four points immediately produced results
that looked contradictory: `exit 2147483647` (`INT32_MAX`) gives rc 255
(matches `mod 256`), but `exit 1000000000` -- a *smaller*, in-range positive
number -- also gives rc 0, indistinguishable at the `$?` level from a genuine
rejection. Chased down: **`1000000000` is an exact multiple of 256**, so "it
converted successfully and the low byte happens to be 0" and "it was rejected
and fell back to the default of 0" are the same observable rc. Every probe
using a "round" test number (`1500000000`, `2000000000`, ...) hit this same
coincidence and looked like a huge, unexplained rejected range in the middle
of the positive `i32` span. Re-ran with values whose true `mod 256` is
nonzero (`1000000001`, `2147483641`, ...) and the coincidence disappeared:
**every positive value up to and including `2147483647` converts and
truncates correctly; nothing about the `1e9`-to-`2.1e9` range is special.**
The actual boundary is exactly `i32::MAX`/`i32::MIN`, confirmed by bisection
around `2147483647`/`2147483648` and `-2147483648`/`-2147483649`.

The remaining, real asymmetry: `exit -2147483647` gives rc 0, but the
*positive* `exit 2147483647` gives rc 255, and both are within `i32` range.
Dispatched a sub-agent to trace the C++ (`interpreter/instructions/
ExitInstruction.cpp`, `interpreter/api/ThreadContextStubs.cpp::ObjectToInt32`,
`interpreter/runtime/Numerics.cpp::objectToSignedInteger`,
`interpreter/classes/IntegerClass.cpp::RexxInteger::minus`); its finding,
independently re-verified against the oracle here (`numeric digits 20; exit
-2147483647` -> rc 1, matching `-2147483647 mod 256` exactly): the int32
bound itself (`INT32_MIN..=INT32_MAX`) **is** symmetric. The asymmetry is
`-2147483647` being `PREFIX-MINUS(2147483647)`, never a literal (Rexx's
tokenizer never folds a sign into a numeric literal), and prefix `-` is
arithmetic, which rounds to the *active* `NUMERIC DIGITS` (9 by default) the
moment the result is created (`RexxInteger::minus`/`NumberString::minus`,
C++; `eval_prefix`'s `Number::zero().sub(&number, digits)`, this crate,
already built by Task 7). `0 - 2147483647` rounded to 9 significant digits is
`2147483650` (round-half-up on the dropped `7`), one past `INT32_MAX`, so the
*rounded* value is what gets range-checked and rejected -- not the literal
`-2147483647` a naive reading of the number suggests. A bare positive literal
never passes through arithmetic at all, so it never gets rounded, and the
only bound left is the `i32` one.

This resolved into `Interp::exit_code_for` needing **no special casing at
all** for "was this value produced by arithmetic": D15's own rule (a number's
precision is fixed at creation, `value.rs`'s module doc) already guarantees
`to_number(value)` hands back whatever was actually stored, rounded or not,
so the asymmetry falls out for free from machinery Task 7 already built. The
remaining decision was the *width* to check whole-number-ness at:
`rexx_num::Number::whole_value` takes a `digits` parameter, and the oracle's
own conversion (`NumberString::int64Value`) uses a fixed internal width
independent of the *current* `NUMERIC DIGITS` (measured: `numeric digits 3;
exit 2147483647` still gives rc 255) -- so `whole_value` is called with
`rexx_num::ARGUMENT_DIGITS` (18), the crate's own already-public constant for
exactly this kind of current-DIGITS-independent conversion, rather than the
activation's `settings.digits()`. Any width from ten digits (what `INT32_MAX`
needs) up would give an identical answer for every case reachable here, since
a value wide enough to need rounding at 18 digits is already wide enough to
fail the final `i32::try_from` regardless of how the rounding landed; 18 was
chosen because it already exists and is already documented for this class of
conversion, not derived from first principles.

Full transcript, independently re-verified end to end through `rexx-run`
(not just the oracle) after implementing, in "Verification" below.

### A dead end, recorded because it wasted real time

First attempt at the `DROP (v)` verbatim-vs-resolved discrimination used
`b = 'xyz'; a.xyz = 'hit'; drop a.b; say a.xyz`, expecting "if `drop a.b`
resolves `B` as a variable it drops `a.xyz`'s own tail and `hit` disappears."
It does not disappear (`say a.xyz` still gives `hit`), which first looked
like evidence that direct `DROP` does *not* resolve tail pieces as variables.
It is not: `a.xyz`'s own tail piece is `XYZ` (a *symbol*, letter-led, hence
`Tail::Variable`, per `compound_parts`), never the literal string `"xyz"` --
Rexx has no syntax for a literal alphabetic tail at all, digit-led pieces are
the only `Tail::Constant` case. `a.xyz = 'hit'` sets the tail keyed by
variable `XYZ`'s value (unset, derives `"XYZ"`), and `drop a.b` (with `b`
unset too) drops the tail keyed by `B`'s value (`"B"`), a different key by
case alone. Two more attempts with deliberately mismatched case landed in the
same trap before the actual test (`b = 'FOO'` still doesn't discriminate,
since `a.foo`'s own tail resolves through variable `FOO`, unset, deriving
`"FOO"` -- the same string `b` happens to hold) -- what finally worked was
comparing `Direct` against `Indirect` on the *same* dotted spelling
(`drop a.b` vs `drop (v)` with `v = 'A.B'`) rather than trying to construct a
single self-contained transcript for `Direct` alone.

## Implementation

`Flow`, `step`, `step_in_temps_frame`, `run_fragment` and `run_activation`
moved verbatim from `lib.rs` into new `run.rs` (including `run_activation`'s
long borrow-shape doc comment and its two doctests, unedited beyond `s/fn
run_activation(&mut self)/pub(crate) fn run_activation(&mut self)/` --
required because `Interp::run` (staying in `lib.rs`, the crate root) is not a
descendant of the new `run` module, so a root-private item is no longer
visible to it the way it was when both lived in the root; nothing else moved
needed a visibility change, since everything else that reaches into them
(`step_in_temps_frame`, `run_fragment`, `Flow` itself) now lives in the same
file). `lib.rs` keeps `Code<'a>` (used by `eval.rs`, `plan.rs`, `stem.rs` too,
none of which this task may edit, so it cannot move without rippling into
files outside this task's scope), `Interp`, `Novalue`, `read`, `execute`,
`run_program`, `Loud`, `form_name`, the thread setup, and picks up one new
method, `exit_code_for`. Crate-level doc comment updated to name `run.rs` and
to stop claiming the borrow discipline's code, rather than its exercise,
lives in `lib.rs`.

`step` gains: `Assignment`'s `Stem`/`Compound` targets (`stem_assign`/
`stem_set`, both already built by Task 5, dispatch only); all of `Drop`
(`drop_variable`, new helper); `Numeric`, all six spellings (`exec_numeric`/
`numeric_operand`, new helpers, plus a local `raised_from_settings` since
`SettingsError`'s `(major, sub)` accessor is private to `rexx-num` and this
task may not add one -- the four pairs are copied from `settings.rs`'s own
doc comments instead); `Exit` now handles `Some(expression)` as well as the
existing `None`; `Label`/`Nop`, both no-ops. `exec_numeric`, `numeric_operand`,
`drop_variable`, `shape_of`/`NameShape` and `raised_from_settings` are new,
private to `run.rs`.

`lib.rs::execute` restructured (not behaviourally changed on the `Loud`/
`Raised` arms) so `exit_code_for` can run on `&mut interp` before `interp.trace`/
`interp.out` are moved out into the `Outcome` -- a partial move of one field
ends `interp`'s usability as a whole value, so the conversion has to happen
first.

`rexx-run.rs`'s exit-code conversion changed from `u8::try_from(outcome.exit_code)
.unwrap_or(u8::MAX)` (saturating) to `outcome.exit_code as u8` (truncating,
matching the oracle's own `value & 0xFF` behaviour -- Rust's numeric `as`
narrowing is a defined two's-complement truncation, not
implementation-specific, so `-1i32 as u8 == 255`, `256i32 as u8 == 0`).

## Testing

Tests written first, in `run.rs`'s own `#[cfg(test)] mod tests`, covering the
brief's own list: assignment to a variable/stem/compound; `SAY` of each value
kind and of an omitted expression (verified it is a blank line, not nothing);
`DROP` of a variable, a tail, a whole stem, and the `(v)` indirect form (three
sub-cases: simple, whole stem, and the joined-dots-verbatim compound case);
`NUMERIC DIGITS`/`FUZZ`/`FORM` including both `VALUE` spellings and the
no-expression reset (plus its 33.1-conflict transcript); `EXIT` with and
without an expression, and a dedicated test for `exit_code_for`'s own
conversion rule covering every measured boundary (`INT32_MAX` exact,
`INT32_MAX + 1` failing, fractional, non-numeric, the negative
rounding-at-creation asymmetry, and that raising `NUMERIC DIGITS` before the
subtraction removes it); a label as a traced no-op; `NOP`. 16 tests, all
green on first run against the implementation as written -- expected, since
every nontrivial rule (`DROP`'s shape classification, the indirect
verbatim-vs-resolved split, the reset-as-if-typed behaviour, the `EXIT`
rounding asymmetry) had already been measured against the oracle by hand
before any test or implementation code was written, so the tests encode
measurements rather than a guess to be corrected.

**Two of the tests were deliberately re-run against a plausible-wrong
implementation, per this project's own practice, since "16/16 green on the
first attempt" is not by itself evidence a test catches anything:**

1. `drop_of_the_indirect_form`, against a mutant that skips
   `.to_ascii_uppercase()` on the indirect name -- fails
   (`left: [49, 10] ("1\n")`, `right: [88, 10] ("X\n")`), confirming the test
   would catch a missing upcasing step.
2. `exit_code_for_converts_the_result_the_way_the_oracle_does`, against a
   mutant calling `whole_value(9)` (the activation's own default DIGITS)
   instead of `rexx_num::ARGUMENT_DIGITS` -- fails on the `INT32_MAX` case
   (`2147483647` rounds to 9 digits and no longer equals itself), confirming
   the test would catch reaching for the wrong precision constant.

Both mutations were reverted before the final run.

**An operational note for whoever reads this next**: mid-session, a
`git checkout -- rust/crates/rexx-exec/src/lib.rs`, run from the wrong
assumption about what state the file was in (intended to revert only a
just-applied mutation, but `lib.rs`'s mutation had already been reverted by
hand and the `git checkout` instead discarded every uncommitted edit this
task had made to `lib.rs` -- the `mod run;` declaration, the doc comment
update, the `Flow`/`run_activation`/`step`/`step_in_temps_frame`/
`run_fragment` removal, `exit_code_for`, and the `execute` restructuring, all
of it). Caught immediately by `git status` showing a clean `lib.rs`, and
every edit above was reapplied from scratch (not from a backup) and re-verified
by rebuilding and re-running the full suite. No destructive `git` command was
run against a committed state, and nothing outside `lib.rs` was touched by the
mistake.

## Verification

`cargo test -p rexx-exec`: **75 unit tests + 11 integration tests
(`tests/spike.rs`, unaffected by the move) + 2 doctests (`run_activation`'s
pair, now living in `run.rs`), all green.** `cargo clippy -p rexx-exec
--all-targets -- -D warnings`: clean. `rustfmt` (run directly on the three
files this task touched, per the dispatch's own instruction not to run the
package-wide `cargo fmt` while other agents are live) then `cargo fmt -p
rexx-exec -- --check`: clean.

Every instruction this task adds, run under both interpreters through
`rexx-run` and `build/bin/rexx` directly (all under
`( ulimit -v 1048576; ... )`), matching on stdout, stderr and rc:

```
x = 5; say x                                              ->  5                        rc 0
a. = 'wd'; a.1 = 'one'; say a.1; say a.2; say a.           ->  one / wd / wd            rc 0
a = 5; drop a; say a                                       ->  A                        rc 0
y = .nil; drop y; say y                                    ->  Y                        rc 0
u.='d'; u.1='one'; drop u.1; say u.1; say u.2              ->  U.1 / d                  rc 0
x.='d'; x.1='one'; drop x.; say x.1; say x.                ->  X.1 / X.                 rc 0
v='x'; x=1; drop (v); say x                                ->  X                        rc 0
v2='A.1.2'; a.1.2='q'; drop (v2); say a.1.2                ->  A.1.2                    rc 0
numeric digits 3; say 1/3; numeric digits; say 1/3         ->  0.333 / 0.333333333      rc 0
numeric digits 5; numeric fuzz 3; say(1.001=1); numeric fuzz; say(1.001=1)
                                                            ->  1 / 0                   rc 0
numeric form engineering; say 1e10+0; numeric form; say 1e10+0;
  numeric form value 'ENGINEERING'; say 1e10+0             ->  10E+9 / 1E+10 / 10E+9    rc 0
numeric digits 20; numeric fuzz 15; numeric digits         ->  Error 33 / 33.1 (byte-for-byte) rc 223
say 'before'; exit 42                                      ->  before                  rc 42
say 'before'; exit                                         ->  before                  rc 0
here: say 'hit'; nop; say 'after'                           ->  hit / after             rc 0
exit 256 / 257 / -1 / 255 / 2147483647 / 2147483648 /
  -2147483647 / 5.9 / 5.0 / 'abc' / 120 / 999999999 / -100000001
                                          ->  0/1/255/255/255/0/0/0/5/0/120/255/255      all rc match
```

Every one of the above matched exactly, including the raised-condition's
three-line stderr report byte for byte (the 33.1 case) and `exit 120` not
being confused with `NOT_IMPLEMENTED_EXIT`.

## Commit

`git add rust/crates/rexx-exec/src/run.rs rust/crates/rexx-exec/src/lib.rs rust/crates/rexx-exec/src/bin/rexx-run.rs`,
committed with `git commit -F`: `43e18462`, "The instruction loop, and the
seven instructions that do not branch". `.superpowers/` itself is
gitignored, so this report is not part of that commit.

---

## Fix round 1, against `task-9-review.md`

Scope this round: `run.rs` only (the review's Critical and both Minors are
all inside it; `lib.rs` and `bin/rexx-run.rs` needed no change).

### Critical: `DROP (v)`'s indirect form is a subsidiary list, not one name

Re-measured all six of the review's rows against the oracle myself before
touching code (all six confirmed), then measured three more questions the
review's own six rows do not answer, needed to get the fix right rather than
plausible:

1. **What exactly separates words?** `'a    b'` (a run of blanks) and
   `'  a  b  '` (leading/trailing blanks too) both give exactly two words.
   A tab (`'09'x`, confirmed by `c2x` that the byte really is `0x09`) also
   separates -- `'a'||'09'x||'b'` drops both `a` and `b`. A line feed
   (`'0a'x`) does **not** -- `'a'||'0a'x||'b'` is one word, `"a\nb"`, which
   then fails the character check and raises 20.928 with the literal
   newline inside the message. So "blank" here is space-or-tab, not
   "any whitespace byte" -- `is_ascii_whitespace()` would have wrongly
   accepted the newline case as a separator, so `split_indirect_words` uses
   an explicit `b' ' | b'\t'` predicate rather than that.
2. **Does an empty or blanks-only value error or no-op?** Measured
   `v=''; drop (v)` and `v='   '; drop (v)`: both run clean, no error --
   zero words is a legal, silent no-op. (A **literal** `drop ('')`, as
   opposed to a variable holding `''`, is a different thing entirely and
   unrelated to this fix: the parenthesised form's grammar requires a
   symbol token immediately inside the parens, so a literal string there is
   a parse-time 20.906, never reaching `step` at all. First measurement of
   this task nearly conflated the two; caught by checking `drop_variable`
   is never even called for that case.)
3. **Does the whole list validate before any of it drops, or does it drop
   words as they pass and stop at the first failure?** `a=1; b=2; v='a 9
   b'; drop (v)` under `SIGNAL ON SYNTAX` recovery leaves **both** `a` and
   `b` at their original values -- `a`, which sits before the bad word `9`,
   is never dropped. So the fix is two-pass: validate and upcase every
   word first (`validate_indirect_word`, fallible), and only once the whole
   list passes, drop each collected name (`drop_by_name`, infallible).
   Interleaving the two (validate-then-immediately-drop, per word) would
   have dropped `a` before reaching `9` and diverged from this.

Landed as: `split_indirect_words` (space/tab splitting, empty words
discarded), `validate_indirect_word` (character-set check first -- this is
what rejects `(w)` and `a-b` alike with 20.928, since neither is special-
cased for recursion, parens are simply not legal symbol characters -- then
the digit-led/dot-led checks, 31.2/31.3, each substituting the word's own
unmodified bytes), and `drop_by_name` (the shared "resolve this exact string
as a variable/stem/tail and drop it" operation, factored out of the old
`Indirect` arm and now also used by `Direct`'s `Simple`/`Stem` cases, which
are the identical operation on an already-scanner-upcased name). `Direct`'s
`Compound` case is untouched -- it still resolves tail pieces as variables
via `tail_key`, which is `drop_by_name`'s one deliberate exclusion, spelled
out in its own doc comment.

New tests, all using **set** targets per the review's own warning (an unset
target's cleared slot and its derived name render identically, so unset
targets can't discriminate "split into N names" from "one verbatim name"):
the two-word blank-separated list, multi-blank and leading/trailing-blank
collapsing, the tab separator, a mixed stem-and-variable list, each of the
three validation errors with their exact substitution text, the
validate-before-drop ordering (checked by inspecting `a`'s slot directly
after the error, since the test's own `run_source` never pops the
activation on failure), and the empty/blanks-only no-op. 4 new tests, 79
total (was 75).

**Mutation check**: reverted `Indirect`'s handling to the pre-fix
"upcase the whole value, treat as one name" shape and re-ran the four new
tests -- three of the four failed (the subsidiary-list test, the
validation test, and the validate-before-drop test); only the empty/
blanks-only no-op test still passed, since that behaviour is unaffected by
whether splitting happens. Restored the fix, re-ran clean.

### Minor 1: `EXIT`'s result sits unrooted from the wrapper's pop to `exit_code_for`

Added a comment at the exact `push_temp` call in `step`'s `EXIT` arm
documenting the window precisely (what pops it, how long it stays unrooted,
why that's benign only in the absence of a collector, and what a future
GC-introducing task needs to do instead -- a root that survives past the
temps-frame pop, not the one-clause `push_temp` every other result gets).
Not re-architected: the review's own wording ("a root before the pop **when
GC work starts**") reads as deferring the actual fix to that later task,
and re-plumbing rooting now, in code the review just called the strongest
on the branch, for a window that is inert until a collector exists, seemed
like the wrong trade against that alternative reading -- flagged to the
team lead in case that reading is wrong.

### Minor 2: validate before classifying, not inside `shape_of`

Addressed as a consequence of the Critical fix rather than as a separate
change: `validate_indirect_word` runs in full, and can fail, before
`drop_by_name` ever calls `shape_of` on the result. `shape_of` itself is
unchanged and still permissive, which is fine now that every caller reaching
it (`Direct`'s already-scanner-valid name, or an already-validated indirect
word) is guaranteed valid by construction before it gets there.

### Minor 3: `run_source`'s test helper bypassed `step_in_temps_frame`

Switched its one call site from `interp.step(&code, instruction)` to
`interp.step_in_temps_frame(&code, instruction)`, and reworded its doc
comment to say why (the wrapper is the chokepoint that heals `eval.rs`'s six
documented temps leaks, and a test helper that skips it is exactly the shape
a future non-test caller could copy). No test assertions changed as a
result -- confirmed by re-running the full suite before and after, both 79
green -- since nothing collects yet, so "popped immediately" and "popped
whenever this test's `Interp` drops" are behaviourally identical today.

### Re-verification

`cargo test -p rexx-exec`: 79 unit (+4 from this round) + 11 integration +
2 doctests, all green. `cargo clippy -p rexx-exec --all-targets -- -D
warnings`: clean. `rustfmt` on `run.rs` directly, then `cargo fmt -p
rexx-exec -- --check`: clean.

All six of the review's own rows, plus the mixed-shape list, the tab
separator, and the empty/blanks-only no-op, re-run end to end through
`rexx-run` against `build/bin/rexx` (`( ulimit -v 1048576; ... )`
throughout): every one byte-identical on stdout, stderr (including the
31.2/31.3/20.928 three-line reports) and rc.

Only `run.rs` changed this round; `git status --short` shows exactly that
one file modified.
