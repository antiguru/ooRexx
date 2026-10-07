# Loud refusal census, by owner

Tree: `plan/rust-rewrite` at 76dd9582e. Source text only; nothing was built or run. The question is
which loud refusals (rc 120, `rexx-exec: ...` on stderr, or a rexx-api abort) name no remaining
phase (9, 10, 11).

Classes:

- (a) names Phase 9, 10 or 11.
- (b) names a closed phase. Only `Phase 5` occurs; no literal names Phases 0-4, 6, 7 or 8 (C5).
- (c) names no phase, and a program can reach it as a feature gap.
- (d) other: a licensed deviation or oracle-crash stand-in (`d-licensed`), or an
  internal-inconsistency guard that a parser or compiler guarantee keeps out of reach (`d-guard`).

The class of each site is my reading of the source, written out as the mapping in `classify.py`
(appendix). The site list itself is derived.

## Commands

All from `rust/`, `S` being the directory holding the appendix scripts.

C1, every `Loud` construction site in `rexx-exec/src`, test files excluded:

    python3 -I $S/sites.py crates/rexx-exec/src
    -> 354 sites 47 constructors

The 354 include the constructors' own bodies (`lib.rs:366-890`) and the `-> Loud {` signature at
`dispatch/native.rs:735`; C2 drops both.

C2, the class of each remaining site:

    python3 -I $S/sites.py crates/rexx-exec/src list | python3 -I $S/classify.py
    ->  27 a
         6 a|b
       100 b
         6 c
         1 c|d
       115 d-guard
         6 d-licensed
       261 sites

C3, the constructor set against `corpus/refusal-sites.tsv`:

    python3 -I $S/sites.py crates/rexx-exec/src | awk 'NF==2 && $2!="<literal>"{print $2}' | sort > mine
    grep -P "^Loud\t" corpus/refusal-sites.tsv | cut -f2 | sort > tsv
    wc -l < mine; wc -l < tsv; diff mine tsv
    -> 46, 47, "> deferred_send"

`deferred_send` builds its `Loud` as a struct literal, which C1 lists as `<literal>`. The sets
otherwise agree.

C4, rexx-api interface members still holding their refusing stub:

    python3 -I $S/api.py crates/rexx-api/src/layout.rs crates/rexx-api/src/ffi.rs
    -> 5 refusing members, each with a REFUSING_MEMBERS row naming Phase 9

C5, every phase literal outside comments and test files:

    grep -rnoE '"Phase [0-9]+[a-z]?"' --include=*.rs crates | grep -v '/tests/\|tests\.rs' \
      | awk -F: '{print $1" "$3}' | sort | uniq -c
    -> "Phase 5" in environment.rs, lib.rs, redirect.rs, run.rs; "Phase 9" in layout.rs, lib.rs;
       "Phase 10" in directives.rs, dispatch/library.rs, dispatch/native.rs, environment.rs,
       internal_routines.rs, lib.rs, redirect.rs, rexx-inventory/src/lib.rs

C6, the loud lines ooTest actually produced in the S5 whole-groups record:

    grep -o "rexx-exec: [^,]*\(, which is not implemented\)\?" \
      docs/superpowers/records/2026-10-01-phase-6-s2-s5/whole-groups/table.txt | sort | uniq -c
    -> 12 distinct lines: DO (c); USE LOCAL and five "does not parse here" (b);
       three `method ... (Phase 9)` (a); CONDITION "O" (c); the deadline report (d)

## Summary per class

Sites from C2, plus refusals that are not `Loud` construction sites.

| class | C2 sites | outside C2 |
|---|---|---|
| (a) | 27, plus the Phase 10 half of 6 `a\|b` | rexx-api: the C4 members (Phase 9) and two `assert!`s at `ffi.rs:4172`, `:4214` (Phase 9); owner tables: `internal_routines.rs` rows (all `Phase 10`), `native.rs` deferred rows (all `Phase 10`), `rexx-inventory` `EXCLUDED` (all `Phase 10`) |
| (b) | 100, plus the Phase 5 half of 6 `a\|b` | none |
| (c) | 6, plus `Raised` of 1 `c\|d` | top-level parse failure, `lib.rs:3223` |
| (d) | 6 licensed, 115 guards, plus `StaleHandle` of 1 `c\|d` | deadline report, `lib.rs:216` |

