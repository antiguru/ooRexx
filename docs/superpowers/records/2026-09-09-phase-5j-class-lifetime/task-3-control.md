# Phase 5j Task 3 — the negative control

The mutation: `Body::Class { owned } => out.extend(owned.iter().copied())` in `Body::trace`
becomes `Body::Class { .. } => {}`, so a class stops keeping its own payload alive.

## The prediction, written before the run

1. The stress mode reddens: a class variable pool, a `Method` object or an annotation table is
   swept while its class is still alive.
2. The ordinary suite stays green, because nothing else collects often enough — so the collection
   is the only thing that sees this.

## The result

**1. CONFIRMED**, and the panic names the subject rather than a downstream symptom:

```
thread 'rexx-interp' panicked at crates/rexx-exec/src/lib.rs:7455:14:
an exposed variable's owner is a rooted Body::Instance
```

Two tests reddened, `the_l0_subset_passes_again_under_collect_on_every_allocation` and
`a_parked_reply_keeps_its_variables_across_a_collection`. Both live in `collect_stress.rs`; they
are two names in one binary, which is why the run reports two `FAILED` result lines for three
names.

**2. CONFIRMED.** Nothing outside `collect_stress.rs` moved. The only other failing name,
`the_table_holds_every_constructor_the_source_defines`, was already red before the mutation — it is
`corpus/refusal-sites.tsv`'s line-keyed drift from this task's own edits, and it is re-derived as
the last step before the commit.

So the answer to the question the plan asked is that the collection is the only thing that sees it.
That is the argument for the stress mode existing: without it this task would have had no witness
at all, and the payload would have sat unrooted for as long as classes stayed immortal.
