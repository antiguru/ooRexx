# Task 3 report: `builtin/string.rs`, the 22 remaining string builtins

**Status: DONE.** Commit `63a9ea9f504fe582f958132a27d8f07a1d7ea06c`, working tree clean.

Files changed (6):

| Path | What |
|---|---|
| `rust/crates/rexx-exec/src/builtin/string.rs` | the 22 implementations, the shared helpers, 23 unit tests |
| `rust/crates/rexx-exec/src/builtin/mod.rs` | 22 `IMPLEMENTED` rows, the `Run` type, `dispatch` passing the name |
| `rust/crates/rexx-exec/src/error.rs` | 7 new raisers: 40.12, 40.23, 93.906, 93.915, 93.923, 93.924, 5 |
| `rust/corpus/builtin-status.txt` | 22 rows `loud` -> `implemented` |
| `rust/corpus/keyword-exempt.txt` | 17 rows removed, header count and blocker list corrected |
| `rust/crates/rexx-exec/tests/builtin_status.rs` | `STRING_FAMILY` + `every_string_builtin_is_implemented` (Step 4) |

---

## 1. The verify block, exit statuses read unpiped

Run from `rust/`, final state, each `echo "rc=$?"` on the bare command:

| Command | rc | Result |
|---|---|---|
| `cargo test --offline --workspace --no-fail-fast` | **0** | 1055 passed, 0 failed (was 1039 before this task; +16 net = 23 new in `builtin::string` less the pre-existing `builtin` tests already counted, plus 1 in `builtin_status`) |
| `cargo fmt --all --check` | **0** | clean |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | **0** | clean; the run re-checked `rexx-exec` (`Checking rexx-exec` in the output), so it did lint the changed crate rather than reusing a cached result |
| `REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus` | **0** | `mode: STRICT (the gate)`, `42 of 42 matching` |

`cargo test -p rexx-exec --test builtin_status` (rc 0): 12 tests, including
`the_status_file_matches_a_live_differential_run` and the new
`every_string_builtin_is_implemented`.

---

## 2. The 40.12 / 40.23 substitution answer

**It is the argument's rendered value, and specifically the rendering fixed
when the value was created (D15) -- not the source spelling, and not a
re-rendering under the `DIGITS` in force at the call.**

The probe that settles it is one line in which all three differ:

```text
numeric digits 3 ; zz = 2 / 3 ; numeric digits 9 ; say left('ab', zz)
     1 *-* say left('ab', zz)
Error 40 running PROG line 1:  Incorrect call to routine.
Error 40.12:  LEFT argument 2 must be a whole number; found "0.667".
rc 216
```

* the source spelling is `zz`
* a re-rendering at the current `DIGITS 9` would be `0.666666667`
* `0.667` is the value's own rendering, captured by the division under
  `DIGITS 3` -- confirmed by `say zz` on the next line, which is `0.667`

The same value in a **pad** position gives the same answer under 40.23:

```text
numeric digits 3 ; zz = 2 / 3 ; numeric digits 9 ; say left('ab', 5, zz)
Error 40.23:  LEFT argument 3 must be a single character; found "0.667".   rc 216
```

Two corroborating pairs, both measured:

```text
zz = 'xy' ; say left('ab',5,zz)   ->  40.23 ... found "xy"    (value, not `zz`)
zz = 'qq' ; say left('ab',zz)     ->  40.12 ... found "qq"
say left('ab', 1.50)              ->  40.12 ... found "1.50"  (literal text is the value)
numeric digits 20 ; zz = 1/3 ; say left('ab',zz)
                                  ->  40.12 ... found "0.33333333333333333333"
```

Recorded in `Raised::argument_not_whole`'s own doc comment, with
`argument_not_a_pad` pointing at it.

**The 93.9xx family answers the opposite way**, which is why it is worth
stating: it reports the value *after* conversion to a whole number, not the
argument's text. Measured, all three spellings collapsing to one message:

```text
say left('ab','-1.0')   93.923  Invalid length argument specified; found "-1".   rc 163
say left('ab',' -1 ')   93.923  ... found "-1".
say left('ab','-1e0')   93.923  ... found "-1".
say substr('abc','0.0') 93.924  Invalid position argument specified; found "0".  rc 163
```

---

## 3. The probe table

Every probe was run from a fresh `mktemp -d` subdirectory of the scratchpad
with absolute paths, wrapped as
`( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib .../build/bin/rexx ABS )`,
with stdout, stderr and exit status read as three separate descriptors.

### 3.1 Hand-built probes (p1-p18): 473 programs, all also run through
`rexx-run` and compared on all three descriptors

Full transcripts below, grouped by builtin. `rc` is the oracle's; the Rust
executor matches on stdout, stderr and rc for every row except the three
marked **[other task]** and the one marked **[divergence]**.

#### CENTER / CENTRE (min 2, max 3)

| probe | rc | output |
|---|---|---|
| `say '['center('ab',6,'-')']'` | 0 | `[--ab--]` |
| `say '['center('ab',6)']'` | 0 | `[  ab  ]` |
| `say '['center('ab',5,'-')']'` | 0 | `[-ab--]` |
| `say '['center('abc',6,'-')']'` | 0 | `[-abc--]` |
| `say '['center('abcdef',3)']'` | 0 | `[bcd]` |
| `say '['center('abcdef',2)']'` | 0 | `[cd]` |
| `say '['center('abcdef',5)']'` | 0 | `[abcde]` |
| `say '['center('abcde',2)']'` | 0 | `[bc]` |
| `say '['center('abc',0)']'` | 0 | `[]` |
| `say '['center('abc',3)']'` | 0 | `[abc]` |
| `say '['center('',4,'*')']'` | 0 | `[****]` |
| `say '['center('The blue sky',8)']'` | 0 | `[e blue s]` |
| `say '['center('The blue sky',7)']'` | 0 | `[e blue ]` |
| `say '['center('ABC',8,'-')']'` | 0 | `[--ABC---]` |
| `say '['centre('ab',6,'-')']'` | 0 | `[--ab--]` |
| `say '['centre('ab',5,'-')']'` | 0 | `[-ab--]` |
| `say '['centre('abcdef',3)']'` | 0 | `[bcd]` |
| `say center('ab',6,'--')` | 216 | `40.23  CENTER argument 3 must be a single character; found "--".` |
| `say centre('ab',6,'--')` | 216 | `40.23  CENTRE argument 3 ...` **(the names differ)** |
| `say center('ab',-1)` | 163 | `93.923  Invalid length argument specified; found "-1".` |
| `say centre('ab',-1)` | 163 | `93.923  ... found "-1".` |
| `say center('ab',-1,'xx')` | 216 | `40.23  CENTER argument 3 ...` (call layer beats operation layer) |
| `say center('ab','x')` | 216 | `40.12  CENTER argument 2 must be a whole number; found "x".` |
| `say centre('ab','x')` | 216 | `40.12  CENTRE argument 2 ...` |
| `say center('ab')` | 216 | `40.3  ... minimum expected is 2.` |
| `say centre('ab')` | 216 | `40.3  ... invocation of CENTRE; minimum expected is 2.` |
| `say center('ab',6,'-','z')` | 216 | `40.4  ... maximum expected is 3.` |
| `say center(,6)` / `say centre(,6)` | 216 | `40.5  ... argument 1 is required.` |
| `say '['center('abcdef',,'.')']'` | 216 | `40.5  ... argument 2 is required.` |

#### LEFT / RIGHT (min 2, max 3)

| probe | rc | output |
|---|---|---|
| `say '['left('ab',5,'.')']'` | 0 | `[ab...]` |
| `say '['left('ab',5)']'` | 0 | `[ab   ]` |
| `say '['left('abcdef',3)']'` | 0 | `[abc]` |
| `say '['left('abc',0)']'` | 0 | `[]` |
| `say '['right('ab',5,'.')']'` | 0 | `[...ab]` |
| `say '['right('abcdef',3)']'` | 0 | `[def]` |
| `say '['right('abc',0)']'` | 0 | `[]` |
| `say left('ab',-1)` / `right('ab',-1)` | 163 | `93.923 ... found "-1".` |
| `say left('ab','q')` | 216 | `40.12  LEFT argument 2 ... found "q".` |
| `say right('ab','q')` | 216 | `40.12  RIGHT argument 2 ...` |
| `say left('ab',5,'..')` | 216 | `40.23  LEFT argument 3 ... found "..".` |
| `say '['left('',0,'xx')']'` | 216 | `40.23 ... found "xx".` (pad checked though unusable) |
| `say '['right('',0,'xx')']'` | 216 | `40.23 ... found "xx".` |
| `say left(,5)` / `say right(,5)` | 216 | `40.5 ... argument 1 is required.` |
| `say '['left('abcdef',,'.')']'` | 216 | `40.5 ... argument 2 is required.` |
| `say left('ab','-1.0')` / `' -1 '` / `'-1e0'` | 163 | `93.923 ... found "-1".` |
| `say left('ab','- 5')` / `'   - 5      '` | 163 | `93.923 ... found "-5".` (blank after the sign parses) |
| `say left('ab','   --5      ')` | 216 | `40.12 ... found "   --5      ".` |
| `say left('ab',' 5 ')` / `'+5'` | 0 | `ab   ` |
| `say left('ab','-0')` / `'-0.0'` | 0 | (empty) |
| `say left('ab','1e1')` / `'1E1'` | 0 | `ab` + 8 blanks |
| `say left('ab','1E18')` | 216 | `40.12 ... found "1E18".` (10^18 exceeds ARGUMENT_DIGITS) |
| `say left('ab','1000000000000000000')` | 216 | `40.12 ...` |
| `say left('ab','1234567890123456789')` | 216 | `40.12 ...` |
| `say left('ab','-1234567890123456789')` | 216 | `40.12 ...` |
| `say left('ab','999999999999999999')` | **251** | `Error 5 ... System resources exhausted.` (no sub line) |
| `numeric digits 2; say '['left('ab','1.0000001')']'` | 216 | `40.12 ... found "1.0000001".` |
| `numeric digits 30; say '['left('ab','1.0000000000000000000004')']'` | 0 | `[a]` |
| `say '['left('ab','1.0000000000000000000004')']'` | 0 | `[a]` |