Policing today: `tests/closed_phases.rs` holds `CLOSED = ["Phase 6", "Phase 7", "Phase 8"]` and
records Phase 5's absence as a debt (`closed_phases.rs:31-38`), so nothing checks class (b).
`tests/loud.rs` witnesses `OPTIONS` alone. Nothing checks (c): `owned_message` (`lib.rs:906-915`)
answers an unsuffixed message by design for a variant this crate "implements".

## (a) caveats

- `Loud::native_method` (`lib.rs:538`) names Phase 9 for every primitive method with no code.
  Phase 8's close relabelled it wholesale (queued `2026-09-28-phase-5-refusal-labels`); queued
  `2026-10-01-condition-object-directory-methods` says the owner was "not checked against any plan
  row". C6 shows three of these in ooTest (`Object~MAKEARRAY`, `Object~OBJECTNAME=`, a
  `~define`d `TEST1`).
- `a|b` sites (`environment.rs:527`, `dispatch/hash.rs:441`, `environment/identities.rs:536`,
  `dispatch/class_protocol.rs:454`, `:745`, `:1052`) take the owner from the owed placeholder
  (`environment.rs:363`). `.local`'s `STDQUE` is the statically known Phase 10 one
  (`environment.rs:448-454`). A Phase 5 placeholder remains only for an `ORACLE_ENVIRONMENT` or
  minted `.local` name the bootstrap failed to fill; which, if any, needs a run.

## (b) full list: names Phase 5

C2 sites per constructor: `receiver_class` 63, `method_from_source` 9, `object_position` 5,
`setup_method` 5, `operator_operand` 4, `method_body` 3, `required_source` 3, `object_method` 2,
`environment_symbol` 2, `library_source` 1, `expose_receiver` 1, `use_local_in_a_method` 1,
`instruction` 1 (from `classify.py b`).

