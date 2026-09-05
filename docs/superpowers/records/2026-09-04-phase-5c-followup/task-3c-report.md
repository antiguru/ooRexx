# Task 3c report -- the mutators, over the cores

The third Task 3 commit of `docs/superpowers/plans/2026-09-04-phase-5c-followup.md`: eleven
`MutableBuffer` mutator rows bound over the cores Task 3a landed, one corpus witness filed with
them. BASE is `3c4585cb4bcf3a77f91e5e4bd45af731c354dcc9`. `$S` below is
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/ae5f3c99-11ad-4b29-9dfe-f3d991a1a398/scratchpad/task3c`
and `$B` is `/home/moritz/dev/repos/claude-build-scratch/5c-followup-task3c`. Every claim is marked
**measured** (with the command or file that produced it) or **inferred**.

## 1. What landed

`git diff --stat` at the commit: six tracked files changed, three added.

* `crates/rexx-exec/src/dispatch.rs` -- eleven `MutableBuffer` instance rows, each at the count
  `interpreter/memory/Setup.cpp:1416`-`:1479` declares: `insert` 4, `overlay` 4, `replaceAt` 4,
  `[]=` 3, `changeStr` 3, `upper` 2, `lower` 2, `translate` 5, `space` 2, `delWord` 2, `delete` 2.
  New method-layer argument helpers -- `optional_string_method_argument`,
  `optional_non_negative_argument` (93.906), `named_string_argument`, `named_position_argument`
  (88.912), `optional_named_length_argument` (88.911) and `named_pad_argument` (88.910) -- plus the
  shared `buffer_delete`, `buffer_replace_at`, `buffer_case_shift`, `replace_buffer_contents` and
  `buffer_capacity`. `whole_method_argument`, `refuse_method_argument` and `usize_or_refuse` take
  their raiser as `impl Fn(&[u8]) -> Raised` rather than a `fn` pointer, because the new families
  close over the argument's position or its name.
* `crates/rexx-exec/src/error.rs` -- `Raised::named_argument_invalid_pad` (88.910),
  `named_argument_invalid_length` (88.911) and `named_argument_invalid_position` (88.912), the three
  raises `replaceAt`'s named-argument overloads produce where the positional overloads produce
  93.922, 93.923 and 93.924. Each was measured on the oracle before it was written (section 2.3).
* `corpus/lang/mutablebuffer_mutators.rex` (128 lines, 74 lines of output), filed in
  `corpus/phase-5c.txt` and `EXPECTED_SUBSET_5C`, with its
  `rexx-parse/tests/sourceline_oracle/mutablebuffer_mutators.txt` companion.
* `corpus/method-bodies.txt` -- refreshed: eleven `MutableBuffer` rows move, six `loud` to
  `answers` and five `loud` to `loud` with new evidence (section 4).
* `corpus/refusal-sites.tsv` -- re-derived from `refusal_sites.rs`'s own panic by a script
  (`$S/ctrl/rederive.py`), not edited by hand: every `error.rs` row below
  `named_argument_invalid_pad`'s insertion point moves down by the lines the three new doc blocks
  and bodies occupy -- 34 of them, and the diff's changed rows are `error.rs` rows and nothing else
  -- with the same `(kind, name, surface)` keys on both sides of the disagreement; three rows are
  new, each
  `Raised … send … agrees yes` with the witness probed on the oracle and on both engines before the
  row was written. `--test refusal_sites` then reads 5 passed.

No core in `builtin/string.rs` or `builtin/word.rs` changed: the mutators call
`substr_bytes`, `insert_bytes`, `overlay_bytes`, `changestr_bytes`, `space_bytes`,
`translate_bytes`, `case_shift_bytes`, `delete_range`, `count_occurrences`, `delword_bytes` and
`word_count` as Task 3a left them.

## 2. The witness and the oracle

### 2.1 `corpus/lang/mutablebuffer_mutators.rex`

Written and run on the oracle before any crate file was touched -- except for the four sends the
M2 mutation added afterwards (section 3.1), which were run on the oracle before the widened program
was filed. **Measured**, from a fresh empty
directory under the plan's wrapper (`$S/oracle/w/`, descriptors
`$S/oracle/w/mutablebuffer_mutators.{out,err,rc}`): rc 0, stderr 0 bytes, 74 lines of stdout. The
program:

```rexx
/* The mutators over a MutableBuffer's contents -- insert, overlay,
   replaceAt, []=, changeStr, upper, lower, translate, space, delWord and
   delete -- each change the buffer in place and answer the receiver, and
   the ones that can outgrow the capacity raise it. Rendering the buffer
   itself -- say, concatenation, makeString -- is deliberately absent. */
buf = .MutableBuffer~new('abcdef')
/* insert: the position is 0-based and defaults to 0; a position past the
   end pads the gap and a length longer than the string pads the insertion,
   with a blank where no pad is given; a zero-length insertion inside the
   contents changes nothing. */
say buf~insert('XY')~string buf~length
say buf~insert('Z', 3)~string
say buf~insert('', 2)~string buf~insert('QR', 2, 0)~string
say buf~insert('QR', 20, 4, '-')~string buf~length
say buf~insert('PQ', 2, 5, '.')~string buf~insert('W', 2, 1)~string
say .MutableBuffer~new('abcdef')~insert('QR', 9)~string'|' .MutableBuffer~new('abcdef')~insert('QR', 2, 5)~string'|'
/* overlay: the position is 1-based and defaults to 1; past the end it pads
   the gap, with a blank where no pad is given; a zero length and an empty
   string with the default length each change nothing. */
