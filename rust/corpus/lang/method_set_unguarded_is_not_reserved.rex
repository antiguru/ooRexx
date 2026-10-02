/* A guarded method made unguarded by setUnguarded takes no lock: two
   activities run it interleaved. */
.k~method('G')~setUnguarded
o = .k~new
a = o~start('g', 'A')
call syssleep 0.1
b = o~start('g', 'B')
a~wait; b~wait
say 'isGuarded' .k~method('G')~isGuarded
::class k
::method g
  use arg n
  say n 'in'
  call syssleep 0.3
  say n 'out'
