# Phase 6.1 Task 11a review

Reviewed `fbdf615e8..f8d7049e8`, with ledger commits skipped. Paths are under `rust/crates/rexx-exec/src/`
unless they say otherwise. I built HEAD from `git archive f8d7049e8` into `/tmp/claude-1000/p61/t11ar/src`
with its own target dir, and the build log shows one `Compiling rexx-exec` line. Crate runs used
`memcap 2G timeout -k 5 20`, and oracle runs used the global wrapper. Each run started in a fresh
`mktemp -d`, through `t11ar/tools/both.sh BIN FILE N`. "N/N" means identical rc, stdout md5 and stderr md5
across runs of both engines. The probes are still under `/tmp/claude-1000/p61/t11ar/probes/`. The source
copy, target dir and run dirs were deleted after the review.

## Verdicts

* **Spec compliance: not compliant.** Steps 0, 1, 2, 3, 5, 7 and 8 hold. Step 6 holds only for sources
  with no directives. Its directive path resolves `::CLASS ... SUBCLASS` through the running program
  (I1), which the ruling's stop clause covered ("a stop applies if it reaches package installation for
  directive programs"). The narrowed R11 split puts consumers that have a defined oracle answer in the
  refusal set (I3, I4).
* **Quality: needs fixes.** One arm of the Step 6b parent walk has no test that can fail (I2). Every
  other new test I mutated went red. No em-dashes were found in the added lines. Byte accounting is
  untouched.

Counts: Critical 0, Important 4, Minor 2.

## Important

### I1. A `::CLASS` directive in a `Routine~new`/`Method~new` source resolves its superclass through the running program, not the new package's parent

`environment.rs:605` `directive_class` (called from `install.rs:1407`) uses `self.installed_class(upper)`.
That lookup is keyed on `running_program()`, which is the caller, not `installing`, and it never
walks `installing`'s parent chain. The oracle resolves it with `ClassResolver::lookup(package)`, which
uses the installing package's `findClass` (`ClassDirective.cpp:165-195`). Before this task the context
form refused with "NEW of class Routine ... (Phase 9)". It now gives a wrong answer in both directions:
it hides a class the oracle finds, and it finds one the oracle hides.

Probe `a4/routsub.rex`, the main file holding a non-public `::class hidden`, with
`p1 = .package~new('p1', .array~of('::class other', ...))`:

```
say .routine~new('r', .array~of('return .kid~new~name', '::class kid subclass other'), p1)~call
say .routine~new('r', .array~of('return .kid~new~name', '::class kid subclass hidden'), p1)~call
```

`both.sh ... routsub.rex 3`: crate 3/3 `routine subclass other ctx: error 98.909` /
`routine subclass hidden via p1: hidden`. Oracle 3/3 `other` / `error 98.909`. The base binary
`hsr/bin/fr1` gives rc 120 with the Phase 9 refusal on that line.

The same mechanism predates the task, and both of these reproduce at base too. In `a6/pkgsub.rex`,
`.package~new('p3', '::class k subclass hid2', p1)` gives crate `98.909` where the oracle answers
`hid2`. In `a8/main.rex`, `call 'lib2.rex'` with `::class k subclass hiddenx` and `hiddenx` non-public
in the caller gives crate `loaded` where the oracle gives `error 98.909`.

**Fix:** make `directive_class` take `installing`'s view: its own `package_classes`, then
`parent_installed_class(installing, ..)`, then its `merged_public_classes`, then
`parent_public_class(installing, ..)`, then the REXX package. This is `findClass` on the installing
package. Witness `routsub.rex`'s two context lines with a corpus program. The `Package~new` and
external-call variants should go to the controller as pre-existing. The same fix probably closes them,
but they need their own witnesses.

### I2. The merged-public arm of the parent walk (`imported_class`) has no test that can fail

`environment.rs:711`, mutant M2: `if self.has_package_parent(program)` becomes
`if false && self.has_package_parent(program)` inside `imported_class`.
`cargo test -p rexx-exec --lib a_compiled_source_resolves_through_its_package_parent` gives `1 passed`.
On a release build of the mutant, `routine_new_package_parent.rex` and
`package_find_through_parent.rex` keep their HEAD stdout (`788f71ae`, `f21bdd9e`). The arm only
matters for a public class that the parent got from its own `::REQUIRES`, because the
installed-class walk already finds the parent's installed public classes. Probe `a2/req.rex` (main
`::requires 'lib.rex'`, which declares `::class rpub public`) runs
`.routine~new('r', 'return .rpub~new~name')~call`. HEAD matches the oracle 3/3 (`reqpub: rpub`), and
the mutant raises 97.1 at `return .rpub~new~name`.

