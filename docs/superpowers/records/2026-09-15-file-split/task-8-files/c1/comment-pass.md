# c1 comment pass: `invoke.rs`'s `mod tests`

Source: `c1-comments.txt` (comments7.py, which now lists positional words in
the parent as well as the destination) and `c1-positional.txt`. Every hit,
and why it is still true:

* (a) `ffi.rs:756`, "the array is `invoke::run`'s": the production `run`
  in `invoke.rs:101`, which did not move. The test module's helper of the
  same leaf name is `invoke::tests::run`, before and after.
* (a) `invoke.rs:15`, `:32`, `:34`, `:73`: the module doc and the docs of
  `method` and `routine`, production text naming `NativeActivation::run`,
  `entry`, `arguments` and [`method`]; they match only because the test
  module has items with those leaf names (`fn run`, `Host::arguments`).
  Nothing they describe moved.
* (a) `invoke/tests.rs:67` ([`LONGEST`]), `:167` (`double_object`),
  `:169` (`whole_number`), `:174` (`arguments`), `:411`, `:441`
  ([`run`]), `:786` (`arguments`): each names an item of the test module
  itself, which moved whole; same module before and after, so every name
  and intra-doc link resolves to what it did.
* (b): none. No comment in `rust/crates` and no line in `rust/corpus`
  names `invoke.rs` or `invoke/`.
* (c) `invoke.rs:131`, "never held across the call below": inside
  `run`, about the `method`/`routine` call later in the same function;
  production, unmoved.
* (c) `invoke/tests.rs:787`, "answers the outcome beside the bytes": the
  pair `run_stub` returns, not a position in the file.
* positional.py: `invoke/tests.rs:786-788`, the same doc as the line
  above, names `method` (the production function it calls, which is in
  `invoke.rs`), with "beside" meaning the tuple.
