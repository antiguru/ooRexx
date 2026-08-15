# Task 11 report: `builtin/datatype.rs` (DATATYPE, SYMBOL, VALUE, VAR)

## Files touched

* `crates/rexx-exec/src/builtin/datatype.rs` -- new.
* `crates/rexx-exec/src/builtin/mod.rs` -- `mod datatype;` and four `IMPLEMENTED` rows.
* `crates/rexx-exec/src/error.rs` -- `Raised::argument_not_a_symbol` (40.26).
* `crates/rexx-exec/src/lib.rs` -- `Loud::value_selector` (Phase 7).
* `corpus/builtin-status.txt` -- `DATATYPE`, `SYMBOL`, `VALUE`, `VAR` moved `loud` -> `implemented`.

**Discrepancy against the brief's own "Files" line**: it named only `builtin/mod.rs` and
`corpus/builtin-status.txt` as files to modify besides the new one. `error.rs` and `lib.rs`
also needed changes -- the shared block's own Step 2 text says so explicitly ("add a
constructor that passes `Some(\"Phase 7\")`" to `Loud`, in `lib.rs`), and no existing `Raised`
constructor fit 40.26, so `error.rs` needed one too. Noting this because the brief's "Files"
line undercounted rather than because anything was wrong.

## Design decisions the brief left implicit, and what I measured to make them

**Byte-for-byte port of `LanguageParser::scanSymbol`, not the ANSI grammar.** `SYMBOL`,
`VAR`, `VALUE` and `DATATYPE('S'/'V')` all classify a name through one function,
`classify` in `datatype.rs`, ported from `parser/Scanner.cpp:1792` rather than reasoned
from documentation. The one subtlety worth flagging: the C++ reads `*(scan - 1)` when
`scan == 0` (undefined behaviour, since a sign is the very first byte) -- this crate
treats that case as `Bad` outright, which is what every leading-sign probe below measures
as the right answer (`+1E1` and `-1E1` are both `BAD`).

**`VariableDictionary::getVariableRetriever`'s per-classification behaviour**, from
`execution/VariableDictionary.cpp:738`-`994`, not from probing alone:
* `Bad` -> no retriever, `SYMBOL` is `BAD`, `VAR` is `0`, `VALUE` is 40.26 unconditionally.
* `Numeric`/`Literal`/`LiteralDot` -> a *string* retriever (`isString`), so `SYMBOL`/`VAR`
  never even reach an `exists()` check -- unconditionally `LIT`/`0`. This is true **even for
  a leading-dot `Literal`** (`.foo`), because that path builds a `RexxDotVariable` retriever
  instead, and `RexxDotVariable` does not override `exists()` -- the inherited default,
  `execution/ExpressionBaseVariable.hpp:61`, is unconditionally `false`. So a dot name
  reaches `LIT`/`0` by a different route than a plain literal, and both routes were checked
  against the oracle (see the `VAR('.LOCAL')` row below).
* `Name`/`Stem`/`CompoundName` -> a real retriever; `exists()` is the only question `SYMBOL`/
  `VAR` ask.

