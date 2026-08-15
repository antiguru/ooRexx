STATUS: DONE

# Task 5 review: stems and compound variables

Reviewing commit `28a62383`, "Stems: tombstones, aliasing, and a name the object
carries itself". Reviewed from a clean export of the commit, not the working
tree, since `rexx-exec` carries live Task 6 work.

## Verdicts

**Spec compliance: PASS with one Important defect.** Every rule D15a states is
implemented, and the design correction the implementer made under implementation
pressure is right and better-evidenced than the design it replaced. One
interaction between two of those rules is wrong.

**Code quality: PASS.** Clear, the doc comments carry the measurements, borrows
are handled deliberately, the `#[allow(dead_code)]` is scoped and justified.

**1 Important, 2 Minor. Nothing Critical**, because the defect is currently
unreachable: none of these functions is wired into `eval` or `step` yet.

16 unit tests in `stem.rs`, all passing on the clean export, plus 10 integration
and 2 doctests elsewhere in the crate.

---

## I1 (Important). A derived tail name comes from the read site, where the oracle takes it from the object

**This is the pairwise interaction the dispatch asked me to hunt: rule 3, a name
carried on the object, against rule 2, an object shared by aliasing.** Each is
tested alone. Together they reach a state no test does.

`stem_get` uses its `stem_name` parameter for two different jobs: finding the
slot, where the read site's spelling is correct, and deriving the name of an
unresolved tail, where it is not.

```rust
None => return self.derived_tail_name(stem_name, key),   // unset stem
...
None => self.derived_tail_name(stem_name, key),          // tombstone or absent
```

and `derived_tail_name` simply concatenates what it was given:

```rust
let mut name = stem_name.to_vec();
name.extend_from_slice(key);
```

Once `b.` and `a.` share one object, the two jobs want different names.

**Measured on the oracle**, four cases, all wrapped:

```rexx
a.1 = 'x' ; b. = a.        ; say b.2   ->  A.2      (not B.2)
c.1 = 'x' ; d. = c. ; drop d.1 ; say d.1  ->  C.1   (not D.1)
g.1 = 'x' ; h. = g. ; k. = h.  ; say k.9  ->  G.9   (two hops, still G)
m.1 = 'x' ; n.2 = 'y' ; m. = n. ; say m.1 ->  N.1   (not M.1)
```

**Measured against this commit**, by adding one test to a scratch export rather
than inferring from the code:

```
ORACLE=A.2  OURS=B.2
assertion `left == right` failed: derived name must come from the object
  left: [66, 46, 50]      // B.2
 right: [65, 46, 50]      // A.2
```

So the module's own rule 3 — "a stem carries its own name ... never a copy of
the read site's spelling" — is applied correctly in `value.rs`'s `to_text`, for
a *bare* stem read, and not applied in `stem_get`, for a *tail* read. Both
derive a name; only one asks the object.

**Why no test catches it.** `bare_stem_assignment_shares_the_object_when_the_value_is_already_a_stem`
is the one test that aliases, and every read in it *resolves* — through the
shared default, or through a tail that was written. `derived_tail_name` is
reachable only when a tail does **not** resolve, so the aliasing test and the
tombstone test never meet. The fourth probe above is the sharpest version:
`m.1` was set through `m.`'s own object, then `m. = n.` discards that object
entirely, and the name that comes back is `N.1`.

**The control case that shows it is specifically about the read site**, and
which the current code gets right by coincidence:

```rexx
p.1 = 'x' ; r. = p. ; drop r.1 ; say p.1   ->  P.1
```

Here the read site and the object agree, so both models answer `P.1`.

**Suggested fix**, small and local: in `stem_get`, once the stem object has been
resolved, derive from the object's own `name` field rather than from
`stem_name`. The unset-stem early return must keep using `stem_name`, and that
is correct rather than an inconsistency — there is no object to ask, which is
also why `say never_touched.5` answering `NEVER_TOUCHED.5` stays right.

