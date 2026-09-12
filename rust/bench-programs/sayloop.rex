/* The output-route dimension: a loop whose body is one SAY, so the
   measurement is the per-SAY cost and almost nothing else. No other axis
   executes a SAY inside its timed section -- emptyloop says once after its
   loop, and rexxcps' in-loop says are `if ... then say 'FailedN'` guards a
   passing run never takes -- so without this one a change to how SAY reaches
   its destination is invisible to the suite.

   Measured against the same loop with `nop` in place of the SAY: 304
   instructions per SAY here and 7,499 on the oracle, the two bare loops
   within 4% of each other. The count is 100,000 rather than a million
   because the per-SAY figure is identical at 100k, 250k and 1M, and the
   suite keeps every sampled run's stdout in memory to compare them. */
n = 100000
do i = 1 to n
  say 'x'
end