buf = .MutableBuffer~new('abcdef')
say buf~overlay('XY')~string
say buf~overlay('Z', 4)~string
say buf~overlay('QR', 9, 4, '-')~string buf~length
say buf~overlay('W', 2, 0)~string buf~overlay('', 3)~string buf~length
say buf~overlay('', 3, 2, '+')~string buf~length
say .MutableBuffer~new('abcdef')~overlay('QR', 10)~string'|' .MutableBuffer~new('abcdef')~overlay('QR', 2, 5)~string'|'
/* replaceAt: the length defaults to the replacement's own, is cut at the
   end of the contents, an empty replacement excises the range, and a
   position past the end pads the gap with a blank where no pad is given. */
buf = .MutableBuffer~new('abcdef')
say buf~replaceAt('XY', 3)~string buf~length
say buf~replaceAt('Z', 3, 2)~string buf~length
say buf~replaceAt('QRST', 2, 1)~string
say buf~replaceAt('', 2, 4)~string buf~length
say buf~replaceAt('P', 9, 2, '-')~string buf~length
say buf~replaceAt('N', 5, 99)~string buf~replaceAt('', 2, 0)~string buf~length
say .MutableBuffer~new('abcdef')~replaceAt('QR', 10)~string'|'
/* []= is replaceAt with the default pad -- it takes no pad argument of its
   own -- under the message name and under the subscript form. */
buf = .MutableBuffer~new('abcdef')
say buf~'[]='('XY', 3)~string
buf[3] = 'Z'
say buf~string buf~length
buf[3, 4] = 'PQ'
say buf~string buf~length
say buf~'[]='('', 2, 2)~string buf~length
say .MutableBuffer~new('abcdef')~'[]='('QR', 10)~string'|'
/* changeStr: equal, shorter and longer replacements, a count that stops
   early, and the three that change nothing -- an absent needle, an empty
   needle and a zero count. */
buf = .MutableBuffer~new('abcabcabc')
say buf~changeStr('bc', 'ZZ')~string buf~length
say buf~changeStr('ZZ', 'Q')~string buf~length
say buf~changeStr('Q', 'RST')~string buf~length
say buf~changeStr('RST', 'Q', 2)~string
say buf~changeStr('zz', 'Q')~string buf~changeStr('', 'Q')~string buf~changeStr('a', 'Q', 0)~string
/* upper and lower: the whole contents, a range, and the two non-changes --
   a start past the end and a zero length. */
buf = .MutableBuffer~new('abcDEF')
say buf~upper~string buf~length buf~getBufferSize
say buf~lower(2, 3)~string
say buf~upper(9)~string buf~lower(2, 0)~string
say buf~lower(2, 99)~string
say .MutableBuffer~new('a1B!c')~upper~string
/* translate: an output table with an input table, a shorter output table
   padding, an output table alone, an input table alone, a range, and the
   no-table form, which is upper. */
buf = .MutableBuffer~new('abcdef')
say buf~translate('ABC', 'abc')~string
say buf~translate('xy', 'ABC', '-')~string
say buf~translate('QQ', 'def', , 4, 2)~string
say buf~translate('A', 'a', , 9)~string buf~translate('A', 'a', , 1, 0)~string
say .MutableBuffer~new('abcdef')~translate('ABC')~string'|'
say .MutableBuffer~new('abcdef')~translate(, 'abc')~string'|'
say .MutableBuffer~new('abcdef')~translate(, , '?')~string
say .MutableBuffer~new('abcdef')~translate~string
say .MutableBuffer~new('abcdef')~translate(, , , 2, 3)~string
/* space: the default single blank, none, several, a pad, and the two that
   have no interstice to fill -- one word and no words. */
say .MutableBuffer~new('  now is  the time  ')~space~string'|'
say .MutableBuffer~new('  now is  the time  ')~space(0)~string
say .MutableBuffer~new('  now is  the time  ')~space(3)~string'|'
say .MutableBuffer~new('  now is  the time  ')~space(2, '-')~string
say .MutableBuffer~new('one')~space(4, '=')~string'|'
say .MutableBuffer~new('   ')~space~length .MutableBuffer~new('')~space(2)~length
/* delWord: from a word to the end, a count, and the three non-changes -- a
   position past the last word, a zero count and a buffer with no words. */
say .MutableBuffer~new('  now is  the time  ')~delWord(2)~string'|'
say .MutableBuffer~new('  now is  the time  ')~delWord(2, 1)~string'|'
say .MutableBuffer~new('  now is  the time  ')~delWord(1, 2)~string'|'
say .MutableBuffer~new('  now is  the time  ')~delWord(5)~string'|'
say .MutableBuffer~new('  now is  the time  ')~delWord(2, 0)~string'|'
say .MutableBuffer~new('   ')~delWord(1)~length .MutableBuffer~new('')~delWord(1)~length
/* delete is delStr's method under its second name: a start, a start and a
   length, an omitted start, and a start past the end. */