The last three settle that the conversion uses `Numerics::ARGUMENT_DIGITS`
(18), **not** the current `NUMERIC DIGITS`, in both directions. One
subagent's summary claimed the opposite ("NUMERIC DIGITS sensitive, default
9"); the measurement wins and is what the code does.

#### SUBSTR (2, 4)

| probe | rc | output |
|---|---|---|
| `say '['substr('abcdef',2,3)']'` | 0 | `[bcd]` |
| `say '['substr('abcdef',2)']'` | 0 | `[bcdef]` |
| `say '['substr('abcdef',2,8)']'` | 0 | `[bcdef   ]` |
| `say '['substr('abcdef',2,8,'.')']'` | 0 | `[bcdef...]` |
| `say '['substr('abcdef',7)']'` | 0 | `[]` |
| `say '['substr('abcdef',7,3,'.')']'` | 0 | `[...]` |
| `say '['substr('abcdef',2,0)']'` | 0 | `[]` |
| `say '['substr('',1)']'` | 0 | `[]` |
| `say '['substr('abcdef',3,,'.')']'` | 0 | `[cdef]` (interior omission before a supplied pad) |
| `say substr('abc',0)` | 163 | `93.924  Invalid position argument specified; found "0".` |
| `say substr('abc',-1)` | 163 | `93.924 ... found "-1".` |
| `say substr('abc','-1.0')` | 163 | `93.924 ... found "-1".` |
| `say substr('abc','0.0')` | 163 | `93.924 ... found "0".` |
| `say substr('abc',2,-1)` | 163 | `93.923 ... found "-1".` |
| `say substr('abc',9,-1)` | 163 | `93.923 ... found "-1".` |
| `say substr('abc',,2)` | 216 | `40.5 ... argument 2 is required.` |
| `say substr(,2)` | 216 | `40.5 ... argument 1 is required.` |
| `say substr('abc',2,3,'pq')` | 216 | `40.23  SUBSTR argument 4 ... found "pq".` |
| `say substr('abc','x')` | 216 | `40.12  SUBSTR argument 2 ... found "x".` |
| `say substr('abc',0,5,'xx')` | 216 | **40.23**, not 93.924 -- the call layer runs first |
| `say substr('abc',0,5,'x')` | 163 | 93.924 -- the adjacent success, pinning the ordering |
| `say substr('abc','x',5,'yy')` | 216 | `40.12 argument 2` -- leftmost call-layer error wins |
| `say substr('abc',2,-1,'yy')` | 216 | `40.23 argument 4` |
| `say substr('abcdef', 1.0)` / `2e0` | 0 | `abcdef` / `bcdef` |
| `numeric digits 2; say '['substr('abcdef','2.0000001')']'` | 216 | `40.12 ... found "2.0000001".` |
| `say substr('ab',1,'123456789012345678')` | 251 | Error 5 |

#### DELSTR (1, 3)

| probe | rc | output |
|---|---|---|
| `say '['delstr('abcdef',3,2)']'` | 0 | `[abef]` |
| `say '['delstr('abcdef',3)']'` | 0 | `[ab]` |
| `say '['delstr('abcdef',1,6)']'` | 0 | `[]` |
| `say '['delstr('abcdef',9,2)']'` | 0 | `[abcdef]` |
| `say '['delstr('abcdef')']'` | 0 | `[]` (n defaults to 1, length to the rest) |
| `say '['delstr('abcdef',3,0)']'` | 0 | `[abcdef]` |
| `say '['delstr('abcdef',,2)']'` | 0 | `[cdef]` |
| `say '['delstr(123456,,0)']'` / `,,1` | 0 | `[123456]` / `[23456]` |
| `say delstr('abcdef',0)` | 163 | `93.924 ... found "0".` |
| `say delstr('abcdef',2,-1)` | 163 | `93.923 ... found "-1".` |
| `say delstr('abc',9,-1)` | 163 | `93.923` -- the length is checked before the out-of-range start returns |
| `say delstr('abc',9,'q')` | 216 | `40.12  DELSTR argument 3 ...` |
| `say delstr('abcdef','x')` | 216 | `40.12  DELSTR argument 2 ...` |
| `say delstr('abcdef',2,3,4)` | 216 | `40.4 ... maximum expected is 3.` |
| `say delstr(,2)` | 216 | `40.5 ... argument 1 is required.` |
| `say '['delstr(,)']'` | 216 | `40.3 ... minimum expected is 1.` (trailing omissions drop) |

#### INSERT (2, 5) -- `insert(new, target, n, length, pad)`

| probe | rc | output |
|---|---|---|
| `say '['insert('-','abc',1)']'` | 0 | `[a-bc]` |
| `say '['insert('-','abc')']'` | 0 | `[-abc]` (n defaults to **0**) |
| `say '['insert('-','abc',0)']'` | 0 | `[-abc]` (zero is legal here) |
| `say '['insert('-','abc',5)']'` | 0 | `[abc  -]` |
| `say '['insert('-','abc',5,,'.')']'` | 0 | `[abc..-]` |
| `say '['insert('XY','abc',1,4)']'` | 0 | `[aXY  bc]` |
| `say '['insert('XY','abc',1,4,'.')']'` | 0 | `[aXY..bc]` |
| `say '['insert('XY','abc',1,1)']'` | 0 | `[aXbc]` |
| `say '['insert('XY','abc',1,0)']'` | 0 | `[abc]` |
| `say '['insert('','abc',2)']'` | 0 | `[abc]` |
| `say '['insert('XY','',2,,'.')']'` | 0 | `[..XY]` |
| `say '['insert('XY','abc',3)']'` | 0 | `[abcXY]` |
| `say '['insert('123','abc',5,6,'+')']'` | 0 | `[abc++123+++]` |
| `say '['insert('','',3,0)']'` / `,3,1` | 0 | `[   ]` / `[    ]` |
| `say '['insert('A','hasdghj',1,3)']'` | 0 | `[hA  asdghj]` |
| `say '['insert('A','hasdghj',1,9,3)']'` | 0 | `[hA33333333asdghj]` |
| `say '['insert('-','abc',,2)']'` | 0 | `[- abc]` |
| `say insert('-','abc',-1)` | 163 | **93.906**  `Method argument 2 must be zero or a positive whole number; found "-1".` |
| `say insert('a','b','-1.0')` | 163 | `93.906  Method argument 2 ... found "-1".` |
| `say insert('-','abc',1,-1)` | 163 | `93.923 ... found "-1".` |
| `say insert('-','abc','q')` | 216 | `40.12  INSERT argument 3 ...` |
| `say insert('-','abc',-1,'q')` | 216 | `40.12  INSERT argument 4 ...` (call layer first) |
| `say insert('-','abc',1,2,'pq')` | 216 | `40.23  INSERT argument 5 ...` |
| `say insert('-')` | 216 | `40.3 ... minimum expected is 2.` |
| `say insert('-','abc',1,2,'.',9)` | 216 | `40.4 ... maximum expected is 5.` |
| `say insert(,'abc',1)` | 216 | `40.5 ... argument 1 is required.` |
| `say insert('a','b',1,'123456789012345678')` | 251 | Error 5 |

#### OVERLAY (2, 5) -- `overlay(new, target, n, length, pad)`