| text pattern | source | Rexx feature | why no remaining-phase owner |
|---|---|---|---|
| `a message send to {kind} is not implemented (Phase 5)` | `lib.rs:480`; sites in `dispatch/*`, `environment*.rs` (C2 `b` list) | a send whose receiver the method row cannot handle: not an array, list, supplier, hash collection, variable reference, class object; a package, stack frame, method, routine or executable this crate did not build; a stem default (`a. = 'dflt'; a.~length`, refusal-sites.tsv); a context whose scope no longer defines its method (`identities.rs:201`) | owner rotted: Phase 5 closed. Most sites are type guards on primitive rows; refusal-sites.tsv marks the constructor `diverges`, so some are reachable |
| `the operator \`{op}\` applied to {kind} is not implemented (Phase 5)` | `lib.rs:499`; `eval.rs:671`, `:855`, `:1067`, `:1163` | an operator whose left operand is a class object or `.environment`, `.local`, `.methods`, `.context` | owner rotted |
| `{kind} as {position} is not implemented (Phase 5)` | `lib.rs:512`; `run/loops.rs:553`, `:716`, `:2233`, `run.rs:2278`, `run/condition.rs:733` | the same objects as a DO header value, DO OVER target, controlled-loop control variable, `FORWARD ARGUMENTS`, `RAISE ... ADDITIONAL` | owner rotted |
| `a class method built from source text`, `a {class,one-off,enhancing} method whose body this crate does not hold`, `a routine whose body this crate does not hold`, `a method source that is neither a string nor an array` `... (Phase 5)` | `lib.rs:598`; `dispatch/class_protocol.rs:274`, `:777`, `:782`, `:1115`, `dispatch/object_protocol.rs:488`, `:728`, `dispatch/executable.rs:581`, `:622` | `setMethod`, `defineMethods`, `enhanced`, `~new` on Method/Routine with a source or a method object whose body is not held | owner rotted |
| `reporting a file that does not parse ({path}, {error}) is not implemented (Phase 5)` | `install.rs:1778` | `Method/Routine~newFile` on a file with a syntax error | owner rotted; the oracle raises the parse error |
| `{path} does not parse here: {error} is not implemented (Phase 5)` | `lib.rs:632` (`required_source`); `install.rs:657` (`::REQUIRES`), `install.rs:953` (`Package~new` from source), `lib.rs:2208` (external call) | a loaded file or source with a syntax error; C6 shows it for `test` (`Package~new`) and `constant_TestGroup` | owner rotted. Recorded as a KNOWN GAP, `phase-4-exclusions.txt:1895-1925`, which says "NOTE THE OWNER IT NAMES"; the oracle raises the callee's own error (rc 229 in that row) |
| `{name} does not parse here: {error} ... (Phase 5)` | `lib.rs:622` (`library_source`); `lib.rs:2254` | an embedded `.orx` the parser rejects | `rexx-lib` pins each file's sha256 (`lib.rs:2250-2252`), so unreachable from a program |
| `a receiver with no {scope,class} of its own ... (Phase 5)` | `lib.rs:606`; `dispatch.rs:2082`, `dispatch/object_protocol.rs:531` | `setMethod`/`unsetMethod` on a receiver with no instance dictionary; scope `OBJECT` on such a receiver | owner rotted |
| `a ::METHOD/::ATTRIBUTE with no body of its own`, `a method installed by ::{KEYWORD}` `(Phase 5)` | `lib.rs:615`; `directives.rs:437`, `:444`, `:449` | a send resolving to a directive with no body | owner rotted |
| `defineClassMethod ...`, `inheritInstanceMethods ...` `(Phase 5)` | `lib.rs:644`; `dispatch/class_protocol.rs:849`, `:853`, `:859`, `:879`, `:883` | the two Setup methods on `.Class` | refusal-sites.tsv: "no route: Setup.cpp removes both before an image ships" |
| `EXPOSE on an object with no variable pool is not implemented (Phase 5)` | `lib.rs:652`; `run.rs:1533` | `EXPOSE` in a method whose receiver is neither a class object nor an instance | owner rotted |
| `USE LOCAL in a ::METHOD body is not implemented (Phase 5)` | `lib.rs:660`; `run.rs:1617` | `USE LOCAL` as a method's first instruction (oracle rc 0); C6 shows it in ooTest | owner rotted |
| `OPTIONS is not implemented (Phase 5)` | `lib.rs:1014`; `run.rs:1197` | the `OPTIONS` instruction | owner rotted; the only row `tests/loud.rs` witnesses |
| `environment symbol ".STREAM" is not implemented (Phase 5)` | `run.rs:3835`, `redirect.rs:652` | a stream name before `StreamClasses.orx` installs | bootstrap-only per the comments at both sites |
| `directory entry "{X}"`, `environment symbol "{.X}"`, `a directory whose entries this crate does not fill` `(Phase 5)` | the `a\|b` sites | an owed `.environment`/`.local` entry | see (a) caveats |

Queued coverage: `2026-09-28-phase-5-refusal-labels` covers relabelling all of (b) (its line
numbers have drifted: `redirect.rs:644` is now `:652`, `run.rs:3589` is now `:3835`). Feature
coverage: `2026-10-02-context-executable-setmethod` (the `identities.rs:201` receiver);
`2026-09-28-compiled-source-parse-errors` is adjacent (the `Method~new` compile path, which raises
now), not the `required_source` refusal.

## (c) full list: names no phase

