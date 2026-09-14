/* A native method's CSTRING (doparse) and RexxStringObject (does) parameters
   convert an argument by REQUEST('STRING') alone: no STRING send, no default
   name, and a raise inside the argument's MAKESTRING is that raise. */
r = .Re~new('a*b')
signal on syntax name refused
step = 0
next:
step = step + 1
select
  when step = 1 then say 'parse object' r~doparse(.object~new)
  when step = 2 then say 'parse string only' r~doparse(.Stringer~new)
  when step = 3 then say 'parse makestring' r~doparse(.Maker~new)
  when step = 4 then say 'parse nil' r~doparse(.nil)
  when step = 5 then say 'parse raiser' r~doparse(.Raiser~new)
  when step = 6 then say 'match object' r~does(.object~new)
  when step = 7 then say 'match string only' r~does(.Stringer~new)
  when step = 8 then say 'match makestring' r~does(.Maker~new)
  when step = 9 then say 'match nil' r~does(.nil)
  when step = 10 then say 'match raiser' r~does(.Raiser~new)
  when step = 11 then say 'parse array' r~doparse(.array~of('a*b'))
  when step = 12 then say 'match array' r~does(.array~of('aab'))
  otherwise nop
end
if step < 13 then signal next
signal off syntax
say 'untrapped'
say r~does(.Raiser~new)
exit

refused:
say 'step' step 'syntax' condition('O')~code
signal on syntax name refused
signal next

::requires 're.cls'

::class Stringer
::method string
  return 'a*b'

::class Maker
::method makestring
  say 'makestring ran'
  return 'aab'

::class Raiser
::method makestring
  raise syntax 40.1
