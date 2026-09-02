/* `~start` and `~startWith` answer a Message object holding what the send
   returned. What may be asserted about one is bounded by D68: `~start`'s
   interleaving with the program that started it is not reproducible on the
   oracle, and `~completed` sampled BEFORE `~result` is part of that
   interleaving. So every line below reads `~result` first and `~completed`
   and `~hasError` after it, and no started method writes to stdout.

   The array form of the name reaches the base's method where the plain form
   reaches the subclass's, the same override `~send` carries, and a method
   answering nothing leaves `~result` as `.nil`. */

o = .Sub~new

m = o~start('M', 5)
say 'class' m~class~id m~string
say 'result' m~result
say 'completed' m~completed 'haserror' m~hasError

w = o~startWith('M', (6,))
say 'with-result' w~result
say 'with-completed' w~completed 'haserror' w~hasError

s = o~start(('M', .Base), 7)
say 'scope-result' s~result
say 'scope-completed' s~completed 'haserror' s~hasError

n = o~start('NOVALUE')
say 'novalue-result' n~result
say 'novalue-completed' n~completed 'haserror' n~hasError

::CLASS Base
::METHOD M
  use arg x
  return 'base' x
::CLASS Sub SUBCLASS Base
::METHOD M
  use arg x
  return 'sub' x
::METHOD noValue
  return
