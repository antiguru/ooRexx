# c1 comment pass: `block.rs`'s variable-reference walker

Source: `c1-comments.txt` (comments7.py with the widened word list, the
parent listed as well as the destination) and `c1-positional.txt`. The moved
text is BASE `block.rs:599`-`813` (`for_each_variable_name` and the
`visit_*` helpers), which sat between `Block::finish` and `opens`. Every
hit, and why it is still true:

* (a): none. No comment names a moved unit in backticks outside the moved
  text, and none in the corpus reaches one through `block::`.
  (`block/tests.rs:476` names `visit_refs`; comments7.py does not list it,
  since an unqualified name outside the parent and destination names that
  file's own item. Read by hand: "which is why `visit_refs` handles both
  `VariableRef` variants" is about `visit_refs`'s body, which moved whole,
  and names no file.)
* (b) `rexx-exec/src/run.rs:721`, "confirmed by tracing `block.rs` by
  hand": what it traced is where a THEN's `false_target` lands, which
  `set_false_target` and `translate_block` decide, and both stay in
  `block.rs`.
* (b) `rexx-exec/src/run/tests/branch.rs:81`, "The `ast.rs`/`block.rs`-derived
  discriminating case: an `IF`/`ELSE`": the same assembly, which stays.
* (c), a position in Rexx source, in the control stack or in the chain,
  not in this file: `block.rs:28` (bottom of the stack), `:45` (an ELSE
  may still follow), `:137` (bottom of the stack), `:249` and `:251`
  (a reference preceding the guard expression, a later duplicate), `:262`
  (an ELSE following a label), `:270` (after everything added), `:353`
  (end of the body), `:363` and `:368` (right after the THEN, end of
  file), `:474` (after the END), `:527` (before this switch: execution
  order), `:533` (the SELECT behind an OTHERWISE), `:553` (one past the
  end of the chain), `:612` (joins the chain immediately), `:619` (the
  end of the source), `:648` (end of the body), `:761` (immediately in
  front of an ELSE).
* (c), "here" meaning the code it sits in or this parser, all unmoved:
  `:77` (an instruction here, in this assembler), `:126` (`referenced`'s
  doc: observable from this parser), `:246` (`add_clause`, whose call to
  `for_each_variable_name` is unchanged), `:362` (`block_error`'s arm),
  `:453` (`match_select_end`), `:522` (`match_end`), `:762` (the label
  check in `translate_block`).
* (c), a pointer to other code inside `translate_block`, which is unmoved
  and in which no line moved: `:668` (the top of the loop), `:670` (the
  SELECT-membership check below), `:698`-`:699` (further down, step 1
  below, here and there), `:714` (its own arm below).
* (c) in `block/references.rs`: none.
* positional.py: nothing.
* The module doc of `block.rs`, "Block structure: the control stack, and
  assembling one code body", stays true: it never named the walker.
  `Block::symbols`'s doc ("so that a variable slot's `SymbolId` can be
  resolved to the name `referenced` is keyed by") and `referenced`'s doc
  stay true: `add_clause` still fills `referenced` through
  `for_each_variable_name`, now imported from the child.
