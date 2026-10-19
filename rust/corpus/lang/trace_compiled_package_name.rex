/* A >I> or <I< line names the package of code built from source text by the
   name it was built under: Routine~new, Method~new, and Package~new for its
   prologue and for a routine it defines. */
r = .Routine~new('rnamed', 'trace i; return 1')
say 'routine' r~call
o = .holder~new
o~put('M1', .Method~new('mnamed', 'trace i; return 2'))
say 'method' o~m1
o~put('M2', .Method~new('optnamed', 'return 3; ::options trace i'))
say 'options' o~m2
p = .Package~new('pnamed', ('trace i', 'nop', '::routine q public', 'trace i', 'return 4'))
say 'package routine' p~findRoutine('q')~call
say 'names' r~package~name p~name

::class holder
::method put
  use arg name, method
  self~setMethod(name, method)
