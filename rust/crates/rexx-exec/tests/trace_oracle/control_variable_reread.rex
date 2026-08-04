/* A Controlled loop re-READS its control variable on every re-tested pass
 * (4b Task 9, review round 1 F2). The body assigns 10, so the oracle reads
 * 10 back, adds 1, and 11 > 3 ends the loop after ONE pass -- both the
 * trip count and the >V> line depend on the read.
 *
 * An implementation that reuses the loop's own saved value instead runs
 * three passes, prints 4, and traces >V>     II => "1".
 *
 * The second loop is the adjacent passing case and is not decoration: its
 * body never touches JJ, so reading the variable back and reusing a saved
 * value agree exactly. A witness with only the first loop in it would pass
 * against an implementation that read the variable but got the ordinary
 * case wrong. */
trace i
do ii = 1 to 3
  ii = 10
end
say ii
do jj = 1 to 2
  nop
end
say jj
