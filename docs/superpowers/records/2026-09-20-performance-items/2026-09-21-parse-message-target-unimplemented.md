# Defect: a PARSE target that is a message term is unimplemented

Found 2026-09-21 by the `exec_parse` cost scout, incidentally, while checking
whether a target assignment can run Rexx code. Not a performance item. Not
looked for anywhere else, so **the extent is underived**: this is one witness,
not a survey.

Confirmed independently against the oracle and against `rexx-run` built at
`rust/` subtree `63686bcc7247d7cd2baac82969fc43242c63f01a`.

## The witness

    o = .Kustom~new
    parse value 'aa bb' with o~v1 w2
    say 'v1=['o~v1']' 'w2=['w2']'
    ::class Kustom
    ::attribute v1

| side | rc | stdout | stderr |
|---|---|---|---|
| oracle | 0 | `v1=[aa] w2=[bb]` | empty |
| here | 120 | empty | `rexx-exec: a message send is not implemented` |

## The control, which narrows it to PARSE

    o = .Kustom~new
    o~v1 = 'aa'
    say 'assign v1=['o~v1']'
    parse value 'cc dd' with w1 w2
    say 'plain w1=['w1']'
    ::class Kustom
    ::attribute v1

Both sides rc 0, both `assign v1=[aa]` and `plain w1=[cc]`, byte-identical. So
assigning through a message works outside `PARSE`, and `PARSE` works with
ordinary targets. **The gap is the combination**: a message term used as a
`PARSE` target.

## Why this matters more than its size

The failure is a **loud refusal**, rc 120 with a message on stderr, so it cannot
silently produce a wrong answer. That is the good case. What it says about
coverage is the concern: a `PARSE` target list accepts a grammar of terms, and
this one is refused, which raises the question of what else in that grammar is
accepted by the parser and refused at run time. Nobody has enumerated the target
forms.

## What the work is

Not "implement this one form". First **derive the set of target forms the parser
accepts**, then run each against both sides, then decide. A single fix here would
close one cell of a table nobody has drawn, and the table is the deliverable.

Adding a `corpus/lang/*.rex` case is not free: editing any byte of one reddens
`sourceline_matches_the_interpreter_for_every_corpus_program`, whose recorded
file holds that program's line count and every line verbatim. The regeneration
driver is named in that test's module comment.