**`VALUE`'s read for an undefined leading-dot name.** `RexxDotVariable::getValue`
(`expression/ExpressionDotVariable.cpp:195`) tries a package-environment lookup, then a
small "special rexx name" table, and falls back to `"." + name` (via `concatToCstring`,
which *prepends*, not appends -- confirmed by usage elsewhere in the C++, e.g.
`ObjectClass.cpp:1783`'s `"an " + name`). This crate has no environment/`.local` subsystem
at all (`eval.rs`'s own module doc: only `.NIL`/`.TRUE`/`.FALSE` are admissible dot-variable
*expressions*, everything else loud), so `literal_value` reproduces exactly those three
specials and falls back to the upcased text otherwise. **This is a documented, bounded gap,
not a measured-complete implementation**: it is correct for every undefined dot name (which
is every case `VALUE.testGroup` exercises -- `.zl`/`.zu`/the `'.'||'B'` case are all built by
concatenation from undefined names, never a `.local~x`-defined one passed to `VALUE`
directly) and would be silently wrong for a `.name` this crate's `.local`/environment gap
already covers nowhere else. **Flagging as inferred rather than fully measured**: I did not
attempt to find or probe a real `.local~x=val; value('.x')` case, because doing so
faithfully needs the environment subsystem `eval.rs` itself declares out of scope.

**Compound-tail resolution is the *substituting* form (`buildCompoundVariable(name, false)`,
`VariableDictionary.cpp:923`), not the *direct/literal* form `DROP (v)`'s indirect syntax
uses.** `assign_by_name`/`drop_by_name` in `run.rs` were the wrong tool for this reason and
I did not reuse them; `resolve_compound_key` in `datatype.rs` is a new, small piece of logic
mirroring `tail_key`'s shape but starting from a runtime string with no `SymbolId`.

**Existence checks reuse `Interp::slot_of`'s growing resolver even for reads**, per the
brief's own direction, generalised: growing a slot without writing a value is unobservable
(a freshly grown slot reads as `None` either way), so `slot_has_value` uses it for `SYMBOL`/
`VAR`'s simple-variable *and* bare-stem checks -- the same shape, because
`RexxActivation::localVariableExists` and `localStemVariableExists`
(`execution/RexxActivation.hpp:515`,`:507`) turn out to be the identical test once this
crate's own `read_stem`/`stem_set`/`stem_assign`/`stem_drop` all leave a `Body::Stem` behind
regardless of whether it was dropped. A compound tail's existence reuses `Interp::stem_get`'s
already-tombstone-aware `Novalue` answer instead of re-deriving it.

## Probe tables (measured against the oracle, all from a fresh scratchpad subdirectory, wrapped in `ulimit -v 1048576`)

**DATATYPE, one-argument and option-letter forms** -- alphabet: printable ASCII operands,
one binary/hex-boundary case each, one 18/19/20-digit boundary each for `I`, one
9/10-digit boundary for `9`. 22 cases, 0 mismatches:

```
datatype('12.5') datatype('abc')                      -> NUM CHAR
datatype(123,'n') datatype(123,'NUM') datatype(123,'NX') -> 1 1 1
datatype('','B') datatype('abc','L') datatype('ABC','U') datatype('AbC','M') -> 1 1 1 1
datatype('abc','A') datatype('ab 3','A')               -> 1 0
datatype('0','O') datatype('2','O')                    -> 1 0
datatype('a.b','V') datatype('1abc','V')               -> 1 0
datatype('a.b','S') datatype('*','S')                  -> 1 0
datatype('FF','X') datatype('FG','X')                  -> 1 0
datatype('1010','B') datatype('1012','B')              -> 1 0
datatype(99999999999999999999,'I') datatype(123456789012345678,'I') -> 0 1
datatype(1000000000,'9') datatype(999999999,'9')       -> 0 1
datatype('80'x,'W') datatype('ff'x,'W')                -> 0 0
```

**DATATYPE, omitted vs. empty option** -- 3 cases, 0 mismatches:

```
say datatype('')          -> CHAR                     (rc 0)
say datatype('','')       -> Error 93.915:  Method option must be one of "ABILMNOSUVWX9"; found "?".  (rc 163)
say datatype(1,'Z')       -> Error 93.915:  Method option must be one of "ABILMNOSUVWX9"; found "Z".  (rc 163)
```

**High-byte sweep** -- alphabet: every byte from `0x80` to `0xFF` (128 values, not 256 --
corrected in Fix round 1 below) crossed with options `A`, `U`, `L`, `M` in the crate's own
test (`no_byte_above_0x7f_is_alphanumeric_upper_lower_or_mixed`), 128 * 4 = 512 cases, plus 2
more for `'W'`'s two representative rows (`no_byte_above_0x7f_is_a_whole_number`) = 514 total,
0 mismatches against the brief's own Step-1 measurement (every byte `0x80..=0xFF` is `0` for
all five). Not independently re-run against the live oracle byte-by-byte over the full range
(that measurement is the brief's own, attributed to it in the code comment); the two
representative `'80'x`/`'ff'x` `'W'` rows above were run against the oracle directly.

**SYMBOL/VAR, variable-pool-sensitive cases** -- 12 cases, 0 mismatches:

```
drop a.3; j=3
symbol('J') symbol(J) symbol('a.j') symbol(2) symbol('*')  -> VAR LIT LIT LIT BAD
symbol('.') symbol('.a') symbol('')                        -> LIT LIT BAD
var('J') var(J) var('a.j') var(2) var('*') var('.LOCAL')    -> 1 0 0 0 0 0
```

**The stem-default rule (brief Step 0c)** -- 2 cases, 0 mismatches:

```
s.='dflt'; symbol('s.9') var('s.9')     -> VAR 1
(no default) symbol('t.9') var('t.9')   -> LIT 0
```

**VALUE, 40.26 and the presence-vs-content split (brief Step 0a)** -- 6 cases, 0 mismatches:

```
value('*')                    -> Error 40.26:  VALUE argument 1 must be a valid symbol; found "*".   (rc 216)
value('5','x')                -> Error 40.26:  VALUE argument 1 must be a valid symbol; found "5".   (rc 216)
myvar='ORIGINAL'; value('myvar',)      -> ORIGINAL           (trailing omission drops to 1 arg, stays local)
drop a.3; j=3; value('a.j')            -> A.3
a.3='hit'; value('a.j')                -> hit
myvar='ORIGINAL'; value('myvar') value('myvar','NEWVAL') value('MYVAR') value('nosuchvar')
                                        -> ORIGINAL / ORIGINAL / NEWVAL / NOSUCHVAR
```

**VALUE, undefined dot-literal fallback** -- 3 cases, 0 mismatches:

```
c='.'; b='B'; value(c||b)              -> .B
value('.nil')=.nil                     -> 1
value('.true') value('.false')         -> 1 0
```

The 3-argument (selector-present) form itself was **not** run against the oracle for a real
answer, by design: it is a declared Phase 7 gap here, so the only oracle fact needed is that
the *shape* (a present third argument, any content) is what the brief's Step 0a already
measured (`value('myvar',,'')` -> `.MYVAR`, `value('zz',,'NOSUCHPOOL')` -> 40.914) -- I did
not re-measure those two exact transcripts myself; they are carried over from the brief
verbatim and are the reason the third argument's mere presence, not `NOSUCHPOOL`'s validity,
is the trigger for the loud path.

## Divergences found

None, against every probe above. The implementation matches the oracle on every measured
case.

## What was inferred rather than measured

* The undefined-dot-literal fallback's correctness for a *defined* `.local~x` entry --
  explicitly flagged above, not measured, because measuring it would require building the
  environment subsystem this crate declares out of scope elsewhere.
* The byte range `0x80..=0xFF` for options `A`/`U`/`L`/`M`/`W` is the brief's own prior
  measurement (Step 1), not independently re-run against the live oracle here beyond the two
  `'W'` spot checks above.

## Mutation check

Mutated `value`'s selector-presence test from `arg(args, 3).is_some()` to
`optional_string(interp, args, 3).is_some_and(|s| !s.is_empty())` (treating an empty selector
as absent, the exact wrong-answer shape the brief warns about). Ran
`cargo test --offline -p rexx-exec --lib builtin::datatype` (16 tests) with the mutation in
place: **1 failed** (`values_third_argument_is_decided_by_presence_not_content`), 15 passed.
Restored the correct code from a backup copy (not `git checkout --`) and re-ran: 16 passed,
0 failed. This is the one test in the suite built specifically to distinguish "presence
decides" from "non-empty content decides"; every other test in the file is agnostic to this
mutation since none of them pass an empty-but-present selector elsewhere.

## Verification output (from `rust/`, clean `target/` before clippy per CLAUDE.md)

```
$ cargo fmt --all --check
(no output, exit 0)

$ rm -rf target && cargo clippy --offline --workspace --all-targets -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.22s
(exit 0)

$ cargo test --offline --workspace --no-fail-fast
... 73 `Running`/`Doc-tests` process headers (grep -c "^     Running\|^   Doc-tests"), matching
    the documented baseline exactly ...
0 lines containing FAILED, 0 lines containing "error["
(exit 0, confirmed via a separate unpiped run redirected to a log file)

$ REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus
49 of 49 matching
test result: ok. 9 passed; 0 failed; 1 ignored
(exit 0)

$ cargo test --offline -p rexx-exec --test builtin_status
test result: ok. 13 passed; 0 failed
(exit 0; DATATYPE/SYMBOL/VALUE/VAR all measured `implemented` against the live oracle,
matching the committed corpus/builtin-status.txt rows)
```

## Commit

`d6e8a8c75e4e5b8e848c6638275e33d0aac60c75` -- "Implement DATATYPE, SYMBOL, VALUE and VAR
(4c Task 11)", read back with `git log -1`.

# Fix round 1

Three Criticals, three Importants, four Minors. Two of the three Criticals were pre-verified
by the reviewer against the oracle; the third (VALUE on a defined environment name) carried a
binding user ruling: declare the gap, do not implement it.

## C1 -- 40.26 substituted the upcased text, not the caller's own spelling

**Before:** `Raised::argument_not_a_symbol(name, 1, &upper)` at both call sites in `value`
(`builtin/datatype.rs`); `error.rs`'s contract paragraph asserted the insert was "already
upcased the way the caller upcases before classifying it", citing
`VariableDictionary::getVariableRetriever`. Measured, `value('ab*')` reported `found "AB*"`.

**Why the premise was false:** `getVariableRetriever(RexxString *variable)` takes the pointer
by value and reassigns its own local (`variable = variable->upper();`) -- that reassignment
never reaches back into `BUILTIN(VALUE)`'s own `variable`, which is what
`expression/BuiltinFunctions.cpp:1840`'s `reportException(..., variable)` actually passes.

**After:** both call sites pass `&text` (the pre-upcase argument); `error.rs`'s doc rewritten
to state the true rule and the aliasing mistake, with the `value('ab*')` transcript. Added
`value_40_26_substitutes_the_arguments_own_bytes_case_and_all`, five witnesses measured
against the oracle directly (alphabet: a bare-case BAD input, a lowercase Numeric-with-newvalue
input, a period-bearing BAD input, one raw byte `0x80` that must not round-trip through
`from_utf8_lossy`, and the `VALUE.testGroup:88`/`test008` transcript with its `.zl` indirection
resolved):

```
value('ab*')                found "ab*"
value('1e1','x')             found "1e1"
value('abc.def*','x')        found "abc.def*"
value('a'||'80'x)            found "a" + raw byte 0x80 (measured via xxd, not cat -v)
value('lowercase garbage')   found "lowercase garbage"
```

All five measured against the oracle before writing the test (see the transcripts above);
0 mismatches.

## C2 -- an unrooted read-before-write in VALUE's compound write

**Before:** `value`'s `SymbolKind::CompoundName` arm called `stem_get` then `stem_set` with
nothing rooting the value `stem_get` returned in between. Reproduced with
`run_program_collect_every_alloc` on `"j=3\nsay value('a.j','new')\n"`: panics at
`value.rs`'s `a live value`.

**After:** `interp.roots.push_temp(old);` immediately after `stem_get`, before the
`stem_set` call, following `eval_arithmetic`'s `left_value` convention (`eval.rs:610`). Added
`values_compound_write_roots_the_old_value_before_the_stems_first_allocation` to
`tests/collect_stress.rs` (the L0 subset that file already reads does not contain this shape),
with three adjacent-success control rows plus the failing shape itself, all under
`run_program_collect_every_alloc`.

**Mutation, both directions:**

* With the `push_temp` line removed: `cargo test --offline -p rexx-exec --test collect_stress
  --no-fail-fast` -> 3 passed, 1 failed (the new test), panicking at `a live value` exactly as
  the plain (uncollected) run does not.
* Workspace-wide with the same mutation, `--no-fail-fast`: **73 process headers**, exactly
  one failure (`values_compound_write_roots_the_old_value_before_the_stems_first_allocation`),
  1209 passed / 1 failed (1210 total, matching the clean baseline's total test count).
* Restored from a checksummed backup (`md5sum`-verified before and after, not `git checkout
  --`) and re-ran: 73 headers, 0 failed, 1210 passed.

## C3 -- VALUE on a defined environment name (ruled: declare, do not implement)

**Before:** no record of the gap; `datatype.rs`'s `literal_value` doc claimed the undefined-dot
fallback was "silently wrong only for a `.name` this crate cannot represent regardless",
reading as a fringe case.

**Measured** (both directions, by the reviewer, cited here rather than re-measured): `say
value('.LOCAL')` is oracle rc 0 `The Local Directory` against this crate's rc 0 `.LOCAL`; `say
value('.ARRAY')` is oracle rc 0 `The Array class` against `.ARRAY`; `say .LOCAL` is already
loud here at rc 120, so the same name is loud on one path and silently wrong on the other.

**After:** added a KNOWN GAP row to `docs/superpowers/plans/phase-4-exclusions.txt` naming the
transcripts, the ruling, and why `builtin-status.txt`'s `VALUE` row correctly stays
`implemented` (the same reasoning the file's own TRANSLATE row gives -- the probe
`zz=41; say value('zz')` never reaches a dot-prefixed argument). Corrected `literal_value`'s
doc comment: the affected set is the whole absent Phase 5 environment/`.local` subsystem
(`.local`, `.environment`, every class name, every stream alias), not one fringe name --
without enumerating that set in this crate's own repository, per the ruling.

## Important fixes

**I1 -- byte-count arithmetic.** The high-byte sweep covers `0x80..=0xFF`, 128 values, not
256. Fixed the code comment ("Swept over every byte from `0x80` to `0xFF`", no restated
count) and this report's own probe table above: **514 cases** (128 * 4 for `A`/`U`/`L`/`M`,
plus 2 for `W`'s two representative rows), not 1026. Grepped the tree for `256 \* 4`/`1026
cases`/`all 256 byte` outside this report and the code comment: no other occurrence of this
specific claim.

**I2 -- mutation check re-run workspace-wide.** The original mutation check (VALUE's
presence-vs-content discriminator, `arg(args, 3).is_some()` -> `optional_string(...).
is_some_and(|s| !s.is_empty())`) was re-run properly this round:

* Confirmed the mutation's own diff before running it (`diff` against a checksummed backup):
  exactly the one line changed, not silently reformatted to a no-op.
* `cargo test --offline --workspace --no-fail-fast`, mutated: **73 process headers**,
  exactly one failure (`values_third_argument_is_decided_by_presence_not_content`), 1209
  passed / 1 failed.
* Restored from the checksummed backup, `diff`-confirmed byte-identical, re-ran: 73 headers,
  0 failed, 1210 passed.

Comparing against **1210** (this round's clean baseline: the coordinator's stated 1208 plus
the two tests this round adds, `value_40_26_substitutes_the_arguments_own_bytes_case_and_all`
and `values_compound_write_roots_the_old_value_before_the_stems_first_allocation`), not
against 1208, since 1208 predates both new tests.

**I3 -- `Loud::value_selector`'s owner.** Dropped the `Some("Phase 7")` owner; the constructor
now follows `Loud::builtin_option_object`'s no-owner convention (a sub-case inside a builtin
`builtin-status.txt` calls `implemented`). Read `BuiltinFunctions.cpp:1848`-`1913` directly
rather than trust the prior analogy to `ADDRESS`'s command layer: the selector's value picks
one of three destinations -- an empty selector reads/writes `.environment` (Phase 5's, the
same subsystem C3's KNOWN GAP row names), the literal `'ENVIRONMENT'` reads/writes the OS
environment (neither phase, cited but not attributed), and anything else tries a platform
selector and the value exit (Phase 7's, the only one of the three the prior analogy was
actually true of). Recorded the per-path attribution in `phase-4-exclusions.txt` rather than
in the doc comment. Updated the one test asserting the message text to drop `" (Phase 7)"`.

## Minor fixes

* **M1:** `symbol_reads_the_variable_pool_for_names_and_compounds` was missing
  `symbol('  ')` (two blanks), the fourth of `SYMBOL.testGroup`'s "new tests"; the doc
  comment claimed "four" while the test asserted three. Measured against the oracle
  (`BAD`), added the assertion, corrected the comment to name all nine of `test_SYMBOL`'s
  assertions (five documented examples plus its four "new tests").
* **M2:** `classify_matches_every_symbol_testgroup_numbered_case`'s doc claimed "28 numbered
  cases" above a 26-row table. `test007`/`test015` and `test019`/`test026` each repeat the
  other's input (`'1E1.'` and `'1E1 '` respectively). Corrected to "26 distinct inputs across
  `test001`-`test028`".
* **M3:** `a_stems_default_value_makes_every_tail_report_var`'s doc misnamed `s.9`'s own
  classification as `STRING_STEM` (it is `SymbolKind::CompoundName`: one period, not
  trailing) and misspelled `StemClass` as `STemClass`. Corrected both; the cited mechanism
  (`StemClass.hpp:139`) was already right.
* **M4:** removed two task/brief-provenance clauses forbidden by `rust/CLAUDE.md`'s comment
  rules -- `is_symbol_byte`'s doc ("measured at Task 5's sibling builtins and confirmed
  here,") and `slot_has_value`'s doc ("and the reason 4c's brief names it rather than a
  non-growing lookup"). Both sentences survive intact with the clause deleted. Grepped the
  file for `Task [0-9]`/`this task's`: two remaining hits (`datatype.rs`'s own Step-0(b) and
  Step-1 citations) were not among the two the review named and were left as measurement
  attributions, not task numbers.

## A defect introduced and caught during this fix round itself

While redoing I2's mutation check, a `cp` restore used a backup (`datatype.rs.bak2`) taken
*before* the C3/I1/I3/M1-M4 edits landed, silently reverting all of them back to their
pre-fix state without any error -- exactly the failure mode `rust/CLAUDE.md`'s "restore from
a copy rather than git" rule exists to avoid, except the copy itself was stale. Caught by
re-grepping for every fix's own marker text immediately afterward (all nine markers checked
in one pass) rather than trusting the earlier "restored, 4 passed" result, which had run
before the fresher edits existed and so could not have detected their absence. Every
mutation check from that point on used one `md5sum`-verified backup, taken after confirming
all edits present, with a `diff` taken immediately before and immediately after each
restore.

## Final verification (from `rust/`)

```
$ cargo fmt --all --check
(no output, exit 0)

$ rm -rf target && cargo clippy --workspace --all-targets -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.88s
(exit 0)

$ REXX_CORPUS_GATE=1 cargo test --offline --workspace --no-fail-fast
73 process headers (grep -c "^     Running\|^   Doc-tests")
0 lines containing FAILED
1210 passed / 0 failed (summed across every `test result:` line)
49 of 49 matching (the corpus differential gate)
(exit 0, confirmed unpiped)

$ cargo test --offline -p rexx-exec --test builtin_status
test result: ok. 13 passed; 0 failed
(exit 0; DATATYPE/SYMBOL/VALUE/VAR still measured `implemented`)
```

## Commit

`9c3d96df99050fad60430db730c0139075e7ce25` -- "Fix round 1 for Task 11: 40.26 case bug, an
unrooted VALUE write, and the environment gap", read back with `git log -1`.
