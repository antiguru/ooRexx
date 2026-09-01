/* uninit_nested_collection's question asked of the termination sweep: G's
   finalizer is reached because the program ends rather than because the
   program called GC('force'), so the collection driven inside it arrives at a
   second copy of the interlock, and K's finalizer must still not run inline.
   The padding is load-bearing.  Without it the oracle never collects K's
   instance at all and answers 0 for that reason instead of for the interlock:
   measured, this same finalizer body run as an ordinary class method answers
   1 with the padding and 0 without it.
   When K's finalizer does run is deliberately not printed, because the oracle
   answers that two different ways across runs. */
say 'start'
g = .G~new
say 'end'
exit 0

::class k
::method uninit
  n = .K~mark
::method mark class
  expose ran
  ran = 1
  return 1
::method ran class
  expose ran
  if var('ran') = 0 then return 0
  return ran

::class g
::method uninit
  say 'outer fin'
  o = .K~new
  y1='a'; y2='b'; y3='c'; y4='d'; y5='e'; y6='f'
  y7='g'; y8='h'; y9='i'; y10='j'; y11='k'; y12='l'
  drop o
  call gc 'force'
  say 'inner ran inline?' .K~ran
