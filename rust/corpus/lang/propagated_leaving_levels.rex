-- PROPAGATED is 1 once a SYNTAX condition leaves a method or routine, and 0
-- while the raising activation (an internal call, INTERPRET or itself)
-- traps it. RAISE PROPAGATE re-raises the same object in the caller of the
-- first activation that is not an internal call.
signal on syntax name s1
say .a~new~m
s1: say 'method error' condition('O')~propagated condition('O')~position
signal on syntax name s2
call r
s2: say 'routine error' condition('O')~propagated condition('O')~position
signal on syntax name s4
x = 1 + 'z'
s4: say 'own error' condition('O')~propagated
signal on syntax name s5
say .a~new~n
s5: say 'method raise' condition('O')~propagated condition('O')~position
signal on syntax name s6
interpret 'x = 1 + "y"'
s6: say 'interpret' condition('O')~propagated
signal on syntax name s7
.a~new~p
say 'not reached'
s7:
  c = condition('O')
  say 'propagated from a method' c~position c~propagated sigl
  signal on syntax name s8
  .a~new~q
  say 'not reached'
s8:
  c = condition('O')
  say 'propagated from an internal call' c~position c~propagated sigl filespec('n', c~program)
  do l over c~traceback; say ' ' l; end
  do f over c~stackframes; say ' ' f~type f~line; end
  signal on syntax name s9
  .a~new~w
  say 'not reached'
s9:
  say 'the method''s own trap is passed' condition('O')~position sigl
  signal on syntax name s3
  call i
s3:
  say 'internal error' condition('O')~propagated condition('O')~position
  exit
i: x = 1 + 'q'; return
::routine r
  x = 1 + 'q'
::class a
::method m
  x = 1 + 'q'
::method n
  raise syntax 40.1 array('x')
::method p
  signal on syntax name h
  x = 1/0
  return
h:
  say 'own trap' condition('O')~code sigl condition('O')~propagated
  raise propagate
::method q
  signal on syntax name h
  call deeper
  return
deeper: procedure
  x = 1/0
h:
  raise propagate
::method w
  signal on syntax name mt
  call sub
  return
mt: say 'not reached'; return
sub:
  signal on syntax name hs
  x = 2/0
  return
hs:
  raise propagate