say .MutableBuffer~new('abcdef')~delete(2)~string'|'
say .MutableBuffer~new('abcdef')~delete(2, 3)~string
say .MutableBuffer~new('abcdef')~delete(, 2)~string
say .MutableBuffer~new('abcdef')~delete(99)~string .MutableBuffer~new('abcdef')~delete(2, 0)~string
say .MutableBuffer~new('abcdef')~delete~length
/* Every mutator answers the receiver itself, and they chain. */
buf = .MutableBuffer~new('  now is  the time  ')
say (buf~upper == buf) (buf~space == buf) (buf~insert('') == buf) (buf~delete(99) == buf)
say (buf~overlay('n', 1) == buf) (buf~replaceAt('N', 1) == buf) (buf~changeStr('z', 'Q') == buf)
say (buf~lower == buf) (buf~translate('A', 'a') == buf) (buf~delWord(9) == buf)
say .MutableBuffer~new('  now is  the time  ')~upper~space~delWord(2, 1)~string
/* Growth: insert, overlay, replaceAt, []=, changeStr and space each raise
   the capacity past the size the buffer was built with, and past the 256
   the constructor defaults to. */
grow = .MutableBuffer~new('abc', 10)
say grow~insert(copies('y', 40), 0)~length grow~getBufferSize
say grow~insert('Q', 400)~length grow~getBufferSize
grow = .MutableBuffer~new('abc')~insert(copies('y', 400), 0)
say grow~length grow~getBufferSize
grow = .MutableBuffer~new('abc')~overlay('Z', 400)
say grow~length grow~getBufferSize
grow = .MutableBuffer~new('abc', 10)~overlay('Z', 400)
say grow~length grow~getBufferSize
grow = .MutableBuffer~new('abc')~replaceAt(copies('z', 400), 2)
say grow~length grow~getBufferSize
grow = .MutableBuffer~new('ab')~'[]='(copies('w', 300), 1)
say grow~length grow~getBufferSize
grow = .MutableBuffer~new('abcabc')~changeStr('b', copies('q', 200))
say grow~length grow~getBufferSize
grow = .MutableBuffer~new('a b c', 10)~space(20)
say grow~length grow~getBufferSize
grow = .MutableBuffer~new('a b c')~space(200)
say grow~length grow~getBufferSize
```

**Measured**, this build, both engines (`$S/probes/mutablebuffer_mutators.{ir,tree-walker}.*`, run
by `$S/run_crate.sh` from a fresh empty directory): rc 0 on both, and `cmp` against the oracle's
stdout and stderr is byte-identical on both.

**What each name's cases are**, since the brief asks for a positive and a negative per name and
"negative" here means a send that changes nothing rather than one that refuses -- the refusals are
section 2.3's. `insert` and `overlay` each get the defaulted position, a position inside the
contents, a position past the end with an explicit pad, an insertion length longer than the string,
and the two forms that write the *default* pad -- the gap past the end and the run after a short
string -- against `insert('', 2)`, `insert('QR', 2, 0)`, `overlay('W', 2, 0)` and `overlay('', 3)`,
which change nothing. `replaceAt` gets the defaulted length, a shorter and a longer replacement, a
length cut at the end, a position past the end with a pad and one with the default pad, and an empty
replacement excising a range, against `replaceAt('', 2, 0)`. `changeStr` gets an equal-length, a
shorter and a longer replacement and a count that stops early, against an absent needle, an empty
needle and a zero count. `upper` and `lower` each get the whole contents and a range, against a
start past the end and a zero length. `translate` gets an output table with an input table, a
shorter output table padding, an output table alone, an input table alone, a range, a pad-only form
and the no-table form that is `upper`, against a start past the end and a zero range. `space` gets
the default single blank, none, several and a pad, against a one-word and a no-word buffer, which
have no interstice to fill. `delWord` gets a word to the end, a count and the first two words,
against a position past the last word, a zero count and a buffer with no words. `delete` gets a
start, a start and a length and an omitted start, against a start past the end and a zero length.

**`[]=` is the one name with no no-change case.** Its sends are the message form, the one- and
two-subscript forms, an empty replacement excising a range -- the degenerate direction -- and a
position past the end, which is the only way to observe the pad `[]=` can never be given. It shares
`replaceAt`'s C++ body (`MutableBufferClass.cpp:545`), whose no-change case is covered above, so
what `[]=`'s own sends have to pin is the binding: the name, the count of 3, the pad left to its
default and the subscript syntax.

### 2.2 Against BASE's binary -- the negative control

**Predicted before running** (`$S/ctrl/predictions.md`, C1): rc 120, 0 lines of stdout, stderr
`rexx-exec: method "INSERT" of class "MutableBuffer" is not implemented (Phase 5)` on both engines,
because `insert` is the first mutator the program sends. (The prediction said "line 9", which was
the line it sat on before the widening of section 3.1 moved it; the loud message carries no line
number, so nothing measured turned on it.)

**Measured**: BASE built from `git archive 3c4585cb4 rust interpreter` under `$B/base/` with
`CARGO_TARGET_DIR=$B/target-base` (`$S/base-build.status`: `build rc=0`, binary sha256
`9371cc818a480c5b33d28c0555a646a010f8470b9dd91330fc78bed8113ff0f3`). The witness against it
(`$S/probes/base/mutablebuffer_mutators.{ir,tree-walker}.*`): both engines rc 120, 0 lines of
stdout, stderr exactly that line. **Confirmed.** (The archive needs `interpreter/` as well as
`rust/`, as Task 3b recorded: `rexx-classes/build.rs` and `rexx-lib/build.rs` read it.)

### 2.3 Refusals measured before they were written

Every refusal was run on the oracle first as a two-line program -- `buf = .MutableBuffer~new('abcdef')`
then the probe -- from a fresh empty directory (`$S/oracle/r/`, `$S/oracle/r2/` for the
conversion-order cases and `$S/oracle/r3/` for the rendering cases, runner `$S/run_oracle.sh`), then
on this build under both engines (`$S/run_crate.sh`). **Measured**: over `$S/oracle/{r,r2,r3,e,e2}`
the loop compared stdout, stderr and rc raw -- both sides run the same absolute path -- and read
**276 probe-engine pairs, 0 mismatching**.

| probe | oracle | which raiser here |
|---|---|---|
| `insert`, `insert(, 1)`, `overlay`, `changeStr`, `changeStr('a')`, `delWord`, `delWord(, 1)` sent nothing at the position the C++ requires | 93.903 `Missing argument in method; argument N is required.`, rc 163 | `string_method_argument` / `required_position_argument` → `Raised::missing_method_argument(N)` |
| `insert(.nil)`, `overlay(.nil)`, `changeStr(.nil, 'b')`, `changeStr('a', .nil)`, `translate(.nil)`, `translate('a', .nil)` | 88.909 `Argument N must have a string value.`, rc 168 | `required_string_argument(.., N)` |
| `insert('a', 1, 3, .nil)`, `overlay('a', 1, 3, .nil)`, `translate('a', 'b', .nil)`, `space(1, .nil)` | 88.909 `Argument N must have a string value.` | `pad_method_argument` → `required_string_argument(.., N)` |
| `insert('a', -1)`, `('a', 'x')`, `('a', .nil)`, `('a', '1.5')`, `('a', 99999999999999999999)`, `('a', '-1.0')`, `('a', ' -1 ')`, `('a', .array)`; `changeStr('a', 'b', -1)`, `('a','b','x')`, `('a','b',.nil)`, `('a','b','1.5')`, `('a','b',99999999999999999999)`, `('a','b','-1.0')` | **93.906** `Method argument N must be zero or a positive whole number; found "…"`, rc 163 | `optional_non_negative_argument` → `Raised::argument_not_non_negative` -- `insert`'s position is `optionalNonNegative` (`MutableBufferClass.cpp:426`), not a position argument, which is why `insert('a', 0)` answers |
| `overlay('a', 0)`, `('a', 'x')`, `('a', .nil)`, `('a', '0.0')`; `upper(0)`, `('x')`, `(.nil)`, `(' 0 ')`; `lower(0)`; `translate('a','b','-',0)`, `('a','b','-','x')`; `translate(, , , 0)`; `delWord(0)`, `('x')`, `(.nil)`; `delete('x')`, `(0)`, `(.nil)` | 93.924 `Invalid position argument specified; found "…"`, rc 163 | `optional_position_argument` |
| `insert('a', 1, -1)`, `(1,'x')`, `(1,.nil)`; `overlay('a', 1, -1)`, `(1,'x')`; `upper(1, -1)`, `(1,'x')`; `lower(1, -1)`; `translate('a','b','-',1,-1)`, `(…,1,'x')`; `translate(, , , 1, -1)`; `space(-1)`, `('x')`, `(.nil)`; `delWord(1, -1)`, `(1,'x')`; `delete(1, -1)`, `(1,'x')` | 93.923 `Invalid length argument specified; found "…"`, rc 163 | `optional_length_argument` |
| `insert('a', 1, 3, 'xx')`, `(1,3,'')`; `overlay('a', 1, 3, 'xx')`; `translate('a','b','xx')`; `space(1, 'xx')` | 93.922 `Incorrect pad or character argument specified; found "…"`, rc 163 | `pad_method_argument` → `Raised::incorrect_pad` |
| `replaceAt`, `~'[]='()` | **88.901** `Missing argument; argument new is required.`, rc 168 | `named_string_argument(.., "new")` → `Raised::missing_named_argument` |
| `replaceAt('a')`, `('a', )`, `~'[]='('a')`, `buf[] = 'a'` | 88.901 `Missing argument; argument position is required.` | `named_position_argument(.., "position")` |
| `replaceAt(.nil)`, `~'[]='(.nil)` | 88.909 `Argument new must have a string value.`, rc 168 | `required_string_named_argument(.., "new")` |
| `replaceAt('a', 1, 3, .nil)` | 88.909 `Argument pad must have a string value.` | `named_pad_argument` → `required_string_named_argument(.., "pad")` |
| `replaceAt('a', 0)`, `('a','x')`, `('a',.nil)`, `('a','0.0')`, `('a',' -1 ')`, `('a',.array)`; `~'[]='('a', 0)`, `('a','x')`; `buf['x'] = 'a'` | **88.912** `Argument position is an invalid position value; found "…"`, rc 168 | `Raised::named_argument_invalid_position`, **new** in `error.rs` |
| `replaceAt('a', 1, -1)`, `(1,'x')`, `(1,.nil)`, `(1,'-1.0')`; `~'[]='('a', 1, -1)` | **88.911** `Argument length is an invalid length value; found "…"`, rc 168 | `Raised::named_argument_invalid_length`, **new** |
| `replaceAt('a', 1, 3, 'xx')`, `(1,1,12)`, `(1,1,'')` | **88.910** `Argument pad is an invalid pad or character argument; found "…"`, rc 168 | `Raised::named_argument_invalid_pad`, **new** |
| `upper(1,1,1)`, `lower(1,1,1)`, `space(1,'-',1)`, `delWord(1,1,1)`, `delete(1,1,1)` | 93.902 `Too many arguments in invocation of method; 2 expected.`, rc 163 | `Arity::Fixed(2)`, the table |
| `changeStr('a','b',1,1)`, `~'[]='('a',1,3,4)` | 93.902 `3 expected` | `Arity::Fixed(3)` |
| `insert('a',1,3,'-',5)`, `overlay('a',1,3,'-',5)`, `replaceAt('a',1,3,'-',5)` | 93.902 `4 expected` | `Arity::Fixed(4)` |
| `translate('a','b','-',1,1,1)` | 93.902 `5 expected` | `Arity::Fixed(5)` |

**`found` is the argument's own rendering on every one of these, never the converted value**, which
is what `refuse_method_argument`'s `string_value_text` already did for the positional families:
measured, `insert('a', '-1.0')` reports `found "-1.0"` and `replaceAt('a', '0.0')` reports `found
"0.0"`, where the builtin layer's identically-numbered 93.906 reports the converted `found "-1"`
(not re-measured here; it is the figure `Raised::argument_not_non_negative`'s own doc block carries
for `copies('ab','-1.0')`).
The C++ passes the argument object -- `requiredNonNegative` (`classes/ObjectClass.cpp:1608`),
`positionArgument` (`classes/StringClassUtil.cpp:225`) -- rather than the number it failed to build.

**Argument conversion order, Task 3b's §7 question, answered for these eleven by running it.**
Every one of them converts every argument before it acts, so none has `match`'s
return-before-converting shape. **Measured**, `$S/oracle/r2/`: `insert('', 2, , .nil)` is 88.909
argument 4 although a zero-length insertion inside the contents returns unchanged;
`insert('a', 2, 0, 'xx')`, `overlay('a', 99, 1, 'xx')` and `replaceAt('a', 99, 1, 'xx')` are the pad
refusal; `upper(9, -1)`, `lower(9, 'x')`, `translate('a','b','-',9,-1)`, `delWord(9, -1)`,
`delWord(1, -1)` on an empty buffer and `delete(99, -1)` are the length refusal; `changeStr('a',
.nil, 0)` is 88.909 argument 2 although a zero count returns unchanged;
`changeStr('', 'b', -1)` is 93.906 although an empty needle returns unchanged; and
`space(1, 'xx')` on an empty buffer is the pad refusal. The bodies convert in the same order.

Semantics measured the same way and agreeing on both engines (`$S/oracle/e/`, `$S/oracle/e2/`,
`$S/oracle/r/`):
`insert`'s position is a 0-based count, so `insert('a', 0)` answers where `overlay('a', 0)` is
93.924; `translate` with all three table arguments omitted is `upper` taking its position and length
from arguments four and five, so `translate(, , , 2, 3)` is `aBCDef`; an omitted `translate` input
table is the null string and reads each byte as its own index, so `translate('ABC')` on `abcdef` is
six blanks; `replaceAt('XY', 7)` on six bytes is `abcdefXY` rather than a padded gap, because
`begin > dataLength` is false at exactly the end.

### 2.4 Which methods can grow the buffer, and how the capacity is raised

**Measured** per method, oracle rc 0, and the arithmetic **inferred** from the C++ line the row
cites. `ensureCapacity(added)` (`MutableBufferClass.cpp:236`) takes `dataLength + added` as the
needed size and answers `max(needed, 2 * bufferLength)`, so **what each method passes is
observable through `getBufferSize`, and the call sites this commit reaches do not all pass the same
thing**.

| method | can grow | what the C++ passes to `ensureCapacity` | measured |
|---|---|---|---|
| `insert` | yes | `insertLength`, or `insertLength + (begin - dataLength)` past the end (`:444`, `:448`) -- both make the needed size the result's own length | `.MutableBuffer~new('abc', 10)~insert(copies('y', 40), 0)` reads `43 43`; `~insert('Q', 400)` on that reads `401 401`; `.MutableBuffer~new('abc')~insert(copies('y', 400), 0)` reads `403 512` |
| `overlay` | yes | `begin + replaceLength` (`:502`) -- **not** the result's length | `.MutableBuffer~new('abc', 10)~overlay('Z', 400)` reads `400 403`, the capacity outrunning the contents by the original length; the 256-default form reads `400 512` |
| `replaceAt` | yes | `finalLength` (`:607`) | `.MutableBuffer~new('abc')~replaceAt(copies('z', 400), 2)` reads `401 512` |
| `[]=` | yes | `replaceAt`'s, with the pad left to its default (`:545`) | `.MutableBuffer~new('ab')~'[]='(copies('w', 300), 1)` reads `300 512` |
| `changeStr` | yes, on one branch | `resultLength`, **only** where the replacement is longer than the needle (`:1081`); the equal-length and shorter branches call it not at all | `.MutableBuffer~new('abcabc')~changeStr('b', copies('q', 200))` reads `404 512` |
| `space` | yes, for a pad longer than one byte | `count * (padLength - 1)`, **after** `dataLength` has been set to the single-blank form's length (`:2031`, `:2038`) | `.MutableBuffer~new('a b c', 10)~space(20)` reads `43 43`; the 256-default form under `space(200)` reads `403 512` |
| `upper`, `lower` | no | -- | in place over the existing bytes |
| `translate` | no | -- | in place over the existing bytes |
| `delWord` | no | -- | a `drain` |
| `delete` | no | -- | `delete_range`, as `delStr` |

The bodies reproduce each of those expressions rather than a common "grow to the result" rule, and the witness's growth block is what holds them: it reads `43 43 / 401 401 / 403 512 /
400 512 / 400 403 / 401 512 / 300 512 / 404 512 / 43 43 / 403 512`.

## 3. Controls, each predicted before it ran

Predictions in `$S/ctrl/predictions.md` (C1, C4, C6 and every mutation) and
`$S/ctrl/method-bodies-prediction.txt` (section 4), each written before the run it names.

| # | control | prediction | reading |
|---|---|---|---|
| C1 | the witness against BASE's binary (§2.2) | rc 120 at `INSERT`, stderr naming it, both engines | **confirmed** |
| C2 | the witness on this build against the oracle's descriptors, both engines | rc 0, stdout and stderr `cmp`-identical | **confirmed** (§2.1) |
| C3 | every probe on this build against the oracle, both engines | all three descriptors identical | **confirmed**, 276 pairs, 0 mismatching (§2.3) |
| C4 | the probe comparison can see a difference | `cmp` of two oracle stdouts that differ exits 1 | **confirmed**: `cmp $S/oracle/w/mutablebuffer_mutators.out $S/oracle/e/e1_insert.out` exits 1, `differ: byte 1, line 1` (`$S/ctrl/c4/out.txt`) |
| C5 | `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus`, read with its `N of M matching` line | exit 0, `M of M matching` with M one more than before this commit | **confirmed**: rc 0, `361 of 361 matching` (`$S/ctrl/fast/corpus.err`; Task 3b read 360 of 360) |
| C6 | the pin's inversion: `lang/mutablebuffer_mutators.rex` removed from `EXPECTED_SUBSET_5C` only | `phase_5c_subset_matches_the_committed_list` red | **confirmed**: rc 101, that one test FAILED and the other 19 ok (`$S/ctrl/c6/out.txt`); `coverage.rs` restored from `$S/coverage.rs.base`, the line re-added and the file `touch`ed, then 20 passed (`$S/ctrl/c6/after.txt`) |
| C7 | mutations, one per family plus one that defeats `ensure_capacity`'s growth (§3.1) | each red at the named catcher, and green without the new witness where the new witness is the claimed catcher | **six of seven confirmed, one falsified and then fixed**: M2 was green, because no line of the witness observed a default pad byte; four `say` lines were added, M2 re-ran red, and the widened witness is what is committed (§3.1). M6 is red but **adds no coverage**, which its own prediction said and the without-the-witness run measured |

### 3.1 Mutations

Predictions written before any mutation ran (`$S/ctrl/predictions.md`) and, for the re-run,
`$S/ctrl/predictions-m2b.md`, written after M2's reading and before M2b's. The run
`$S/mut/run_all.sh` 11:07 to 11:34 and `$S/mut/run_m2b.sh` 11:36 to 11:40, one mutation at a time,
each applied by `$S/mut/mutate.py`, which refuses unless the replaced text occurs exactly once.
Every build is `--profile mutation` (`target/mutation/`); after each mutation the edited file is
restored from a copy, `cmp`-checked and `touch`ed. Each mutant's `target/mutation/rexx-run` sha256
is recorded: `5bfcb2be794eb737`, `846162ab0eeac50f`, `a73b395404897caa`, `9cff8298ba26c4bc`,
`ab811adb45069b49`, `0f364d3e1da81f1a` -- six distinct values, so no run measured a stale binary --
and M2b's is `846162ab0eeac50f` again, the same mutation over an unchanged source. **The release
binary reads `062f3e5e868f643f…` before the run, after every restore, and after a full rebuild at
the end**, which is also the control that the restores were byte-exact.

The checks per mutation are `cargo test --profile mutation -p rexx-exec --lib --no-fail-fast`,
`REXX_CORPUS_GATE=1 … --test corpus --no-fail-fast`, `… --test method_bodies --no-fail-fast`, and
then the same STRICT corpus again with `lang/mutablebuffer_mutators.rex` taken out of
`corpus/phase-5c.txt` -- "can fail" is not "adds coverage". **That last run exits 101 for a reason
that is not the differential**: removing the line leaves the program in `corpus/lang/` and in no
subset file, which reddens `every_lang_program_is_run_or_named_unfiled` (`corpus.rs:845`). Its
`N of M matching` line is the reading, and `corpus_differential` is among the tests that pass.

| id | site | mutation | predicted catcher | reading |
|---|---|---|---|---|
| M1 | `native_mutable_buffer_insert` | the position default `unwrap_or(0)` becomes `unwrap_or(1)` | STRICT corpus only, on the new witness | **confirmed**: lib 779/0, corpus rc 101 `360 of 361 matching` on `lang/mutablebuffer_mutators.rex`, method_bodies 0 regressions and 0 drift; without the witness `360 of 360` |
| M2 | `native_mutable_buffer_overlay` | the pad default `unwrap_or(b' ')` becomes `unwrap_or(b'.')` | STRICT corpus only | **falsified**: corpus rc 0, `361 of 361 matching`. The prediction cited a witness line that did not exist -- it had been written from the exploration program, not from the witness -- and reading the witness again showed that **no line of it observed a default pad byte** for `insert`, `overlay`, `replaceAt` or `[]=`. The growth block does reach those paths (`insert('Q', 400)`, `overlay('Z', 400)`) but prints only the length and the capacity |
| M2b | the same | the same, against the widened witness | STRICT corpus red, and green without the witness | **confirmed**: corpus rc 101 `360 of 361 matching`, method_bodies 0 regressions, and without the witness `360 of 360 matching` -- so the added sends are what catches it and nothing else in the corpus does |
| M3 | `native_mutable_buffer_changestr` | `limit` defaults to `1` instead of `usize::MAX` | STRICT corpus only | **confirmed**: corpus rc 101 `360 of 361`; without the witness `360 of 360` |
| M4 | `native_mutable_buffer_space` | the gap default `unwrap_or(1)` becomes `unwrap_or(0)` | STRICT corpus only | **confirmed**: corpus rc 101 `360 of 361`; without the witness `360 of 360` |
| M5 | `native_mutable_buffer_delword` | the 1-based word ordinal becomes `position + 1` | STRICT corpus only | **confirmed**: corpus rc 101 `360 of 361`; without the witness `360 of 360` |
| M6 | `BufferState::ensure_capacity` (`rexx-core/src/body.rs`) | `needed.max(self.capacity * 2)` becomes `needed` | STRICT corpus, on `mutablebuffer_state.rex` **and** this witness | **confirmed, and the one that adds no coverage**: corpus rc 101 `359 of 361 matching`, both `lang/mutablebuffer_state.rex` and `lang/mutablebuffer_mutators.rex`; without the new witness `359 of 360`, Task 2's program alone. This is the mutation the brief asked for -- one that defeats `ensure_capacity`'s growth -- and the measurement says the new witness is redundant against it |

`--lib` is green under all seven, `779 passed; 0 failed` each: the mutators have no unit tests, and
`BufferState`'s growth rule has none in `rexx-exec`'s `--lib` either. `--test method_bodies` is
green under all seven, `regressions this run: 0. other drift from the committed table: 0.`, and the
reason differs by row: `insert`, `overlay`, `changeStr` and `delWord` raise before the mutated
default is used, `space` answers the receiver and its row is `loud` at `MAKESTRING` whatever the
gap, and `ensure_capacity`'s growth rule is never called at all on a three-byte buffer built at the
256 default. Task 3b's M4 is the precedent for a mutation going red by a panic
rather than a mismatch; **none of these did** -- every one changed a value inside a bound the cores
already clamp, and every red reading is a stdout mismatch.

**What M2 cost and what it bought.** The falsified prediction is the reason the committed witness
carries four `say` lines it would not otherwise have had -- the sends `insert('QR', 9)`,
`insert('QR', 2, 5)`, `overlay('QR', 10)`, `overlay('QR', 2, 5)`, `replaceAt('QR', 10)` and
`~'[]='('QR', 10)` -- and the general shape is worth writing down: a default that is *reached* is
not a default that is *observed*, and a growth case that prints only `length` and `getBufferSize`
reaches the padding path while hiding the byte it pads with. (`$S/ctrl/predictions-m2b.md` calls
those "four sends" and then lists six; the file is left as it was written, since editing a
prediction after its run is what makes predictions worthless.)

## 4. `corpus/method-bodies.txt` row moves

**Predicted before the refresh ran** (`$S/ctrl/method-bodies-prediction.txt`). The probe sends
`say o~'NAME'()` to `.MutableBuffer~new('abc')`, so **a mutator that answers the receiver makes the
probe's own `say` reach `MAKESTRING`**, which this commit leaves unbound -- the row stays `loud`
and only its evidence changes, which is exactly what `delStr`'s row already reads.

| rows | from | to | evidence |
|---|---|---|---|
| `insert`, `overlay`, `changeStr`, `delWord` | `loud`, the method's own name | `answers` | `rc 163` -- 93.903 on both sides |
| `replaceAt`, `[]=` | `loud` | `answers` | `rc 168` -- 88.901 `argument new` on both sides |
| `upper`, `lower`, `translate`, `space`, `delete` | `loud`, the method's own name | `loud` | `method "MAKESTRING" of class "MutableBuffer"` -- the send succeeds and the `say` is what refuses |
| every other row, `MutableBuffer`'s others included | unchanged | unchanged | -- |

**Measured**: `REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies`
reads `regressions this run: 0. other drift from the committed table: 11.` (`$S/ctrl/fast/mb.err`),
and `git diff -- corpus/method-bodies.txt` is those eleven rows and no other. **No row moves to
`diverge`**, which is the gate. Prediction confirmed row for row, rc for rc.

## 5. Corrections to prose and tests

* **The brief's "`buf~string` is not available" is false.** Task 2 bound
  `("MutableBuffer", "STRING", Arity::Fixed(0), native_mutable_buffer_string)`,
  `corpus/method-bodies.txt` carries `MutableBuffer string instance answers rc 0` at BASE, and both
  `mutablebuffer_state.rex` and `mutablebuffer_readers.rex` read their contents back with it. The
  witness uses `buf~string`; `buf~substr(1, buf~length)` would also have worked. What is genuinely
  unavailable is `makeString`, and the witness contains neither it nor `say buf` nor any
  concatenation of a buffer.
* **`replaceAt` is not `overlay_bytes`'s neighbourhood**, which is how the brief's table describes
  it. Measured, oracle rc 0: `.MutableBuffer~new('abcdef')~replaceAt('XYZW', 3, 1)` is `abXYZWdef`,
  nine bytes -- the tail shifts right, where `overlay('XYZW', 3, 1)` would write one byte over it
  and answer `abXdef`. The body is `substr_bytes` for the
  padded front, then the replacement whole, then the tail from `begin + replaceLength`, which is
  `MutableBuffer::replaceAt`'s own steps (`:615`-`:633`).
* `corpus/refusal-sites.tsv` is again the one changed file no brief named, for the reason Task 3b
  recorded: `error.rs` rows are cited by line.
* `DELETE` is a second row over `mydelete`'s body as the brief says, but not literally over
  `native_mutable_buffer_delstr`: the two rows go through `buffer_delete`, which takes the name, so
  a receiver carrying no state gets `Loud::native_method` naming the spelling that was sent rather
  than the other one.

## 6. Files created outside the tree

* `$B/base/` -- `git archive 3c4585cb4 rust interpreter` extract; `$B/target-base/` -- its build.
* `$S/oracle/w/`, `$S/oracle/e/`, `$S/oracle/e2/`, `$S/oracle/e3/`, `$S/oracle/r/`,
  `$S/oracle/r2/`, `$S/oracle/r3/` -- the probe programs and their three descriptors, oracle and
  both engines; `$S/oracle/run.*/` -- the fresh empty run directories.
* `$S/probes/`, `$S/probes/base/`, `$S/probes/run.*/` -- the witness against this build and against
  BASE, and their run directories.
* `$S/run_oracle.sh`, `$S/run_crate.sh`, `$S/base-build.sh`, `$S/base-build.{out,err,pid,status}`,
  `$S/srclines/` (the `sourceline_oracle` driver and its stderr).
* `$S/{dispatch,error,coverage}.rs.base`, `$S/{phase-5c,method-bodies}.txt.base`,
  `$S/refusal-sites.tsv.base` -- copies at BASE.
* `$S/ctrl/` -- `predictions.md`, `predictions-m2b.md` and `method-bodies-prediction.txt` (each
  written before the runs it names), `rederive.py`, `release-sha.txt`, `fast/`, `c4/`, `c6/`.
* `$S/mut/` -- `mutate.py`, `run_all.sh`, `run_m2b.sh`, `status.txt`, `status-m2b.txt`, `pid`,
  `pid-m2b`, the `.good` restore copies, and `M1/` to `M6/` and `M2b/` with each mutation's checks.
* `$S/gates/` -- the gate runner, its status file and its pidfile.

Nothing under `$S` or `$B` was deleted.

## 7. Open questions

1. **`MutableBuffer::translate` numbers both its position and its range `ARG_FOUR`**
   (`classes/MutableBufferClass.cpp:1399`-`:1400`), where the range is argument five. It is
   unobservable -- 93.923 and 93.924 carry no argument number -- so this crate reproduces the
   behaviour and not the numbering, and nothing tests the difference. If a later ooRexx corrects it,
   nothing here changes.
2. **The named 88.910/88.911/88.912 family is reached through `replaceAt` and `[]=` alone today.**
   Whether any other class's methods use the named argument overloads is not surveyed here; the
   three `refusal-sites.tsv` rows carry `replaceAt` witnesses because that is where they were
   measured.
3. Carried from Task 3a §6.1 and Task 3b §7, unchanged: `MutableBuffer~verify` answers `counted` on
   every path while the builtin's past-the-end zero is an untagged text; and `refusal-sites.tsv`
   citing constructor definitions by line means every `error.rs` insertion re-derives every row
   below it -- 34 here.
4. `makeString`, `string`'s partner, `makeArray` and `subWords` stay unbound until the conversion
   commit, which is why five of this commit's eleven rows are still `loud`.

## Fast checks before the commit

Run from `rust/` in the foreground, each status read unpiped
(`$S/ctrl/fast/`).

| check | reading |
|---|---|
| `cargo fmt --all --check` | rc 0 (rc 1 first, `cargo fmt --all` applied, then rc 0) |
| `cargo clippy --workspace --all-targets -- -D warnings` | rc 0 |
| `cargo build --release -p rexx-exec --bin rexx-run` | rc 0, sha256 `062f3e5e868f643f…` |
| `cargo test --release -p rexx-exec --lib` | rc 0, 779 passed, 0 failed |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus` | rc 0, `361 of 361 matching`, 18 passed |
| `cargo test --release -p rexx-exec --test refusal_sites` | rc 101 before the re-derivation, rc 0 and 5 passed after |
| `cargo test --release -p rexx-exec --test coverage` | rc 0, 20 passed |
| `cargo test --release -p rexx-exec --test builtin_status` | rc 0, 19 passed |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | rc 0, 1 passed |
| `REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies` | rc 0, 16 passed, `regressions this run: 0. other drift from the committed table: 11.` |
| `cargo test --release -p rexx-exec --test method_bodies` (no refresh, after the run) | rc 0, 16 passed, `regressions this run: 0. other drift from the committed table: 0.` |

Every one of them was re-read in this order after the mutation run and the witness widening, from a
`target/release/rexx-run` rebuilt to the same sha256 `062f3e5e868f643f…` -- which is itself the
control that the mutation restores were byte-exact. After that, section 5's corrections to four
`dispatch.rs` doc blocks (C++ line citations, no code) were followed by `cargo fmt --all --check`
rc 0, `cargo clippy` rc 0, a release rebuild whose sha256 is again `062f3e5e868f643f…`, the STRICT
corpus at `361 of 361 matching` and `--test refusal_sites` at 5 passed. `git status --short` then
names six modified tracked files and three new ones, and `rust/Cargo.lock` is not among them.

## Gates

Run from `rust/` by `$S/gates/run.sh` in the background, each status written unpiped to
`$S/gates/status.txt` as it goes -- the commit sha as its first line, `finished` as its last -- with
the pidfile `$S/gates/pid` beside it and each gate's descriptors in `$S/gates/g<N>.{out,err}`.

| # | command | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **G1** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **G2** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **G3** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **G4** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **G5** |
| 6 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **G6** |
| 7 | `REXX_PHASE_GATE=5d REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **G7** |