| text pattern | source | Rexx feature | why no owner | queued |
|---|---|---|---|---|
| `DO is not implemented`, `LOOP is not implemented` | `run/loops.rs:770` via `loop_header_plan` (`run/loops.rs:371-410`); the IR falls back to it at `:1333` | `DO/LOOP ... COUNTER`, `DO WITH ... OVER`, `DO x OVER stem.` | owner rotted. The 4a split gave `LoopKind::With` to Phase 5 (`2026-07-30-phase-4a-executor-design.md:71`); Phase 5 closed without it. The message is unsuffixed because `instruction_owner` answers `None` for `Do` (`run/tests/loops.rs:196-228` pins that). The stem case contradicts deviation 1 (`phase-4-exclusions.txt:844`), which describes a hash-order walk | `2026-10-02-do-with-over-refusal` covers WITH and COUNTER ... OVER; COUNTER on other loop kinds and stem OVER are not named. C6: the most frequent ooTest refusal |
| `a message send is not implemented` | `run.rs:1853` (`use_target_name`) | `USE ARG o~a` | never assigned | `2026-09-28-use-arg-message-term` ("owner: not assigned") |
| `CONDITION option "O" answers a Directory, which is not implemented` | `dispatch/context.rs:322` | `.context~condition` inside a handler | never assigned; `Interp::condition_copy` now builds the copy the oracle answers (queued note) | `2026-09-28-condition-object-divergences-found-in-fix-round-a`, last paragraph. C6: ooTest `TESTCONDITION01` |
| `CONDITION option "D" answers the NOVALUE variable's name, which is not implemented` | `builtin/state.rs:360` | `CONDITION('D')` for a NOVALUE condition with no description | never assigned; no record found beyond the code comment | none |
| `a generated accessor for the attribute "{X}" is not implemented` | `lib.rs:668`; `dispatch.rs:3078` | `::ATTRIBUTE` whose variable is a stem or a compound tail (`::attribute "A.B"`) | never assigned; refusal-sites.tsv `diverges`; no exclusions row (grep for the text finds none) | none |
| `a DELEGATE to the variable "{X}" is not implemented` | `lib.rs:680`; `dispatch.rs:3059` | `::METHOD m DELEGATE a.b` | as above | none |
| `the interpreter raised a condition while converting is not implemented` | `dispatch/library.rs:857` (`Refused::Raised`) | a native argument conversion that raised | the comment at `:851-855` justifies `StaleHandle` only; reachability not checked | none |
| `{ParseError}` (no "is not implemented") | `lib.rs:3196-3232` | a main program with a syntax error: rc 120 where the oracle reports the error | known gap, `phase-4-exclusions.txt:1924-1925` ("driven through `rexxc` by `corpus/errors/parse-errors.tsv`") | none |

The queued `2026-10-02-elapsed-clock-per-routine-and-reset` and both 2026-10-07 debug items are
silent wrong answers, not loud refusals, so they cover nothing here.

## (d) summary

`d-licensed`, each with a record:

| text pattern | source | record |
|---|---|---|
| `an array subscript that is an empty slot is not implemented` | `dispatch/array.rs:180` | refusal-sites.tsv: `oracle-crashes.txt` entry 6 (SF 2085), not-run |
| `an entry-method assignment to "{X}" with no value is not implemented` | `dispatch/hash.rs:1856` | refusal-sites.tsv `recorded` |
| `a wait that nothing left to run can end is not implemented` | `scheduler.rs:998`, `:1018` | deviation 20, `phase-4-exclusions.txt:1673`, OWNER: none |
| `a pinned wait for {X} that only an activity pinned below it can end is not implemented` | `scheduler.rs:1021` | deviation 14, `:1574`, OWNER: none |
| `a REPLY its method body runs on a nested Rust frame is not implemented` | `run.rs:2070` | deviation 15, `:1592`, OWNER: none |
| `the handle is no longer held by this activation is not implemented` | `dispatch/library.rs:857` (`StaleHandle`) | comment `:851-855`: outside the API |

These three wait/reply refusals are Phase 6 design limits (spec 2026-09-29) with no owner by
decision, so they stay unattributable after every remaining phase.

`d-guard`: the constructors in `classify.py`'s guard group, the `instruction` sites other than
`run.rs:1197` and `run/loops.rs:770`, the `expression` sites other than `run.rs:1853`,
`run/settings.rs:356` (`NUMERIC FORM VALUE with no expression`), and `Failure::Slice` at
`lib.rs:3130`, `:3332`. Their doc comments describe each as an internal inconsistency or a parser guarantee;
refusal-sites.tsv marks
`missing_body`, `scheduler_inconsistency` and `library_procedure_gone` `agrees`. `phase-6-gate.md:311`
records one firing in a group run (`a compiled call op does not name a call of its own body`), so "unreachable" is a claim, not a measurement. The deadline report
(`lib.rs:216`) is harness output.