| probe | rc | output |
|---|---|---|
| `say '['overlay('XY','abcdef',3)']'` | 0 | `[abXYef]` |
| `say '['overlay('XY','abcdef')']'` | 0 | `[XYcdef]` (n defaults to **1**) |
| `say '['overlay('XY','abcdef',3,4)']'` | 0 | `[abXY  ]` |
| `say '['overlay('XY','abcdef',3,4,'.')']'` | 0 | `[abXY..]` |
| `say '['overlay('XY','abcdef',3,1)']'` | 0 | `[abXdef]` |
| `say '['overlay('XY','abcdef',8)']'` | 0 | `[abcdef XY]` |
| `say '['overlay('XY','abcdef',8,,'.')']'` | 0 | `[abcdef.XY]` |
| `say '['overlay('XY','abc',3,0)']'` | 0 | `[abc]` |
| `say '['overlay('XY','abc',4,0)']'` | 0 | `[abc]` |
| `say '['overlay('XY','abc',5,0)']'` | 0 | `[abc ]` (zero length still extends) |
| `say '['overlay('','abcdef',3)']'` | 0 | `[abcdef]` |
| `say '['overlay('XY','',2,,'.')']'` | 0 | `[.XY]` |
| `say '['overlay('','abc',6,1)']'` | 0 | `[abc   ]` |
| `say '['overlay('','abc',3,1)']'` | 0 | `[ab ]` |
| `say '['overlay('34','abc',3,1)']'` | 0 | `[ab3]` |
| `say '['overlay('.','abcdef',3,2)']'` | 0 | `[ab. ef]` |
| `say '['overlay('qq','abcd',4)']'` | 0 | `[abcqq]` |
| `say '['overlay('XY','abcdef',,3)']'` | 0 | `[XY def]` |
| `say overlay('XY','abcdef',0)` | 163 | `93.924 ... found "0".` (contrast INSERT) |
| `say overlay('XY','abcdef',3,-1)` | 163 | `93.923 ... found "-1".` |
| `say overlay('XY','abcdef','q')` | 216 | `40.12  OVERLAY argument 3 ...` |
| `say overlay('XY','abcdef',3,2,'pq')` | 216 | `40.23  OVERLAY argument 5 ...` |
| `say overlay('XY')` | 216 | `40.3 ... minimum expected is 2.` |
| `say overlay(,'abcdef',1)` | 216 | `40.5 ... argument 1 is required.` |
| `say overlay('a','b',1,'123456789012345678')` | 251 | Error 5 |

#### POS (2, 4) and LASTPOS (2, 4)

| probe | rc | output |
|---|---|---|
| `say pos('an','banana')` | 0 | `2` |
| `say pos('an','banana',3)` | 0 | `4` |
| `say pos('an','banana',5)` | 0 | `0` |
| `say pos('an','banana',1,3)` / `,1,2` | 0 | `2` / `0` |
| `say pos('','banana')` / `pos('a','')` | 0 | `0` / `0` |
| `say pos('a','banana',9)` / `,2,0` | 0 | `0` / `0` |
| `say pos('banana','an')` | 0 | `0` |
| `say pos('a','banana',,2)` | 0 | `2` |
| `say pos('a','BANANA')` | 0 | `0` (case-sensitive) |
| `say pos('e','abcdeeeeef',2,4)` / `pos('eee',...,5,2)` | 0 | `5` / `0` |
| `say pos('a','banana',0)` | 163 | `93.924 ... found "0".` |
| `say pos('a','banana','-1.0')` | 163 | `93.924 ... found "-1".` |
| `say pos('a','banana',1,-1)` | 163 | `93.923 ... found "-1".` |
| `say pos('a','banana','q')` | 216 | `40.12  POS argument 3 ...` |
| `say pos('a')` | 216 | `40.3 ... minimum expected is 2.` |
| `say pos('a','banana',1,2,3)` | 216 | `40.4 ... maximum expected is 4.` |
| `say pos(,'banana')` | 216 | `40.5 ... argument 1 is required.` |
| `say lastpos('an','banana')` | 0 | `4` |
| `say lastpos('an','banana',4)` / `,3` | 0 | `2` / `2` |
| `say lastpos('a','banana',3)` | 0 | `2` |
| `say lastpos('an','banana',6,2)` / `,6,3` | 0 | `0` / `4` |
| `say lastpos('abc','xxabc',5)` / `,4` / `,3` | 0 | `3` / `0` / `0` |
| `say lastpos('a','aaaaabcdef',9,5)` / `,10,5` | 0 | `5` / `0` |
| `say lastpos('aaa','aaaaabcdef',5,2)` | 0 | `0` |
| `say lastpos('b','banana',5)` | 0 | **`1`** -- the range default is the whole haystack |
| `say lastpos('b','banana',5,6)` / `,5,2` | 0 | `1` / `0` |
| `say lastpos('','banana')` / `lastpos('a','')` | 0 | `0` / `0` |
| `say lastpos('a','banana',99)` / `,,2` | 0 | `6` / `6` |
| `say lastpos('A','banana')` | 0 | `0` |
| `say lastpos('a','banana',0)` | 163 | `93.924 ... found "0".` |
| `say lastpos('a','banana',1,-1)` | 163 | `93.923 ... found "-1".` |
| `say lastpos('a','banana','q')` | 216 | `40.12  LASTPOS argument 3 ...` |
| `say lastpos(,'banana')` | 216 | `40.5 ... argument 1 is required.` |

`lastpos('b','banana',5)` is the probe that discriminates the range default:
whole-haystack gives 1, `len - start + 1` would give 0. The oracle answers 1.

#### REVERSE (1, 1)

`reverse('abcdef')` -> `fedcba`; `reverse('')` -> ``; `reverse('a')` -> `a`;
`reverse()` -> 40.3 min 1; `reverse('a','b')` -> 40.4 max 1;
`reverse(,)` -> 40.3 (trailing omissions drop).

#### STRIP (1, 3)

| probe | rc | output |
|---|---|---|
| `say '['strip('  ab  ')']'` | 0 | `[ab]` |
| `say '['strip('  ab  ','L')']'` / `'T'` / `'B'` | 0 | `[ab  ]` / `[  ab]` / `[ab]` |
| `say '['strip('  ab  ','l')']'` | 0 | `[ab  ]` (case-insensitive) |
| `say '['strip('  ab  ','Leading')']'` | 0 | `[ab  ]` (only the first letter) |
| `say strip(778877,'Bla',4+3)` | 0 | `88` |
| `say '['strip('xxabxx',,'x')']'` | 0 | `[ab]` |
| `say '['strip('xxabxx','L','x')']'` | 0 | `[abxx]` |
| `say '['strip('xyabyx',,'xy')']'` | 0 | `[ab]` (a set, not a pad) |
| `say '['strip('+-+-a-+b-+-+','L','-+')']'` | 0 | `[a-+b-+-+]` |
| `say '['strip('abc','B','')']'` | 0 | `[abc]` (empty set strips nothing) |
| `say '['strip('   ')']'` / `strip('')` | 0 | `[]` / `[]` |
| `say '['strip('..ab..','B','.')']'` | 0 | `[ab]` |
| `say '['strip('a'\|\|'09'x\|\|'b'\|\|'09'x)']'` | 0 | `[a<TAB>b]` (tab is in the default set) |
| `say '['strip('a'\|\|'0a'x)']'` | 0 | `[a<LF>]` (newline is not) |
| `say '['strip('  ab  ',,' ')']'` | 0 | `[ab]` |
| `say strip('ab','X')` | 163 | `93.915  Method option must be one of "BLT"; found "X".` |
| `say strip('ab','Xyz')` | 163 | `93.915 ... found "Xyz".` (the whole string, not the letter) |
| `say strip('ab','')` | 163 | `93.915 ... found "".` |
| `say strip('ab','B','xy')` | 0 | `ab` (arg 3 is never pad-checked) |
| `say strip('ab','B','x','y')` | 216 | `40.4 ... maximum expected is 3.` |
| `say strip(,'L')` | 216 | `40.5 ... argument 1 is required.` |

#### SPACE (1, 3)

| probe | rc | output |
|---|---|---|
| `say '['space('a   b  c')']'` | 0 | `[a b c]` |
| `say '['space('a   b  c',2)']'` / `,0` | 0 | `[a  b  c]` / `[abc]` |
| `say '['space('a   b  c',2,'-')']'` | 0 | `[a--b--c]` |
| `say '['space('   ')']'` / `space('')` | 0 | `[]` / `[]` |
| `say '['space('  a  ')']'` | 0 | `[a]` |
| `say '['space('a'\|\|'09'x\|\|'b')']'` | 0 | `[a b]` (tab separates) |
| `say '['space('a'\|\|'0a'x\|\|'b')']'` | 0 | `[a<LF>b]` (newline does not) |
| `say '['space('a  b',,'-')']'` | 0 | `[a-b]` |
| `say '['space(' ','30'x)']'` | 0 | `[]` |
| `say space('ab',-1)` | 163 | `93.923 ... found "-1".` |
| `say space('ab','q')` | 216 | `40.12  SPACE argument 2 ...` |
| `say space('ab',1,'..')` / `,1,''` | 216 | `40.23  SPACE argument 3 ... found ".."` / `found ""` |
| `say space('ab',-1,'xx')` | 216 | `40.23` (call layer first) |
| `say space('ab',1,'.','x')` | 216 | `40.4 ... maximum expected is 3.` |
| `say space(,2)` | 216 | `40.5 ... argument 1 is required.` |
| `say space('a b','123456789012345678')` | 251 | Error 5 |