**Fix:** add the required-public case to a corpus witness or to the crate test, and confirm M2 turns
it red.

### I3. The R11 split refuses consumers whose oracle answer is defined, and the report's "every other site reads bytes" is false

The ruling says object consumers give the oracle's defined answer. I enumerated the consumers with
the report's own grep, which finds 34 sites, and ran one probe per consumer. Each probe was
`o = .sn~new` plus one statement, with `::method string` answering `.directory~new`. I ran each 2
times on the oracle. These answers were stable and are not reads of `.nil` through the string
layout. The crate refuses each one with rc 120.

| consumer (site) | oracle, 2/2 |
|---|---|
| `signal value o` (`run/condition.rs:1088`) | rc 240, 16.1 `Label "The NIL object" not found.` |
| `x.o = 1; say x.o` (`stem.rs:146`) | rc 0, `1`; `x.['The NIL object']` is `1`, so the tail is "The NIL object" |
| `say 1 + o` (`eval.rs:785`) | rc 215, 41.1 `Nonnumeric value ("a SN")` |
| `numeric digits o` (`run/settings.rs:386`) | rc 230, 26.5 `found "a SN"` |
| `do o; end` (`run/loops.rs:685`) | rc 230, 26.2 `found "a SN"` |
| `o` as a command (`command.rs:936`) | rc 0, runs `The NIL object` (`RC(127)`) |
| `interpret o` (`run.rs:622`) | rc 0, interprets `The NIL object` |

The other consumers I probed give garbage on the oracle or already match the crate. Error 5 comes
from `||` on either side, `value()`, `upper()` and `"abc"~"||"(o)`. `drop (v)` gives SIGSEGV rc 139,
and `trace value o` gives 24.1 found `"?"`. `address value o` gives 29.1, which is a garbage length
check. Both engines answer `say`, `queue`, `lineout` and `charout` with 88.909, and the crate refuses
those as recorded. No bytes consumer answers silently in the crate. Every case where the crate
answered matched the oracle 1/1 (`b17`, `b24`, `b25`, `b30`, `b37`, `b38`, and the native-argument
88.909/93.x rows `b18`-`b40`).

**Fix:** either put these sites in the `.nil` set, keeping the original object where the oracle
reports it ("a SN" in 41.1, 26.5 and 26.2), with a witness per row, or get a controller ruling to keep
them refused. In either case, change Deviation 30 and the report so they do not say every non-object
consumer reads garbage.

### I4. Under `SIGNAL ON NOSTRING`, a bytes consumer refuses where the oracle signals first

`dispatch/reqstr.rs:208` returns `Loud::string_answer_not_a_string` before the NOSTRING check at
`:228`. The oracle raises NOSTRING straight after `primitiveMakeString` gives `.nil`
(`ObjectClass.cpp:1259-1287`), so a trapped signal leaves before any byte is read. Probe
`b3/ns_sigbytes.rex` is `signal on nostring name h; say length(o)` with
`h: say 'h' condition('C') '['condition('D')']' sigl`. The oracle gives 2/2 rc 0
`h NOSTRING [The NIL object] 3`. The crate gives 2/2 rc 120 with the refusal. The truth-context
version (`ns_sig.rex`) matches 2/2.

**Fix:** in the `None` arm with `!nil`, check whether NOSTRING raises as SYNTAX or a signal first, and
raise it with `The NIL object` as the description. Refuse only if it does not. Add a witness.

## Minor

### m1. `run_repeating`'s clock store is unreached (the implementer's concern)

`run/loops.rs:1479` was confirmed unreached by probes, per the report. I added labelled empty-body
loops (`d/labelled.rex`: labelled WHILE, UNTIL, controlled WHILE, FOREVER with LEAVE, INTERPRET, and
inside SELECT LABEL). They match the oracle 2/2 and do not show which driver ran. **Fix:** a crate
test that drives `run_repeating` directly with an empty body, or a ruling that keeps the store
untested to match Task 2's ruling on the same function.

### m2. The Step 7 copy counter only sees copies made through `array_slots_owned` and `occupied`

`dispatch/array.rs` `note_slots_copied`. Mutant M11, which puts an `array_slots_owned` copy back in
`append_slot` (`collection.rs:268`), turns `appends_and_reads_copy_slots_linearly` red, as it
should. A `.to_vec()` written at a new site would not be counted. The scaling tables in the report
are the evidence that holds. **Fix:** none needed for this task. Note the counter's scope if the test
is ever cited as the linearity proof.

