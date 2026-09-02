# Task 3 fix round -- brief

Findings from `task-3-review.md`: 1 Critical, 3 Important, 6 Minor. BASE is the commit named in your
dispatch. The tree is clean and you are its only writer.

## CRIT-1, the one that changes what ships: three levels, not two

**Verified independently by the controller** at `e30055b37`, its own probe, both engines, three
descriptors read separately, rc 0 and empty stderr on every side:

```
t = .StringTable~new ; t['MM'] = "return 'enhanced'" ; e = .K~enhanced(t)
say 'a' e~mm  /  e~cover (self~setMethod)  /  say 'b' e~mm
                e~uncover (self~unsetMethod) / say 'c' e~mm / say 'd' e~hasMethod('MM')

oracle          a enhanced / b one-off / c enhanced   / d 1
ir, tree-walker a enhanced / b one-off / c from-class / d 1
```

`c` is a **silent wrong answer**. `unsetMethod` deletes the enhanced method where the oracle reveals
it, because `Class~enhanced`'s methods are stored in the same per-object dictionary `setMethod`
writes. The oracle keeps them in a dummy subclass's behaviour (`ClassClass.cpp:1454`-`:1461`), and
`removeInstanceMethod` only removes what the object's own `instanceMethods` table held
(`MethodDictionary.cpp:363`-`:371`). **Three levels: the object's one-offs, then the enhanced
behaviour, then the class.** The crate has two.

The code comment that justifies the current storage quotes `ClassClass.cpp:1457` -- "look like they
were added with setMethod" -- which is about the `.nil` SCOPE, a thing this crate already gets right,
and not about which dictionary holds them. Fix the comment with the code.

Read the C++ yourself before building; the review's citations are a claim, not a premise. Then decide
where the middle level lives, and say what you rejected.

## IMP-1, the fifth instance of a witness that cannot fail

`setmethod_hidden.rex`'s own comment says "the send is 97.1 even though the class defines the method",
and **the row never makes that send**. Arm HIDESEND (lookup matches only `Some(Some(..))`, so a
hidden name falls through to the class, leaving `hasMethod` and `answers_uninit` alone) leaves the
corpus at 298 of 298 and exit 0. One added send fixes it. Re-run that arm yourself, add the send, and
show the row reddening under it.

## IMP-3, a second missing witness

A scope override that skips the object's dictionary has no witness either. The **behaviour is right**
-- `o~mm:.k` and a probed `self~mm:super` both agree -- but arm SCOPEOVR (drop
`start_scope.is_none()`) leaves 298 of 298 at exit 0. Add the row.

## IMP-2, the unreclaimed synthetic program, now measured

`maxrss`, `REXX_ENGINE=ir`: an empty program is 16,524 KB; 20,000 `setMethod` calls under 20,000
distinct names reach 385,032 KB against the oracle's 111,904; **20,000 calls under one name, each
replacing the last, reach 383,452 KB against the oracle's 20,588, which is flat**; and 20,000
re-attachments of one pre-built `Method` object stay at 18,444 KB, also flat. About 18.4 KB per
compiled source, linear at 200, 2,000 and 20,000. That fourth row is the control, and it says the
cost is `compile_method_source`'s synthetic program and not the per-object dictionary.

**The superseded case is the one to answer**: replacing a name should not retain the program it
replaced. Either reclaim it, or license the divergence explicitly with this measurement recorded
beside it -- and if you license it, say what a long-running program pays. Do not leave it as the
report's one-line "never reclaimed".

## Minors

1. `Loud::method_from_source`'s class-side bullet still says `install_enhancing_methods` "declines the
   install rather than answering running `<path>`" -- the reason this task removed. Its divergence (a
   source-string enhancing table: oracle `Error 42 running M line 1:` rc 214, crate rc 120) is on no
   open list.
2. Two more unlisted loud divergences: an unparseable `setMethod` source (oracle
   `Error 35 running MM line 1:` rc 221), and `Object~RUN` from a method context.
3. `install_object_model`'s "the lowest identities any run holds" is false as written: `Setup.cpp`
   builds `Class` at `:455` before `Object`'s private rows at `:549`.
4. `install_enhancing_methods` and `install_enhanced_methods` differ by one character and do different
   things.
5. A dead arm in `write_object_method`.
6. `program_display_name` has one caller while the traceback reads `compiled_method_names` directly.

## Standing constraints

All of `global-constraints.md`. Oracle probes from a fresh empty directory; three descriptors read
separately, never `2>&1`; both engines; no `unsafe`. **Nothing in `rust/corpus/oracle-crashes.txt`**,
which gained an entry today: a `FORWARD` without `CONTINUE` that resolves back to its own method
SIGSEGVs the oracle. Mutation arms in `git archive` extracts with their own `CARGO_TARGET_DIR`; an
extract has no `ootest/`, so compare failing **sets**, never a count against zero.

Five gates from `rust/`, each status read unpiped from its own file, never chained with `&&`, plus the
phase-gate command. A `src/` change owes a sitting against `bench-baselines/pinned/rexx-run-f558ea501`
with `--baseline bench-baselines/phase-5b-arms.tsv`; `rexx-arms` **appends** to whatever `--baseline`
names, and `rexxcps` needs a path axis. Report to `task-3-fix-report.md`, file first, append as you go.
Message the controller when you finish.