#### COPIES (2, 2)

`copies('ab',3)` -> `ababab`; `copies('ab',0)` -> ``; `copies('',5)` -> ``;
`copies('ab',1)` -> `ab`; `copies('ab',' -0 ')` -> `` (negative zero is fine).
`copies('ab',-1)` -> **93.906** `Method argument 1 must be zero or a positive
whole number; found "-1".` rc 163; `copies('ab','-1.0')` -> the same message,
`found "-1"`. `copies('ab','q')` and `copies('ab','')` -> 40.12 argument 2 (the
second with `found ""`). `copies('ab')` and `copies('ab',)` -> 40.3 min 2.
`copies('ab',2,3)` -> 40.4 max 2. `copies(,3)` -> 40.5 argument 1.
`copies('ab','123456789012345678')` -> 251, Error 5.

#### ABBREV (2, 3)

`abbrev('Print','Pri')` 1; `'Pro'` 0; `('Print','')` 1; `('Print','',0)` 1;
`('Print','',1)` 0; `('Print','Pri',4)` 0; `,3` 1; `,0` 1; `('','')` 1;
`('','x')` 0; `('Print','PRI')` 0 (case-sensitive); `('Print','Print')` 1;
`('Print','Printer')` 0; `abbrev('ab','a','123456789012345678')` 0 (a huge
length is legal, just false); `abbrev('Print','Pri',)` 1.
`abbrev('Print','Pri',-1)` -> 93.923 `found "-1"`; `,'q'` -> 40.12 argument 3;
`abbrev('Print')` -> 40.3 min 2; `,3,4` -> 40.4 max 3;
`abbrev('Print',,3)` -> 40.5 argument 2.
`say datatype(abbrev('a','a'))` -> `NUM` and `abbrev('a','a') + 1` -> `2`, so
the result is the text `1`/`0`, not an object. **[other task: DATATYPE]** --
that one probe is loud under `rexx-run`.

#### COMPARE (2, 3)

`('abcde','abcde')` 0; `('abcde','abXde')` 3; `('abc','abc ')` 0;
`('abc','abc.')` 4; `('abc','abc.','.')` 0; `('','')` 0; `('','a')` 1;
`('abc','ab')` 3; `('ab','abc')` 3; `('abc','ab','c')` 0; `('abc','abd',)` 3.
`compare('abc','ab','..')` -> 40.23 argument 3; `compare('abc')` -> 40.3 min 2;
`,'c','d'` -> 40.4 max 3; `compare('abc',,'c')` -> 40.5 argument 2.

#### COUNTSTR (2, 2)

`('a','banana')` 3; `('an','banana')` 2; `('aa','aaaa')` **2** (non-overlapping);
`('','abc')` 0; `('abc','')` 0; `('x','banana')` 0.
`countstr('a')` -> 40.3 min 2; `,'x'` -> 40.4 max 2; `countstr(,'banana')` ->
40.5 argument 1.

#### CHANGESTR (3, 4)

`('a','banana','X')` `bXnXnX`; `,2` `bXnXna`; `,0` `banana`;
`('a','banana','')` `bnn`; `('','banana','X')` `banana`;
`('an','banana','ANA')` `bANAANAa`; `('z','banana','X')` `banana`;
`('a','','X')` (empty); `('a','banana','X',)` `bXnXnX`.
`changestr('a','banana','X',-1)` -> **93.906** `Method argument 3 ... found
"-1".`; `,'q'` -> 40.12 argument 4; `('a','banana')` -> 40.3 min 3;
`,1,2` -> 40.4 max 4; `changestr('a',,'X')` -> 40.5 argument 2;
`changestr('a','banana',)` -> 40.3 min 3 (trailing omission drops).

#### TRANSLATE (1, 6)

| probe | rc | output |
|---|---|---|
| `say '['translate('abcdef')']'` | 0 | `[ABCDEF]` (no tables, no pad = UPPER) |
| `say '['translate('abcdef','123','abc')']'` | 0 | `[123def]` |
| `say '['translate('abcdef','123')']'` | 0 | `[      ]` (omitted tablein = identity index) |
| `say '['translate('abcdef',,'abc')']'` | 0 | `[   def]` |
| `say '['translate('abcdef','12','abcd')']'` | 0 | `[12  ef]` |
| `say '['translate('abcdef','1234','ab')']'` | 0 | `[12cdef]` |
| `say '['translate('abcdef','12','abcd','.')']'` | 0 | `[12..ef]` |
| `say '['translate('abcdef',,,'.')']'` | 0 | `[......]` |
| `say '['translate('abcdef','','')']'` | 0 | `[abcdef]` (a **supplied** empty tablein finds nothing) |
| `say '['translate('abcABC','123','abc')']'` | 0 | `[123ABC]` |
| `say '['translate('abcdef','X','a',,2)']'` | 0 | `[abcdef]` |
| `say '['translate('abcdef','XY','af',,2,3)']'` | 0 | `[abcdef]` |
| `say '['translate('abcdef',,,,2,3)']'` | 0 | `[aBCDef]` |
| `say '['translate('aXbXc','12','XX')']'` | 0 | `[a1b1c]` (first occurrence wins) |
| `say '['translate('4123','abcd','1234')']'` | 0 | `[dabc]` |
| `say '['translate('abcdef','123456','aaabbbcc','.')']'` | 0 | `[14.def]` |
| `say '['translate('abcdef','123456','aaabbbcc','.',2,3)']'` | 0 | `[a4.def]` |
| `say '['translate('APQRV', ,'PR')']'` | 0 | `[A Q V]` |
| `say translate('abc','1','a',,9)` / `,,1,0` | 0 | `abc` / `abc` |
| `say translate('abcdef ',,,'$',2,0)` | 0 | `abcdef ` (range 0 = unchanged) |
| `say '['translate('61'x)']'` | 0 | `[A]` |
| `say '['translate('e9'x)']'` | 0 | `` `e9`x `` unchanged (ASCII-only fold) |
| `say translate('abc','12','ab','..')` | 216 | `40.23  TRANSLATE argument 4 ...` |
| `say translate('abc',,,'$$',0)` | 216 | `40.23` (before the zero start) |
| `say translate('abc','12','ab','.','q')` | 216 | `40.12  TRANSLATE argument 5 ...` |
| `say translate('abc',,,'$',0,'q')` | 216 | `40.12 argument 6` (before the zero start) |
| `say translate('abc',,,'$',0,1)` | 163 | `93.924 ... found "0".` -- the adjacent success |
| `say translate('abc','12','ab','.',0)` | 163 | `93.924 ... found "0".` |
| `say translate('abc','12','ab','.',1,-1)` | 163 | `93.923 ... found "-1".` |
| `say translate('abc','1','a',,'-1.0')` | 163 | `93.924 ... found "-1".` |
| `say translate('abc','a','b','c','1','2','3')` | 216 | `40.4 ... maximum expected is 6.` |
| `say translate(,'12','ab')` | 216 | `40.5 ... argument 1 is required.` |
| `say '['translate('APQRV',xrange('00'x,'Q'))']'` | 0 | `[APQ  ]` **[other task: XRANGE]** |

#### VERIFY (2, 5)

| probe | rc | output |
|---|---|---|
| `say verify('abcde','abc')` | 0 | `4` |
| `say verify('abcde','abcde')` | 0 | `0` |
| `say verify('abcde','abc','M')` / `'m'` | 0 | `1` / `1` |
| `say verify('abcde','xyz','M')` | 0 | `0` |
| `say verify('abcde','abc','N')` / `'Nope'` | 0 | `4` / `4` |
| `say verify('abcde','')` / `('abcde','','M')` | 0 | `1` / `0` |
| `say verify('','abc')` | 0 | `0` |
| `say verify('abcde','abc','N',3)` | 0 | `4` |
| `say verify('abcde','ab','N',1,2)` | 0 | `0` |
| `say verify('abcde','abc',,4)` | 0 | `4` |
| `say verify('abcde','abc','N',9)` / `,1,0` | 0 | `0` / `0` |
| `say verify('abc','',,,0)` / `,,2,0` | 0 | `1` / `2` (empty reference under N ignores the range) |
| `say verify('abc','x',,,0)` / `,,2,0` | 0 | `0` / `0` (the contrast) |
| `say verify('abc','','m')` / `'n'` | 0 | `0` / `1` |
| `say verify('abc','','m',4)` / `'n',4` | 0 | `0` / `0` |
| `say verify('','',,,3)` | 0 | `0` |
| `say verify('ABCDEF','ABC',,2,3)` | 0 | `4` (absolute, not relative) |
| `say verify('ABCDEF','DEF','M',,3)` | 0 | `0` |
| `say verify('ABCDEF','ADEF','M',2,3)` | 0 | `4` |
| `say verify('abcde','abc','X')` | 163 | `93.915  Method option must be one of "MN"; found "X".` |
| `say verify('a','b','Xyz')` | 163 | `93.915 ... found "Xyz".` |
| `say verify('abcde','abc','')` | 163 | `93.915 ... found "".` |
| `say verify('a','b','X','q')` | 216 | `40.12 argument 4` -- before the bad option |
| `say verify('a','b','X','1')` | 163 | `93.915` -- the adjacent success |
| `say verify('a','b','X',0)` / `,1,-1` | 163 | `93.915` (the option is checked first) |
| `say verify('abc','a','N','-1.0')` | 163 | `93.924 ... found "-1".` |
| `say verify(,'abc')` | 216 | `40.5 ... argument 1 is required.` |

