/* The explicit `DO LABEL name` form, which is not the same thing as a
   clause label. A clause label (`outer: do i = 1 to 3` then `leave outer`)
   is rejected with error 28.3 -- it names a SIGNAL target, not a loop --
   and LEAVE/ITERATE otherwise only accept a loop's own control variable
   (see leave_nested_outer.rex). `DO LABEL name` is the third way: it names
   a block or loop directly, independent of any control variable, and
   works even on a plain non-repetitive block. This is the only corpus
   program that constructs the parser's Loop::label field. */

/* DO LABEL on a plain, non-repetitive block: LEAVE by that label exits it
   early, same as it would exit a loop. */
do label blk
  say "blk-a"
  leave blk
  say "blk-b"
end
say "after blk"

/* DO LABEL on a controlled loop, LEAVE by that label from a nested loop.
   The outer loop's control variable keeps whatever value it held at the
   moment of the LEAVE. */
do label outer i = 1 to 3
  do j = 1 to 3
    if j = 2 then leave outer
    say "leave-outer" i j
  end
end
say "after leave-outer loop" i j

/* DO LABEL on a controlled loop, ITERATE by that label from a nested loop:
   every outer pass is cut short at j = 2, so "outer2-after" never prints. */
do label outer2 i = 1 to 3
  do j = 1 to 3
    if j = 2 then iterate outer2
    say "iterate-outer" i j
  end
  say "outer2-after" i
end
