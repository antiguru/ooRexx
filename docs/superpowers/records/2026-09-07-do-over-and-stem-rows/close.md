# `DO ... OVER` and the three `Stem` rows -- close

Base `da02f3f1e`, landing over `267446a4e`. Plan:
`docs/superpowers/plans/2026-09-07-do-over-and-stem-rows.md`.

Both halves of the goal are done, and the harder half turned out not to be
`DO OVER` at all.

## What moved

`corpus/collection-arity.tsv`: `agree` 422 to **430**, `send-differs` 10 to
**2**. The two left are out of scope and named as such: `Properties save`
(Phase 7 streams) and `Properties setLogical` (`ARG` option `"A"`).
`corpus/method-bodies.txt`: the three `Stem` rows, `loud` to `answers`.
Strict corpus 439 to **441**.

## The three `Stem` rows

All three turn on one field: a stem has a VALUE of its own, separate from its
tails, and an unassigned stem's value is its own derived name -- `a.b = 1`
then `a.~length` is **2**, not 0.

`request` upper-cases its argument; `'ARRAY'` answers the stem's `makeArray`,
which for a `Stem` is its TAILS in the tail tree's post-order, and every
other name is forwarded to the value. A missing argument is the POSITIONAL
93. `toDirectory` answers a `Directory` of one entry per tail that has a
value, in the directory's order rather than the stem's. `unknown` forwards
the message and its arguments to the value.

`a_receiver_with_no_class_here_is_loud` asserted that a `Stem` receiver is
loud "because this phase builds no class for it". `UNKNOWN` is now built, so
the premise is gone and the oracle's own answer replaced the refusal.

## `DO ... OVER`

`requestArray` (`classes/ObjectClass.cpp:1646`) is two paths keyed on
`isBaseClass()`: a base-class object answers `makeArray()` through a direct
call with no message send, anything else is sent `REQUEST` with `'ARRAY'`.
The split is observable four ways, all measured and all in the witness: a
subclass of `Table` overriding `makeArray` answers the override; a subclass
of `Array` doing the same also answers it, which is how `isArray()` shows
itself to be the PRIMITIVE test rather than "an array or a subclass"; a class
overriding `request` answers from `request` with the argument `ARRAY`; and
`makeArray` runs exactly once per loop, asserted by a counter.

It closed **two silent divergences**: `do e over` a two-line string iterated
the whole string once here against the oracle's two lines, and `do e over
.nil` answered `.nil` once at rc 0 against the oracle's 98.913 at rc 158 -- a
wrong answer rather than a missing one.

`.environment` and `.local` keep their refusal deliberately: this crate
models them as a subset, so iterating one differs in MEMBERSHIP and not
merely in order. A class object no longer does -- it reaches `requestArray`,
answers no `MAKEARRAY`, and raises the oracle's own noarray naming itself.

## The defect underneath, which was not this task's

The first attempt at `DO OVER` was reverted because `collect_stress` panicked
rendering a loop item. The cause was **not** in `DO OVER`.
`native_string_makearray` built its line strings inside a `map` closure and
collected them into a `Vec` -- a Rust local, invisible to the collector -- so
every line was swept by the allocation of the next one, and a single-line
string's by the `alloc_with` that built the array to hold it. The
`push_temp` came after the damage.

It was latent because nothing reached it: a short string is INLINE and never
a heap object at all, so `'abc'~makeArray` survives where
`'.RESOURCES'~makeArray` does not. `DO OVER` was simply its first caller on a
heap string under stress. Confirmed on the reverted tree with no `DO OVER`
code present: `'.RESOURCES'~makeArray` then `a[1]` panics, `'abc'` passes,
two long lines panic. The fix is one `push_temp` per line as it is built, and
with it `DO OVER` needed no rooting work of its own beyond the converted
array.

A scan for the same shape -- a fresh allocation gathered into a Rust `Vec`
inside a closure -- found four more, all inside `mod tests`.

## What I got wrong, because the record should carry it

I ruled on the mechanism five times and was wrong every time: the fresh array
is unrooted, the items are unrooted, the extra temps shift the IR register
file, the pre-existing `StringTable` path has the same latent bug, and
finally "a converted array has no root whose lifetime is the loop's". That
last one was committed in the first version of this report and is false --
`RootSet::iter` walks the WHOLE `temps` vector, the IR register file is a
region of it, so `push_temp` roots unconditionally and there is no
per-register bit. Moritz asked whether registers carry such a bit, and
answering it honestly is what showed the report wrong.

Two instruments did the actual work. A driver built outside the repo that
named each program before running it, which localised the failure to one
program in one run. And a prefix bisect **with a positive control** -- the
first bisect cut off a `::routine` the program calls, so every prefix errored
early and every one read as passing: six "OK" lines and no information, until
the control said the untruncated file must fail.

## Gates

Over `548271c4a`: all seven zero, `failed-suites=0` on each of the five
suite-running gates.