#### LOWER / UPPER (1, 3)

`lower('ABCdef')` `abcdef`; `lower('ABCDEF',3)` `ABcdef`; `,3,2` `ABcdEF`;
`,9` `ABCDEF` (start past the end is a no-op); `,3,0` `ABCDEF`; `,3,99`
`ABcdef`; `lower('')` ``; `lower('ABC',,2)` `abC`.
`upper('abcDEF')` `ABCDEF`; `upper('abcdef',3)` `abCDEF`; `,3,2` `abCDef`;
`,9` unchanged; `,3,0` unchanged; `upper('abc',,2)` `ABc`.
`upper('e9'x)` and `lower('c9'x)` unchanged -- ASCII-only folding.
`lower('ABCDEF',0)` -> 93.924 `found "0"`; `,1,-1` -> 93.923; `,'q'` -> 40.12
argument 2; `lower()` -> 40.3 min 1; `lower('a',1,1,1)` -> 40.4 max 3;
`lower(,2)` -> 40.5 argument 1.

#### D15 -- results are text, not numbers

```text
numeric digits 1 ; say pos('a','bbbbbbbbba')                 -> 10
numeric digits 1 ; say lastpos('a','bbbbbbbbba')             -> 10
numeric digits 1 ; say compare('bbbbbbbbba','bbbbbbbbbz')    -> 10
numeric digits 1 ; say countstr('a','aaaaaaaaaa')            -> 10
numeric digits 1 ; say verify('bbbbbbbbba','b')              -> 10
numeric digits 1 ; n = pos('a','bbbbbbbbba') ; say n ; say n + 0
                                                             -> 10 then 1E+1
numeric digits 1 ; say abbrev('abc','abc')                   -> 1
```

A value carrying `DIGITS 1` as its created pair would render `1E+1`, so the
results are created as text. The last line of the sixth probe is D15 from the
other side: the addition is a new operation, creating a new number under the
digits then in force. Each of these is a unit test
(`a_counting_builtin_answers_text_that_no_digits_setting_reshapes`).

### 3.2 Generated differential sweeps

Beyond the hand probes, two generated corpora were run through **both**
interpreters, one program per fresh directory, comparing stdout, stderr and
exit status separately (`scratchpad/bin/worker.sh`, 24-way parallel):

| Corpus | Programs | Mismatches |
|---|---|---|
| Value sweep -- all 23 builtins across a cross product of strings, whole and non-whole numeric arguments, pads, option letters, omitted interior positions | **62,144** | **0** |
| Error sweep -- negative / zero / non-numeric / oversized numeric arguments, multi-character and empty pads, invalid option letters, plus every arity from 0 to max+2 and every interior-omission position for all 23 | **3,105** | **0** |