**Severity.** Important and not Critical because no program can observe it
today: the `#[allow(dead_code)]` on the `impl` block records that nothing
outside the module's own tests calls any of this yet. It becomes wrong output
the moment Task 9 wires `ExprKind::Compound` into `eval_node`, and the existing
tests will still pass when it does.

## m1 (Minor). `unreachable!` formats a whole `Body` with `{:?}`

Three sites do `unreachable!("a stem-named slot holds only Body::Stem, got
{:?}", object.body)`. A `Body::Stem` carries its whole tails map, so this
formats an unbounded structure into a panic message.

I checked whether the arm is genuinely unreachable and it is, structurally:
only `stem_assign`, `stem_set` and `replace_stem` write a stem-named slot, and
each writes either a freshly allocated `Body::Stem` or a value `is_stem` has
already confirmed. So this is about the message, not the logic.

Same class as the defect Task 3d fixed in `Loud::expression`, where `{kind:?}`
on an `ExprKind` produced 373,332 bytes. Lower stakes here, since this is an
abort path rather than a reportable failure, but naming the variant rather than
formatting the value costs nothing.

## m2 (Minor). Rule 4's case-sensitivity is tested in one direction only

`tail_keys_are_verbatim_and_case_sensitive` covers a variable piece whose value
is lower case not being upcased. I probed the other direction and it holds:

```rexx
i = 'abc' ; v.i = 'val' ; drop v.ABC ; say v.i   ->  val
```

Dropping the literal-spelled `V.ABC` does not tombstone the key `abc`, because
they are different keys. That is the interaction of rule 4 with rule 1 and it
is correct in the implementation, since both paths go through `tail_key`. Worth
a test so the property is pinned rather than true by construction — it is one
line beside the existing case-sensitivity test.

---

## What I verified and found correct

* **The design correction is right, and better evidenced than the design it
  replaced.** `stem_assign` sharing the object when the value is already a
  `Body::Stem` is the only model that produces `a.=1; b.=a.; a.1=2; say b.1`
  giving `2`. I re-ran the follow-up too: after `a. = 9`, `b.` still answers
  `1` and `b.1` still answers `2`, which distinguishes "shares the object" from
  "references the slot" — a wrapper model would need `stem_get` to chase a
  nested default, a rule D15a never states.
* **Two alias hops keep the original object's name** (`G.9` above), which the
  share-the-object model gets right for free and a wrapper model would not.
* **Rule 1, the tombstone, is correct**: `Some(None)` resolves to `None` and
  does not fall through to `default`, with the three-way match on
  `tails.get(key)` making the distinction explicit rather than incidental.
* **`stem_drop_tail` on an untouched stem is a genuine no-op**, and the doc
  comment's argument for why is sound: an absent key and a tombstone with no
  default render identically, so there is nothing to observe.
* **`stem_drop`'s reasoning about `set_slot` was correct when written** and has
  since been overtaken in a good way — Task 2b added `RootSet::clear_slot`, so
  the "no way to write unset back into a slot" wall it describes is gone. The
  comment is not wrong about stems, since replace-and-rebind is what the oracle
  does regardless, but the sentence "General `DROP` on a *simple* variable will
  hit the identical wall and needs a `rexx-core` amendment; that is a later
  task's" now describes work that is done.
* **`is_stem` rejects `.nil` and `SmallInt` before the heap lookup**, so a
  small integer can never be mistaken for a stem.
* **The borrow discipline holds**: `stem_get` computes `resolved` fully before
  any further `self` call, with a comment saying why, which is the same shape
  Task 3's spike established.

## Method

Read the commit rather than the working tree. Every oracle probe wrapped as
`( ulimit -v 1048576; build/bin/rexx FILE )`. The I1 divergence was **measured
against the committed code** by adding one test to a scratch export outside the
repository, not inferred from reading — the export was discarded afterwards and
nothing in the repository was touched.

Pairs considered: tombstone x aliasing (I1), name x aliasing (I1), verbatim x
tombstone (m2, correct), verbatim x aliasing (no interaction: keys are keys),
tombstone x default (tested, correct), name x whole-stem replacement (tested,
correct). The three-way tombstone x aliasing x name is the same defect as I1
and is the case the fourth probe reaches.