## Appendix: scripts

The scripts C1, C2 and C4 run, as they ran.

### sites.py

```python
# Every Loud construction site in rexx-exec's non-test sources:
# `Loud::ctor(` calls (any path prefix) and struct literals `Loud {`.
# Comment lines and test files (tests.rs, *_tests.rs, tests/ dirs) excluded.
import re, sys, pathlib, collections
root = pathlib.Path(sys.argv[1])
call = re.compile(r'\bLoud::([a-z_]+)\b')
lit = re.compile(r'\bLoud\s*\{')
rows = []
for p in sorted(root.rglob('*.rs')):
    s = str(p)
    if '/tests/' in s or s.endswith('tests.rs') or s.endswith('_tests.rs'):
        continue
    in_cfg_test = False
    for n, line in enumerate(p.read_text().splitlines(), 1):
        t = line.strip()
        if t.startswith('//'):
            continue
        code = t.split('//')[0]
        for m in call.finditer(code):
            rows.append((m.group(1), f'{p.relative_to(root.parent.parent)}:{n}'))
        if lit.search(code) and 'struct Loud' not in code and 'impl Loud' not in code:
            rows.append(('<literal>', f'{p.relative_to(root.parent.parent)}:{n}'))
mode = sys.argv[2] if len(sys.argv) > 2 else 'count'
if mode == 'count':
    c = collections.Counter(r[0] for r in rows)
    for k, v in sorted(c.items()):
        print(f'{v:4} {k}')
    print(len(rows), 'sites', len(c), 'constructors')
else:
    for r in rows:
        print(*r, sep='\t')
```

### classify.py

```python
# Classifies each rexx-exec construction site sites.py lists (stdin, "ctor\tfile:line")
# by the owner its message names. The mapping is read from the source by hand;
# PER_SITE overrides the constructor default where one constructor serves sites
# of different classes. Sites inside lib.rs's `impl Loud` (366-890) are the
# constructors' own bodies and are dropped.
import sys, collections
DEFAULT = {
  # (a) remaining phase
  'native_method': 'a', 'named_semaphore': 'a', 'package_option_write': 'a',
  'redirection': 'a', 'internal_routine': 'a', 'unresolved_call': 'a',
  # owner from the owed placeholders: Phase 5 or Phase 10 per entry
  'environment_entry': 'a|b', 'unreadable_collection': 'a|b',
  # (b) Phase 5
  'receiver_class': 'b', 'operator_operand': 'b', 'object_position': 'b',
  'method_from_source': 'b', 'object_method': 'b', 'method_body': 'b',
  'library_source': 'b', 'required_source': 'b', 'setup_method': 'b',
  'expose_receiver': 'b', 'use_local_in_a_method': 'b',
  # (c) no phase, a program-reachable gap
  'accessor_variable': 'c', 'delegate_variable': 'c', 'builtin_option_object': 'c',
  # (d) licensed deviation or oracle-crash stand-in
  'array_index_hole': 'd-licensed', 'entry_method_without_a_value': 'd-licensed',
  'unsatisfiable_wait': 'd-licensed', 'inverted_wait': 'd-licensed',
  'immovable_reply': 'd-licensed',
  # (d) internal-inconsistency guard
  'binary_operator': 'd-guard', 'library_procedure_gone': 'd-guard',
  'parse_trigger_operand': 'd-guard', 'missing_body': 'd-guard',
  'chunk_map_too_short': 'd-guard', 'chunk_refused': 'd-guard',
  'op_not_driven': 'd-guard', 'jump_out_of_range': 'd-guard',
  'register_not_logical': 'd-guard', 'select_op_off_its_node': 'd-guard',
  'loop_op_off_its_node': 'd-guard', 'signal_op_off_its_node': 'd-guard',
  'store_op_off_its_node': 'd-guard', 'call_op_off_its_node': 'd-guard',
  'constant_out_of_range': 'd-guard', 'scheduler_inconsistency': 'd-guard',
  'instruction': 'd-guard', 'expression': 'd-guard',
}
PER_SITE = {
  'rexx-exec/src/run.rs:1197': 'b',            # OPTIONS, instruction_owner Phase 5
  'rexx-exec/src/run/loops.rs:770': 'c',       # DO/LOOP COUNTER, WITH, stem OVER
  'rexx-exec/src/run.rs:1853': 'c',            # USE ARG target that is a message term
  'rexx-exec/src/run.rs:3835': 'b',            # .STREAM before StreamClasses.orx
  'rexx-exec/src/redirect.rs:652': 'b',        # .STREAM before StreamClasses.orx
  'rexx-exec/src/environment.rs:527': 'a|b',   # owed .NAME
  'rexx-exec/src/directives.rs:64': 'a',       # ::ROUTINE EXTERNAL REGISTERED, Phase 10
  'rexx-exec/src/dispatch/native.rs:736': 'a', # deferred LIBRARY REXX entry, Phase 10
  'rexx-exec/src/dispatch/library.rs:837': 'a',# ROUTINE_CLASSIC_STYLE, Phase 10
  'rexx-exec/src/dispatch/library.rs:843': 'a',# unfilled API slot, Phase 9 rows
  'rexx-exec/src/dispatch/library.rs:857': 'c|d', # StaleHandle (d) or Raised (c)
  'rexx-exec/src/run/settings.rs:356': 'd-guard', # NUMERIC FORM VALUE with no expression
}
counts = collections.Counter(); rows = collections.defaultdict(list)
for line in sys.stdin:
    ctor, site = line.rstrip('\n').split('\t')
    f, n = site.rsplit(':', 1)
    if f.endswith('lib.rs') and f.startswith('rexx-exec/src/lib.rs') and 366 <= int(n) <= 890:
        continue
    if site == 'rexx-exec/src/dispatch/native.rs:735':
        continue  # the fn signature line `-> Loud {`, its body is :736
    cls = PER_SITE.get(site) or DEFAULT.get(ctor)
    if cls is None:
        cls = 'UNCLASSIFIED'
    counts[cls] += 1; rows[cls].append(f'{ctor} {site}')
for k in sorted(counts):
    print(f'{counts[k]:4} {k}')
print(sum(counts.values()), 'sites')
if len(sys.argv) > 1:
    for r in rows[sys.argv[1]]:
        print('  ', r)
```

### api.py

```python
# Function members of each populated interface table (layout.rs) that no
# `table.Member = ...` line in the table's fill block (ffi.rs) replaces: the
# members still holding their refusing stub, with the owner REFUSING_MEMBERS
# gives (None = unsuffixed refusal).
import re, sys, pathlib
layout = pathlib.Path(sys.argv[1]).read_text()
ffi = pathlib.Path(sys.argv[2]).read_text()
owners = dict(re.findall(r'\("(\w+\.\w+)", "(Phase \d+)"\)', layout))
tables = {}
for m in re.finditer(r'interface! \{.*?\n    (\w+), populated \{\n(.*?)\n    \}\n\}', layout, re.S):
    name, body = m.group(1), m.group(2)
    fields = re.findall(r'^\s*(\w+): \{ (call|aborts|unwinds)\b', body, re.M)
    tables[name] = [f for f, _ in fields]
total = 0
for name, fields in tables.items():
    blocks = re.findall(name + r'::REFUSING;\n(.*?)\n\s*table\n', ffi, re.S)
    filled = set()
    for b in blocks:
        filled |= set(re.findall(r'table\.(\w+) =', b))
    left = [f for f in fields if f not in filled]
    # any other assignment to the member name anywhere in ffi.rs
    for f in left:
        other = len(re.findall(r'\.' + f + r' = ', ffi))
        key = f'{name}.{f}'
        print(f'{key}\t{owners.get(key)}\tother-assignments={other}\tfill-blocks={len(blocks)}')
        total += 1
    print(f'# {name}: {len(fields)} function members, {len(filled)} filled, {len(left)} refusing', file=sys.stderr)
print(f'# {total} refusing members', file=sys.stderr)
```