**Negative control, so the zero means something.** Three seeded mutations
(CENTER's odd pad going left; COMPARE's trailing-mismatch offset off by one;
VERIFY's empty-reference answer ignoring the start) produced **232**
mismatches across the value sweep -- 44 CENTER, 44 CENTRE, 96 COMPARE, 48
VERIFY. The known TRANSLATE divergence below is also reported by the harness,
which is a second, independent demonstration that it can fail.

**Mutation testing of the committed unit suite**, applied one at a time and
each restored from a copy (never `git checkout`):

| Mutation | Unit suite |
|---|---|
| `ARGUMENT_DIGITS` 18 -> 9 | RED |
| `DEFAULT_STRIP_SET` drops the tab | RED |
| COMPARE's trailing offset off by one | RED |
| VERIFY's empty-reference answer ignores the start | RED |
| LASTPOS's window becomes the whole prefix | RED |
| COUNTSTR allows overlapping matches | RED |
| CENTER's odd pad goes left | **GREEN -> fixed** |
| CENTER's odd truncation comes off the left | **GREEN -> fixed** |

The last two were a real coverage gap found by this step: every CENTER case I
had written used an even padding width, which cannot tell the two sides apart.
Six odd-width cases were added and both mutations now go red. The doc comment
that had asserted the behaviour was also *inferred* rather than measured; it
is now measured (`center('ab',5,'-')` is `-ab--`, `center('abcdef',5)` is
`abcde`) and reworded.

---

## 4. `keyword-exempt.txt`: which rows moved and why

Seventeen rows were **removed** (never re-attributed), because the bodies now
pass:

```
ASSIGNMENT::test_2
NUMERIC::test_1  test_2  test_3  test_4  test_5  test_6  test_7
NUMERIC::test_8  test_9  test_10 test_11 test_12 test_13 test_14
SELECT::test_14
VarRef::test_unknown
```

The `NUMERIC` block is the bulk of it: those bodies were blocked on `COPIES`
and `SUBSTR`, which this task lands. `the_exempt_set_matches_the_current_failures`
named each one ("now PASSES but is still on the committed exempt list") and is
green again with 778 failing bodies against 778 committed rows.

Two header corrections in the same file, both required because this change
falsified them:

* `4c  789 bodies` -> `4c  772 bodies` (789 - 17).
* the illustrative blocker list `ARG/COPIES/DIGITS/FORM/FUZZ/SUBSTR/SIGN/...`
  named COPIES and SUBSTR, which are no longer blockers. They are removed, and
  a sentence added saying that which builtins are still blockers is the derived
  half of the file and moves as they land -- so the sentence cannot rot the same
  way again.

---

## 5. Things worth flagging

### 5.1 A real divergence, deliberate and documented

**`TRANSLATE` treats a null string differently depending on where it came
from, and this crate cannot.** The oracle decides whether `tablein` was
supplied by comparing the argument's *address* against its own null-string
singleton (`tablei != GlobalNames::NULLSTRING`, `RexxString::translate`), and
several builtins return that singleton rather than a fresh empty string.
Measured:

```text
zz = ''            ; say '['translate('abcdef','123',zz)']'   ->  [abcdef]
zz = left('abc',0) ; say '['translate('abcdef','123',zz)']'   ->  [      ]
zz = copies('a',0) ; say '['translate('abcdef','123',zz)']'   ->  [      ]
```

This crate answers `[abcdef]` for all three. Reproducing the oracle would mean
giving a string's *identity* observable meaning, which nothing else in Rexx
has, and no rule in the value model could motivate. It is recorded in
`translate`'s own doc comment with the exact reproducer. It is **not** a
`divergent` status row: `builtin-status.txt`'s TRANSLATE probe is
`say translate('abcdef','123','abc')`, which matches, and nothing in the
committed harnesses reaches the singleton path. If a future task wants this
policed, it needs its own corpus program and a `KNOWN GAP` row.

### 5.2 One interface change the brief could not anticipate

`Builtin::run` gained the row's own name as a second parameter
(`type Run = fn(&mut Interp, &'static [u8], &[Option<ObjRef>]) -> ...`), and
`dispatch` passes `builtin.name`. This was forced by the ambiguity the
controller resolved in advance: CENTER and CENTRE must be one implementation,
and measured, their 40.12/40.23 messages name themselves differently
(`CENTRE argument 3 ...` vs `CENTER argument 3 ...`). Without the name as a
parameter, either they are two functions or the messages are wrong.

It also removes a drift hazard for the other 21: an implementation naming
itself would be a second copy of the string in its table row, free to disagree
with it. `LENGTH`'s body and doc are untouched; only its signature gained
`_name`.

`tests/builtin_status.rs` also gained `STRING_FAMILY` and
`every_string_builtin_is_implemented` -- Step 4's "assert the family's list
specifically". Neither file was in the brief's list; both changes are
mechanically required by steps the brief does name.

### 5.3 An allocation guard the brief did not mention

The oracle raises `Error 5` at **rc 251** ("System resources exhausted.", no
sub line) when a result is too large to allocate, measured for `left`, `right`,
`center`, `space`, `substr`, `copies`, `insert` and `overlay`. An unguarded
`Vec` of that size in Rust *aborts the process*, which is strictly worse than
any divergence, so every builtin whose result length comes from an argument
allocates through `Vec::try_reserve_exact` and maps the refusal to
`Raised::system_resources` (`Raised::syntax(5, 0, vec![])`, which
`Raised::report` already renders without a sub line, at `256 - 5 = 251`).

This reproduces the *mechanism* -- ask the allocator, take its answer -- rather
than an invented size threshold, which is a number this project cannot measure.
The boundary is exact where the argument conversion decides it:
`left('ab','999999999999999999')` is Error 5 (18 digits, the largest
`ARGUMENT_DIGITS` admits) and `left('ab','1234567890123456789')` is 40.12
(19 digits, never reaching the allocator). Both are unit tests. Products that
would overflow `usize` (`COPIES`, `INSERT`, `SPACE`) use `checked_mul` /
`checked_add` into the same condition; the C++ overflows silently there, which
is a bug I did not reproduce.

### 5.4 A subagent claim that the measurement overturned

One testGroup-extraction subagent reported that "the whole-number check is
`NUMERIC DIGITS` sensitive (default 9)". It is not: the oracle converts through
`Numerics::ARGUMENT_DIGITS`, fixed at 18 on a 64-bit build. Measured in both
directions --

```text
numeric digits 2  ; say left('ab','1.0000001')                40.12
numeric digits 30 ; say left('ab','1.0000000000000000000004') a
```

-- a two-digit conversion would have accepted the first, and a thirty-digit one
would have rejected the second. The subagent's *expectations* were still
compatible with its wrong explanation (the ooTest case it cited, `1.000000009`,
is non-whole at both 9 and 18 digits), which is exactly why the explanation
survived. Recorded in `Raised::argument_not_whole`'s doc.

### 5.5 Nothing the brief got wrong

Every value in the brief's "Shared facts" block reproduced exactly, including
the three-way 40.3 / 40.5 / 93.923 split, the trailing-versus-interior omission
rule, and the warning that `check_arity` counts rather than checks shape. On
the last point: **none of these 22 has `DATE`'s conditional shape.** Probed
with an interior omission before every optional position --
`substr('abcdef',3,,'.')`, `verify('abcde','abc',,,2)`,
`translate('abcdef',,'abc',,2)`, `insert('-','abc',,2)`,
`overlay('XY','abcdef',,3)`, `lower('ABCDEF',,3)`, `pos('a','banana',,2)`,
`lastpos('a','banana',,2)`, `delstr('abcdef',,2)`, `space('a  b',,'-')`,
`strip('  ab  ',,' ')`, `changestr('a','banana','X',)`,
`abbrev('Print','Pri',)`, `compare('abc','abd',)` -- every one succeeds with
that position's default. The error sweep then covered every interior-omission
position of every one of the 23 systematically, with 0 mismatches.

### 5.6 Not done

* The `translate` singleton divergence (5.1) is not implemented.
* `corpus/lang/string_builtins.rex` was left out of any phase subset: it also
  calls `word`, `words`, `c2x`, `x2c`, `d2x` and `x2d`, which other family
  tasks own. Adding it is theirs, not mine.
* Three hand probes are still loud under `rexx-run` because they call builtins
  other tasks own (`DATATYPE`, `C2X`, `XRANGE`), and one
  (`say '['left('abc',5,'ff'x)']'x`, a deliberate 15.3 hex-literal trap) is a
  pre-existing parse-report gap unrelated to this task. All four are listed
  above; none is a string-builtin divergence.

---

## 6. Commit

```
63a9ea9f504fe582f958132a27d8f07a1d7ea06c  Implement the 22 remaining string builtins
```

Read back with `git log -1 --format='%H %s'`. `git status --short` is empty.

---
---

# Fix round 1

Commit `c0889d7428966cbc95d46dc8e3f36cfd6531a948`, working tree clean. Both
Criticals reproduced independently before anything was changed.

## Verify block, each status read unpiped

| Command | Exit | Result |
|---|---|---|
| `cargo test --offline --workspace --no-fail-fast` | **0** | 1058 passed, 0 failed (was 1055; +3 new tests) |
| `cargo fmt --all --check` | **0** | clean |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | **0** | clean, and the run re-checked `rexx-exec` |
| `REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus` | **0** | `mode: STRICT (the gate)`, `42 of 42 matching` |

---

## F2 (Critical) -- substitutions are bytes now, and the display rule is the oracle's

### Reproduced first

```text
say copies('ab','FF'x)
  oracle rc 216, stderr ends:  ... found " 377 " .          (od -c)
  ours   rc 216, stderr ends:  ... found " 357 277 275 " .
```

### The rule, measured rather than assumed

The reviewer gave the rule; I measured it independently by driving **all 256
byte values** through the message path -- `say copies('ab','NN'x)` for the 246
that are not valid counts, and `say left('ab',5,'NNNN'x)` for the ten ASCII
digits, which are:

```text
rendered as ? (0x3f) :  00-08  0b-0c  0e-1f
rendered raw         :  09-0a  0d     20-ff       (0x7f and all of 0x80-0xff raw)
```

That is exactly `< 0x20 && not TAB/CR/LF`, and it is `RexxString::stringTrace`
(`classes/StringClass.cpp:1638`).

### Where the oracle applies it, which changed where I applied it

Not to the substitution value: `Activity::messageSubstitution`
(`concurrency/Activity.cpp:1246`) inserts the value verbatim. The rule is
applied to **whole output lines** by `Activity::display`
(`concurrency/Activity.cpp:1414`), which sends every traceback echo, the major
line and the secondary line through `displayUsingTraceOutput` ->
`processTraceInfo` -> `stringTrace()`.

So `Raised::report` runs it once over the whole report. **That also fixed the
clause echo**, which carries the program's own source bytes and diverged the
same way -- measured before the fix with a raw `0x01` inside a source literal:

```text
say copies('a<0x01>b','x')
  oracle echo:  1 *-* say copies('a?b','x')
  ours before:  1 *-* say copies('a<0x01>b','x')
```

After the fix, a source containing both a `0x01` and a `0xff` produces
**byte-identical stderr** on both interpreters (`cmp` clean).

### What changed

* `Raised::additional` is `Vec<Substitution>` where `Substitution = Vec<u8>`,
  with a doc comment giving the measurement that forces it.
* `substitute` and `Raised::message` return `Vec<u8>`; the catalogue template
  stays `&str` because `rexxmsg.xml` is.
* `error::displayable` is the byte rule, documented with the 256-value table
  and the C++ citation.
* Every raiser in `error.rs`, `run.rs` and `trace.rs` passes bytes. That
  includes `RAISE ... ADDITIONAL`'s own values (`run.rs`, two sites) and
  `raised_invalid_trace_letter`, which had the identical loss.
* `error::into_substitutions` is the one-way widening at the `rexx-num` /
  `rexx-parse` boundary, whose own substitution lists are `String` and are
  never arbitrary program data.

### A second defect the widened alphabet found, in this task's own code

The 768-program byte sweep turned up one mismatch that was **not** the
substitution class:

```text
say strip('ab','0000'x)     oracle: ab (rc 0)     ours: 93.915 (rc 163)
```

`optionArgument` (`classes/StringClassUtil.cpp:342`) tests
`strchr(validOptions, option) == NULL` over an ASCII-Z string, and `strchr`
**finds the terminating NUL**, so a `0x00` option byte passes a check written
to reject anything outside the set. It is then none of the letters, and each
caller answers with its own "neither" branch. Measured:

| probe | oracle |
|---|---|
| `strip('  ab  ','00'x)` | `  ab  ` -- nothing stripped |
| `strip('  ab  ','00'x\|\|'L')` | `  ab  ` -- only the first byte decides |
| `strip('  ab  ','L'\|\|'00'x)` | `ab  ` -- leading stripped |
| `verify('abcde','abc','00'x)` | `1` -- `'M'`'s answer, not `'N'`'s |
| `verify('abcde','xyz','00'x)` | `0` |
| `strip('  ab  ','01'x)` | 93.915, `found "?"` -- every other control byte still refused |

Reproduced, including the asymmetry that makes `VERIFY` answer `M`: the C++ is
`if (opt == VERIFY_NOMATCH) ... else ...`, so `N` is the branch tested for.
`verify` now branches on `option == b'N'`.

### How the new tests were made to fire

`a_substitution_carries_bytes_and_the_report_makes_them_displayable` and
`a_null_option_byte_is_accepted_and_matches_no_letter`, plus these mutations
(each applied alone, each restored from a copy, each run count checked
non-zero):

| Mutation | Suite |
|---|---|
| `substitute` re-introduces `from_utf8_lossy` | **RED** (1 failed) |
| `displayable` is a no-op | **RED** (1) |
| `displayable` also replaces bytes >= 0x80 | **RED** (9) |
| `displayable` also replaces tab/CR/LF | **RED** (9) |
| `option_letter` refuses `0x00` | **RED** (1) |
| `verify` branches on `M` instead of `N` | **RED** (1) |

And the **adds-coverage** check the project's own rule demands -- the mutation
kept, the new test deleted:

| Mutation | With the new test | Without it |
|---|---|---|
| `substitute` goes lossy | RED | **green (326 passed)** |
| `option_letter` refuses `0x00` | RED | **green (326 passed)** |

So each is the only thing in the suite that catches its defect, not merely a
test that can fail.

---

## F1 (Critical) -- the copy is gone; two causes outside this task remain, and are now recorded

### Reproduced first

```text
say length(copies('a',400000000))     at ulimit -v 1048576
  oracle: 400000000, rc 0
  ours:   SIGABRT rc 134, "memory allocation of 400000000 bytes failed"
```

### What was mine, and is fixed

`buffer()` reserved the result with `try_reserve_exact`, and `Interp::text`
then did `bytes.to_vec()` -- a second, infallible allocation of the same size.
`Interp::text_owned` now takes the buffer, and every builtin whose result size
comes from an argument hands its reservation over rather than lending it.

**The before/after is the abort's own backtrace, at the same size and the same
ulimit:**

```text
before:  11: <rexx_exec::Interp>::text
         12: rexx_exec::builtin::string::copies
after:   11: <rexx_exec::Interp>::resolve_and_run_call
         12: <rexx_exec::Interp>::eval_call
```

Tested by `text_owned_takes_the_buffer_rather_than_copying_it`, which asserts
on the allocation's **address**, since that is the only thing separating the
two -- the bytes are equal either way. Mutating `text_owned` to clone is RED
with it and **green without it** (326 passed), so it adds coverage rather than
merely being able to fail. A behavioural test at a size that fits once and not
twice is not something to put in a suite that runs on every `cargo test`; the
address assertion is the same property, cheaply.

### What is not mine, measured, and now a KNOWN GAP row

The symptom persists, and I isolated both remaining causes rather than
guessing at a threshold.

**Cause 1, the abort.** `resolve_and_run_call` renders every evaluated
argument to an owned `Vec` purely to hand it to `trace_argument`, which
discards it unless the trace mode asks for intermediates -- so a value of any
size is copied whether or not anything will print it. The assignment path does
the same at its `>>>` render (`run.rs:976`, which is where `x = copies(...)`
aborts). Measured as the whole of this cause, by an experiment that was
**reverted, not shipped**: with the argument render placed behind
`if self.trace_mode().intermediates`,

```text
300000000  oracle 300000000 rc 0  |  ours 300000000 rc 0
400000000  oracle 400000000 rc 0  |  ours 400000000 rc 0
```

and no size aborts at all. `run.rs`'s md5 was checked back against the
pre-experiment copy.

I did not ship it. It is one of about fifteen sites of the same shape in a file
this task does not own, choosing the guard wrongly silently drops a trace line,
and it does not close the divergence anyway (cause 2). That is the trade the
round-1 instruction named explicitly, and this is the branch it points to.

**Cause 2, the early Error 5.** `ulimit -v` limits **address space**, and
`INTERPRETER_STACK_BYTES` reserves 512 MiB of it for D19's sized interpreter
thread before any program runs. Under a 1 GiB limit the oracle has about
1000 MiB to allocate a result in and this crate has about 500. Measured, by
raising our limit by exactly that 512 MiB (1048576 KB -> 1572864 KB) while
leaving the oracle at 1 GiB:

| N | oracle @ 1 GiB | ours @ 1 GiB | ours @ 1.5 GiB |
|---|---|---|---|
| 400000000 | rc 0 | SIGABRT | rc 0 |
| 500000000 | rc 0 | Error 5, rc 251 | **rc 0** |
| 700000000 | rc 0 | Error 5, rc 251 | SIGABRT (cause 1) |

The Error-5 threshold moves by exactly the stack reservation. **There is no
threshold in this crate to re-derive**: the guard asks the allocator, and the
two processes simply have different amounts of address space to spend. Closing
it means changing D19, not this task.

Both causes, both tables and both experiments are now a `KNOWN GAP` row in
`phase-4-exclusions.txt`.

The boundary the first round did verify is unaffected and still correct:
`left('ab','999999999999999999')` is Error 5 and
`left('ab','1234567890123456789')` is 40.12, one digit past `ARGUMENT_DIGITS`.

---

## F3 (Important) -- the TRANSLATE gap now has a row, nine producers, and a measured reason for its status

`KNOWN GAP: TRANSLATE` is in `phase-4-exclusions.txt`, with the C++ citation
(`RexxString::translate` tests `tablei != GlobalNames::NULLSTRING`) and the
full producer set. The reviewer found four; I enumerated and measured every
one among the 23 names:

**Produce the singleton** (so `translate` takes the omitted path):
`LEFT`, `RIGHT`, `CENTER`, `CENTRE` with width 0; `SUBSTR` with length 0;
`DELSTR` deleting the whole string; `STRIP` with everything stripped;
`COPIES` with count 0 **or** a null subject; `SPACE` with no words.

**Do not**: a `''` literal, `REVERSE('')`, and a `CHANGESTR` that changes
nothing -- all three return their receiver rather than the singleton.
Concatenating the singleton with a literal `''` still yields it.

The scope claim is measured, not assumed: `!= GlobalNames::NULLSTRING` appears
**once** in `interpreter/classes/`, at `StringClassMisc.cpp:730`, which is
`translate`.

### `implemented` plus a gap row, and why -- measured, not argued

`builtin-status.txt` is **derived** from one probe per name, and TRANSLATE's
probe (`say translate('abcdef','123','abc')`) matches. Committing the row as
`divergent` makes the harness red the other way:

```text
rows whose measured status differs from .../builtin-status.txt:
  TRANSLATE: committed divergent, measured implemented
test result: FAILED. 9 passed; 3 failed
```

The file cannot hold a divergence its own probe does not reproduce, so the
KNOWN GAP row is where the record has to live. That is now stated in the row
itself, so the next reader does not re-litigate it.

---

## F4 (Important) -- the sweep, with its alphabet reported

The finding was correct: the round-1 corpora contained no hex literal and no
byte >= 0x80. Corrected by adding a corpus rather than by rewording the claim.

| Corpus | Programs | Mismatches | Operand alphabet |
|---|---|---|---|
| A: value sweep (round 1) | 62,144 | **0** | printable ASCII only: `''`, `'a'`, `'ab'`, `'abcdef'`, `'banana'`, `'aXbXc'`, `'a b  c '`, `'  ab  '`; numbers `-0 0 1 1.0 2 3 7 9 '  2  '`; pads `. -` and blank |
| B: error paths (round 1) | 3,105 | **0** | as A, plus `-1 0 x '' 1.5 -1.0 1E18 1234567890123456789 '- 5' '--5' .5`, multi-character and empty pads, invalid option letters, every arity 0..max+2, every interior-omission position |
| C: **byte-widened** (new) | 18,576 | **0** | adds `'ff'x 'fffefd'x '80'x '7f'x '00'x '01'x '09'x '0a'x '61ff62'x '6100620a63'x '001f7f80ff'x`, high and control bytes in every numeric, pad and option position, and the option letters `'00'x`, `'01'x`, `'ff'x`, `'00'x\|\|'L'`, `'L'\|\|'00'x` |
| D: **null-string singleton** (new) | 234 | **20** | thirteen producers of a null string in every optional-string position of all 23 names |
| E: all 256 byte values through the message path | 768 | **0** | every byte 00-ff as a bad count, a bad pad and a bad option |

**All 20 of D's mismatches are `translate` with the singleton in the
`tablein` position** -- the recorded KNOWN GAP, and nothing else. That is also
the sweep's own negative control for this round: a corpus that reports 0
everywhere would not be able to see the one divergence known to exist.

The round-1 hand probes were all re-run: 21 groups, 0 mismatches, except p4's
two known singleton lines.

**What the sweeps still do not cover**, stated rather than left implied: no
program is larger than a few dozen bytes, so nothing here exercises the
allocation boundary of F1; no probe combines a builtin with `PARSE`, `ARG` or
a `::routine`, none of which exists yet; and D's producer list is the ten
measured producers plus three non-producers, not a proof that no fourteenth
exists.

---

## Corrections to round 1's own report

* §5.3 said the eight builtins' results were guarded. They were guarded
  against one of the two allocations each result cost. The claim was wrong
  in the way F1 says; §"F1" above replaces it.
* The sweep numbers in §3.2 were true and incomplete. The alphabet is now
  reported beside every count, here and above.

## Still not done

* The `run.rs` unconditional trace renders (F1 cause 1) and D19's address-space
  share (cause 2). Recorded as a KNOWN GAP with the isolating experiments; both
  need an owner.
* The `TRANSLATE` singleton divergence, recorded with all nine producers.

---
---

# Fix round 2

Commit `53ba18b5ddcfbb15b96a45198b173a1546c0fac3`, working tree clean. Both
findings reproduced independently before anything was changed.

## Verify block, each status read unpiped

| Command | Exit | Result |
|---|---|---|
| `cargo test --offline --workspace --no-fail-fast` | **0** | 1059 passed, 0 failed |
| `cargo fmt --all --check` | **0** | clean |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | **0** | clean; the run re-checked `rexx-exec` |
| `REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus` | **0** | `mode: STRICT (the gate)`, `42 of 42 matching` |

---

## F-N1 -- `VERIFY`'s two branches test different letters

### Reproduced first

```text
say verify('abcde','','00'x)      oracle 1   ours 0
say verify('abcde','','00'x,3)    oracle 3   ours 0
```

### The defect

`StringUtil::verify` has **two** option tests, not one:

```cpp
if (referenceLen == 0)
{
    if (opt == RexxString::VERIFY_MATCH) { return IntegerZero; }
    else                                 { return new_integer(startPos); }
}
else
{
    if (opt == RexxString::VERIFY_NOMATCH) { ...first non-match... }
    else                                   { ...first match... }
}
```

Round 1 introduced `let nomatch = option == b'N';` and used it for both. For
`'M'` and `'N'` the two spellings agree, so nothing visible changed -- but the
`0x00` option the oracle's `strchr` admits is *neither*, and it takes the
**second arm of each**, which are opposite letters' answers. The empty-
reference arm was therefore answering `'M'`'s question where the oracle
answers `'N'`'s.

The two tests are separate again, each carrying the C++ line it transcribes.
Round 0 had the empty branch right; round 1 broke it while fixing the other
one, which is exactly the shape the project's own note about correction rounds
describes.

### The corpus lesson, which is sharper than "the corpus lacked empty references"

Counted over the committed corpora:

| corpus | `verify` programs | with an empty reference | with a `0x00` option | with **both** |
|---|---|---|---|---|
| A (value, round 0) | 4,096 | 8 | 0 | **0** |
| B (error paths, round 0) | 102 | 0 | 0 | **0** |
| C (byte-widened, round 1) | 384 | 0 | 384 | **0** |

So it is not that either axis was missing -- **each corpus varied one axis
while holding the other at a value that hides the defect.** A defect at the
intersection of two axes is invisible to corpora that vary them separately,
however large each is. That is a stronger statement than round 1's alphabet
finding and it generalises past bytes.

### Corpus F, and the negative control the round-2 instruction asked for

New generator: every **string** position of every builtin draws from an
alphabet including `''`, a high byte, a control byte and a multi-byte mixed
string, and every **option** position draws from the widened option set at the
same time -- so the two axes cross.

| build | corpus F | corpus C | corpus B |
|---|---|---|---|
| this commit | **0** of 8,070 | 0 of 18,576 | 0 of 3,105 |
| round 1's `verify` restored | **72** | 0 | 0 |

72 mismatches, all `verify` with an empty reference and a non-`M`, non-`N`
option, e.g.

```text
say '['verify('01'x,'','00'x||'L',1)']'    oracle [1]   round-1 build [0]
```

The two older corpora report 0 against the same buggy build, which is the
measurement behind the paragraph above rather than an inference from it.

### Unit coverage

Added to `a_null_option_byte_is_accepted_and_matches_no_letter`: the empty-
reference cases, plus the four letter cases either side of them, which is what
pins each arm to the letter it actually tests rather than to a coincidence.

* With round 1's `verify` restored: **RED** (1 failed of 327).
* With round 1's `verify` restored **and the new assertions deleted**:
  **green** (326 passed). So they add coverage, not merely the ability to fail.

### The other empty-argument branches, enumerated rather than asserted

The instruction asked whether any other builtin has a branch selected by an
empty argument that the corpora never supply. I enumerated them from the C++
rather than from memory, and corpus F drives every one:

| C++ site | builtin | empty argument | corpus F |
|---|---|---|---|
| `StringClassMisc.cpp:1305` | VERIFY | reference | `verify(s,'',...)` -- the defect |
| `StringClassMisc.cpp:88` | ABBREV | info, with length 0 | `abbrev(s,'',0)` |
| `StringClassMisc.cpp:296` | COPIES | subject | `copies('',n)` |
| `StringClassMisc.cpp:708` | TRANSLATE | range 0 | `translate(s,r,r,...)` with range 0 |
| `StringClassWord.cpp:66` | SPACE | subject (no words) | `space('',n,p)` |
| `StringUtil.cpp:217` | POS | needle | `pos('',s,n)` |
| `StringUtil.cpp:346` | LASTPOS | needle **and** haystack | `lastpos('',s,n)`, `lastpos(r,'',n)` |
| `StringUtil.cpp:1186` | COUNTSTR | needle | `countstr('',s)` |
| `StringUtil.cpp:1226` | CHANGESTR | needle | `changestr('',s,r,n)` |
| `StringClassUtil.cpp:350` | STRIP, VERIFY | option | `''` is in the option alphabet |
| `StringClassSub.cpp` (`charsLen`) | STRIP | character set | `strip(s,o,'')` |
| `StringClassSub.cpp` (`newLen`) | INSERT, OVERLAY | inserted string | `insert('',s,n)`, `overlay('',s,n)` |
| `StringClassMisc.cpp` (`length2`) | COMPARE | second string | `compare(s,'',p)` |
| `StringClassMisc.cpp:730` | TRANSLATE | tableout, tablein | `translate(s,'')`, `translate(s,'','')` |

All of them clean.

---

## F-N2 -- the byte rule now runs at both of the oracle's routes to its one sink

### Reproduced first

```text
trace r ; say 'p'||'02'x||'q'
  stdout: identical (the raw byte belongs there)
  stderr oracle:  >>>   "p?q"
  stderr ours:    >>>   "p<0x02>q"
```

### The fix, and the pairing made checkable

The oracle's single sink is `RexxActivation::processTraceInfo`
(`execution/RexxActivation.cpp:5249`), whose first act is
`traceLine->stringTrace()`. It is reached two ways, and this crate now has one
application per way, each naming its counterpart in the code:

| this crate | oracle |
|---|---|
| `Raised::report` (round 1) | `Activity::display` (`concurrency/Activity.cpp:1414`) -> `displayUsingTraceOutput` (`:5262`) -> `processTraceInfo` |
| `trace.rs`'s `make_displayable`, called by the three line formatters | `processTraceInfo` directly, for every live `TRACE` line |

`push_operator` does not call it, because it delegates to `push_tagged`, which
does. Double application on a report's clause echoes is the identity: `?` is
`0x3f`, above the threshold.

### Measured after

`trace r`, `trace i`, `trace a` and `trace l` over programs exercising `*-*`,
`>L>`, `>V>`, `>=>`, `>O>`, `>A>`, `>F>` and `>>>` with control bytes and high
bytes: **14 programs, 1 mismatch**, and that one is `ARG`, a builtin another
task owns and which fails loudly. The `trace r` reproducer above is now
byte-identical on stderr as well as stdout.

### Unit coverage

`every_completed_trace_line_is_made_displayable` checks all four formatters
plus the negative case (`0xff`, `0x09`, `0x0d`, `0x80` pass through), because
without that last line the rule could be "everything outside printable ASCII"
and every other assertion would still pass.

| Mutation | Suite |
|---|---|
| `make_displayable` is a no-op | RED |
| `push_clause` skips it | RED |
| `push_value` skips it | RED |
| `push_tagged` skips it | RED |
| no-op mutation **with the new test deleted** | **green (327)** |

---

## Full corpus state at this commit

| Corpus | Programs | Mismatches |
|---|---|---|
| A: value sweep, printable ASCII | 62,144 | 0 |
| B: error paths | 3,105 | 0 |
| C: byte-widened | 18,576 | 0 |
| D: null-string singleton | 234 | 20 -- all the recorded `KNOWN GAP: TRANSLATE` |
| E: all 256 byte values through the message path | 768 | 0 |
| F: **empty/byte alphabet crossed with the option set** (new) | 8,070 | 0 |
| trace prefixes under four modes (new) | 14 | 1 -- `ARG`, another task's loud gap |

## Corrections to this report's own earlier rounds

* Round 1's `verify` doc comment said "`N` is the branch that is tested for,
  not `M`". That was true of one branch and false of the other, and the code
  matched the comment. Both are now stated separately, each with its C++ line.
* Round 1 reported the byte-widened corpus C as covering the option axis. It
  did, and it held the reference non-empty throughout; the table above says so
  rather than leaving the zero unqualified.
* Two items the coordinator flagged as theirs -- the over-generalised
  "MEASURED AS THE WHOLE OF THIS CAUSE" in the allocation `KNOWN GAP` row, and
  the false reason recorded for TRANSLATE's status staying `implemented` --
  are **not** touched here, as instructed. The TRANSLATE status row is
  unchanged.
