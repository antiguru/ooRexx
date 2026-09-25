# c2 comment pass: `values.rs`'s per-code converters

Source: `c2-comments.txt` (comments7.py, positional words listed in the
parent as well as the destination) and `c2-positional.txt`. Every hit, and
why it is still true:

* (a): none. No comment in `rust/crates` names a moved converter in
  backticks, and none in the corpus reaches one through `values::`.
* (b): none. No comment in `rust/crates` names `values.rs` or `values/`
  by path (`rust/corpus/phase-8.txt:214` names `crates/rexx-api/tests/values.rs`,
  the integration test file, which this task leaves alone; the scan's
  tails exclude a match preceded by `/`, so it is not listed, and it is in
  `files/path-pins-base.txt`).
* (c) `values.rs:368`, "The optional bit is stripped here": `descriptor`'s
  own body, which stayed.
* (c) `values.rs:416`, "`object` is what makes that hold here":
  `CStringPool::intern_for`, which stayed.
* (c) `values.rs:474`, "answering `.nil` is the `None` here": the return
  value of `Host::string_value`, which stayed.
* (c) `values.rs:595`, "The scope above the running method's": a scope,
  not a position.
* (c) `values.rs:624`, "behind the host rather than beside it": the
  locals table's place in the `Host` design, not a position in the file.
* (c) `values/convert.rs:368`, "so here every one is rooted": inside
  `string_object_to_native`, about that function's own registration; it
  moved whole.
* (c) `values/convert.rs:517`, "built here rather than through the host":
  `int_from_native`'s doc about its own body; it moved whole.
* positional.py: nothing (no comment block in either file names a unit
  that is now in the other file together with a positional word).
* The module doc of `values.rs`, "The conversion table an extension's
  declared signature drives", stays true: `TABLE`, its row builders and
  every public lookup over it stay in `values.rs`.
