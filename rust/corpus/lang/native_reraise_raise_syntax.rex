-- A RAISE SYNTAX inside a callback an extension's method made is re-raised
-- in the method's caller like any other SYNTAX condition: POSITION is the
-- caller's line and the condition is PROPAGATED.
k = .k~new
signal on syntax
x = k~send0(.t~new, 'viaRaise')
exit
syntax:
  c = condition('O')
  say c~code c~position c~propagated c~traceback~items
  signal on syntax name again
  x = k~send0(.t~new, 'viaRaiseReturn')
  exit
again:
  c = condition('O')
  say c~code c~position c~propagated
  signal off syntax
  y = k~send0(.t~new, 'viaRaise')
::class k
::method send0 external "LIBRARY orxmethod TestSendMessage0"
::class t
::method viaRaise
  nop
  raise syntax 93.900 array('via raise')
::method viaRaiseReturn
  raise syntax 40.1 array('RR') return 'x'
