/* Who shares a stream position with whom. The table lives on a program or
   method activation; an internal CALL, a PROCEDURE routine, a ::ROUTINE and an
   INTERPRET all borrow their caller's by reference, so each advances the
   position the caller sees next. A method activation gets its own table and
   its own Stream for the same name -- and a fresh one per call, so two calls
   to the same method both read line one and neither moves the caller. */
say 'stream table sharing'
seed = .Stream~new('sh.txt')
zz = seed~lineout('one')
zz = seed~lineout('two')
zz = seed~lineout('three')
zz = seed~lineout('four')
zz = seed~lineout('five')
zz = seed~lineout('six')
zz = seed~close
f = 'sh.txt'
say 'main     [' || linein(f) || ']'
call inner
say 'main     [' || linein(f) || ']'
interpret "say 'interp   [' || linein(f) || ']'"
say 'main     [' || linein(f) || ']'
say 'routine  [' || rout(f) || ']'
say 'main     [' || linein(f) || ']'
o = .K~new
say 'method   [' || o~m(f) || ']'
say 'method 2 [' || o~m(f) || ']'
say 'main     [' || linein(f) || ']'
exit 0

inner:
  say 'call     [' || linein(f) || ']'
  return

proc_inner: procedure expose f
  say 'procedure[' || linein(f) || ']'
  return

::routine rout
use arg n
return linein(n)

::class K
::method m
use arg n
return linein(n)