## Checked and holding

* **(a) Parent walk order.** `installed_class` walks every package's installed classes, public or
  not, up the chain before any merged-public step. `imported_class` checks its own merged classes,
  then each parent's installed-public and merged classes, then the REXX package. This is the order of
  `PackageClass.cpp:982-1049`, and the parent's package local is not searched. Probes, 3/3 each,
  matching the oracle:
  * `a1/shadow.rex`: an own class shadows the parent's, the grandparent chain is walked, the parent's
    class shadows a user class, and `findClass` and `findPublicClass` answer on non-public and public
    classes.
  * `a2/req.rex`: the parent's required public class is found, its required non-public class is not,
    the parent's installed class wins over the parent's required public one, and the source's own
    class wins.
  * `a3/pkgctx.rex`: `.NAME` resolves through a `Package` context, and a class the running program
    holds is not found through it. The `::CLASS` subclass lines in this file are I1.
  * `a5/local.rex`: the parent's package local is not visible, and `.local` is.
  * `a7/sysname.rex`: a parent `::class directory` shadows the REXX package's class.
* **(b)** See I3 and I4. The three object sites (`eval.rs:1324` for `& | &&` only, `eval.rs:1479`,
  `dispatch/library.rs:1503`) match the oracle.
* **(c)** Steps 7 and 8 add no heap writes. The diff adds no `get_mut` on a heap body. Step 7 only
  turns reads into borrows, growth still goes through `array_grow`, and the buffer change only
  changes signatures. `copies_bytes` writes into a buffer sized once to `total`, and
  `extend_from_within` never reallocates it. The heapshape rule is satisfied by inspection, because
  no new path changes how many bytes are held.
* **(d)** Every RESOLVED item's own probe matches the oracle. These ran 3 runs per engine:
  `se1/probes/elapsed*.rex` (8 probes), `hexit`, `pend4`, `sendhalt`, `se2/probes/i9a`, `i9b`, `i9c`,
  `append_1000`, `unwind_2000`, `unwindexit_2000`, `unwindnotrap_2000`, `listappend_10000` and
  `copies2`. Every new corpus witness ran 5/5 per engine (`time_elapsed_empty_loop`, both
  `message_halt_*`, the three `reply_handler_exit*`, `routine_new_context`,
  `routine_new_package_parent`, `package_find_through_parent`, `string_answer_array`). `i7b`, `i7e`,
  `i7f` and `i7h` are Deviation 30 refusals, and on `i7e` the oracle gave three different stdouts in
  three runs. I also checked a sender activity reused after it ended (`d/reuse.rex`: `m~send` on
  started activity A, A ends, B starts, main sends `m~halt`). Both engines give 5/5 `halt: 1` /
  `b: b halted`, because the oracle pools activities too.
* **(e)** Mutants with the fix reverted in the scratch copy, each restored from a file copy. Every
  run printed `1 passed` or `FAILED. 0 passed; 1 failed`:
  * M1, `installed_class` walk off: red.
  * M3, `installed_class_of` walk off: red.
  * M4, `package_public_class_of` walk off: red.
  * M5, `required_string_or_nil` refuses: red.
  * M6, the logical operators refuse: red.
  * M7, the buffer arm dropped: red.
  * M8, the Queue check dropped: red.
  * M9, the hash kind check off: red.
  * M10, `prune_senders` a no-op: red.
  * M11, the append copy put back: red.
  * M2 stays green (I2).

## Pre-existing, outside the task (for the controller's queue)

* `call on nostring name h` gives 25.1 with `found "&1"` on both base `fr1` and HEAD. The oracle says
  `found "NOSTRING"`. The insert is not substituted.
* The `Package~new(..., ctx)` and external-call `::CLASS ... SUBCLASS` variants from I1.

## Re-review 1

Scope: fix round 1, `f8d7049e8..559c78653`. I skipped the ledger commits and the controller's
`857a01931`. HEAD was built from `git archive 559c78653 rust interpreter` into
`/tmp/claude-1000/p61/t11ar/src`, with its own target dir and one `Compiling rexx-exec` line. Runs
went through `t11ar/tools/both.sh` as before. "At 849f3dfb0" means my review-round results at
`f8d7049e8`, which has the same code. New probes are in `/tmp/claude-1000/p61/t11ar/probes/r1/`.

### Verdicts

* **Spec compliance: not compliant.** I1, I2 and I4 are fixed. I3 is fixed for its seven ruled
  sites, but the widening put two consumers in the `.nil` set where the oracle reads `.nil`
  through the string layout: the ordering comparisons (N1) and PARSE VALUE/ARG (N2). R11 narrowed
  says those sites refuse.
* **Quality: needs fixes.** N1 and N2 turn loud refusals into silent wrong answers. The witness
  rows that cover them pass only for the inputs they happen to use.

Open: 2 Critical (N1, N2), 0 Important, 4 Minor (m1 and m2 recorded by the controller, plus new N3
and N4).

### Status of the review findings

* **I1: fixed.** `directive_class` resolves through `installing`. I ran 3 runs per engine, and all
  three routes now match. At `f8d7049e8` the crate's answers were the reverse of the oracle's, or
  `loaded` for the external CALL.
  * `routsub` (Routine~new) gives `2943d9b7`.
  * `pkgsub` (Package~new with a context) gives `f636abb1`.
  * `a8/main` (external CALL) gives `error 98.909`.
  * The corpus witnesses `routine_new_directive_subclass` and `package_new_directive_subclass`
    match 3/3. `external_directive_subclass` is the `a8` shape, which I ran directly.
  * `shadow`, `req` and `pkgctx` still match 3/3.
* **I2: fixed.** `req.rex` matches 3/3. The implementer showed the M2 mutant turning
  `routine_new_parent_required.rex` red in the corpus gate. I did not re-run that mutation.
* **I3: the seven ruled sites are fixed.** `string_answer_nil_consumers.rex` matches 3/3 with
  stdout `9f736739` and stderr `58a6570b`, covering SIGNAL VALUE, the tail, `+`, `**`, DIGITS, FUZZ,
  DO count, DO FOR, FORM, the timeout, OPTIONS, the equality rows, PARSE VALUE and ARG
  (single-variable only), the command and INTERPRET. All 39 earlier `b/` probes were re-run, with
  `drop` skipped because it crashes the oracle. Every one either matches or is an expected rc 120
  refusal. In those refusals the oracle gives Error 5 (`||`, `value()`, `upper()`, `"abc"~"||"`),
  88.909 (say, queue, lineout, charout), 24.1 found `"?"` (trace) or 29.1 (address). The widening
  is assessed in N1 and N2.
* **I4: fixed.** `ns_sigbytes` matches 3/3 (`h NOSTRING [The NIL object] 3`), and
  `string_answer_nostring_trapped.rex` matches 3/3. It gave rc 120 at `f8d7049e8`.

### New findings

#### N1 (Critical). The ordering comparisons compare against the text "The NIL object", and the oracle does not

`eval.rs:1324` sends every non-concatenation operator to `required_string_or_nil`. The `.nil` rule
at `:1325-1329` covers only equality, so `<`, `>`, `<=`, `>=`, `<<`, `>>`, `<<=` and `>>=` compare
the left side with "The NIL object". In the oracle, `RexxString::comp` runs `stringComp` on
`other->requestString()` (`StringClass.cpp:774`, `:795-801`). That reads `getLength()` and
`getStringData()` from `.nil` through the string layout, so the answer is a garbage read. Probe
`r1/order.rex`, 3/3 per engine:

```
                                             crate     oracle
"The NIL object" <= o, >= o                  1 1       0 1
"The NIL objecs" < o, "The NIL objecu" > o   1 1       0 1
<<=  >>=  <<  >>  (same operands)            1 1 1 1   0 1 0 1
"Tha"<o "Thf">o "S"<o "U">o "a">o "A"<o      1 1 1 1 1 1   0 1 0 1 1 0
1<o 1>o ""<o " "<o                           1 0 1 1   0 1 0 0
```

`r1/empty.rex` gives `"" < o` 0 but `"" << o` 1 on the oracle, which no text could produce. The
witness line `ordered ("abc" < o) ("abc" >> o) ("Z" > o)` gives `0 1 1` on both engines only
because those inputs happen to agree. Equality is a layout read in the oracle too. `primitiveIsEqual`
checks for `.nil` before converting, against the original operand, and then compares a garbage
length. Its answers (`=`, `==` are 0 and `\=`, `\==` are 1, including against `""`) are what the
crate's rule gives (`r1/empty.rex`: `eq 0 0 0 1 1` on both engines), so equality may stay by the
controller's ruling.

**Fix:** let `required_string_operand` take the `.nil` path only for `& | &&` and the equality
operators (`is_equality`), and refuse the ordering operators. Remove the `ordered` line from
`string_answer_nil_consumers.rex` and regenerate its sourceline file. In Deviation 30, list the
ordering operators as refused and say that equality is a layout read whose answer the crate pins.

#### N2 (Critical). PARSE VALUE and PARSE ARG answer the text where the oracle answers `.nil` or Error 5

`parse_template.rs:445` and `:631` now read `.nil` as "The NIL object" for every template. The
oracle reads `.nil` through the string layout as soon as the template splits, and a one-variable
template assigns the `.nil` object itself. Probes in `r1/`, 2/2 per engine:

| probe | crate | oracle |
|---|---|---|
| `parse value o with p q r` | `[The][NIL][object]` | rc 251, Error 5 |
| `parse value o with p 4 q` | `[The][ NIL object]` | rc 251, Error 5 |
| `parse value o with p "N" q` | `[The ][IL object]` | rc 251, Error 5 |
| `parse value o with p .` | `[The]` | a different stdout in each run |
| `parse upper value o with p` | `[THE NIL OBJECT]` | rc 251, Error 5 |
| `parse value o with p; say (p == .nil) (p = .nil)` | `0 0` | `1 1` |
| `call pa o` / `parse arg p q` | `[The][NIL object]` | rc 251, Error 5 |
| `call pa o` / `parse arg p; say (p == .nil)` | `0` | `1` |

At `f8d7049e8` each of these refused with rc 120. PARSE ARG was moved without a ruling. PARSE
VALUE was ruled in by the controller, but only the one-variable case was observed, and there the
oracle assigns `.nil`, not a string.

**Fix:** put both sites back in the bytes set. The one defined case is a single-variable template
with no UPPER/LOWER/CASELESS and no other template element, which assigns `.nil`. Answer it only
with its own witness, or ask the controller for a ruling first. Remove the `parse value` and
`parse arg` lines from the witness, or change them to match.

#### N3 (Minor). OPTIONS has no observable effect, so its row cannot show a defined answer

`c03_options` matches (rc 0, `end`), but a garbage read of `.nil` would also do nothing visible.
The row is harmless either way. **Fix:** mark it "no observable" in the table instead of citing
the defined-answer rule.

#### N4 (Minor). The classification table's PARSE rows are wrong

The `parse_template.rs:445` and `:631` rows say "The NIL object". That is true only for a
one-variable template, and even there the oracle assigns `.nil`. **Fix:** correct the rows with N2.

### Requested checks

* **(2) Classification table.** I ran 19 rows: `c01`, `c02`, `c03`, `c06`, `c07`, `c08`, `c09`,
  `c10`, `c12`, `c13`, `c14`, three `c15_*`, `c19`, `b02`, `b11`, `b33` and `parsearg`, 1 crate run
  and 2 oracle runs each. I did not re-run `c05`, `c11` or `b08`, because they crash the oracle and
  are listed in `oracle-crashes.txt`. Each refused-garbage row is rc 120 against the oracle's
  Error 5, 98.920 or 24.1. Each 88.909 row is rc 120 against 88.909. On the ruled-out moves without
  a ruling:
  * NUMERIC FORM VALUE (25.11 "a SN"), the semaphore timeout (88.902 "a SN"), and the native int,
    double, size, stem, CSTRING and ObjectToString conversions match. That includes
    `string_answer_nil_native.rex` 2/2 with `LD_LIBRARY_PATH` set to the oracle's lib. Each
    depends only on the conversion failing or on `.nil` as an object, so they are defined.
  * OPTIONS: N3.
  * PARSE ARG: N2.
* **(3) Comparisons.** `=` and `==` against "The NIL object" give 0 on both engines, and `\=`/`\==`
  give 1. The ordering operators compare against the text in the crate and do not match the
  oracle (N1).
* **(4) Sidecar.** `string_answer_nil_native.env` is the existing `tests/support/sidecar.rs`
  mechanism. It has the same `LD_LIBRARY_PATH={oraclelib}` line as `trapped_condition_native_levels.env`
  and the other native corpus entries, so the in-process side can load `orxmethod`/`orxfunction`
  from the oracle's lib. It is right and follows precedent.
* **(5) Breakage.** Outside N1 and N2 I found nothing. The 39 `b/` probes, the Step 6b probes and
  all new corpus witnesses match or refuse as expected. I did not re-run the perf figures, as
  instructed.
