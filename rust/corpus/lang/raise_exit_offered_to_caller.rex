-- A tail-less or EXIT RAISE of a condition other than SYNTAX leaves every
-- internal call and is offered, as RAISE ... RETURN is, to the caller of the
-- first activation that is not one; an untrapped HALT or NOMETHOD is then a
-- SYNTAX error of that activation.
call on user foo name h
call on halt name hh
t = .T~new
say 'value' t~viaInternal
t~halts
say 'after halts'
call viaRoutine
say 'routine' result
say 'interpreted' interpreted()
signal on user bar name sb
t~signals
say 'not reached'
sb:
  say 'signalled' condition('C') sigl
  call off halt
  signal on syntax name sx
  t~halts
  say 'not reached'
sx:
  c = condition('O')
  say 'halt as syntax' c~code c~position sigl
  signal on syntax name sx2
  call internalHalt
  say 'not reached'
sx2:
  c = condition('O')
  say 'internal halt' c~code c~position sigl
  t~haltReturns
  signal on syntax name sx4
  call noMethod
  say 'not reached'
sx4:
  c = condition('O')
  say 'nomethod' c~code c~position condition('D')
  signal on syntax name sx5
  call noMethodBare
  say 'not reached'
sx5:
  say 'nomethod bare' condition('O')~code
  call on user zz name hz
  call mainLevel
  say 'not reached'
  exit
h: say 'handler' condition('C') condition('D'); return
hh: say 'halt handler' condition('C'); return
hz: say 'hz not reached'; return
internalHalt:
  raise halt
mainLevel:
  raise user zz exit
::routine viaRoutine
  call sub
  say 'not reached'
  return 1
sub:
  raise user foo description 'dd' exit 'rv'
::routine interpreted
  interpret 'raise user foo exit 5'
  say 'not reached'
::routine noMethod
  raise nomethod description 'FOO' additional 'bar' return
::routine noMethodBare
  raise nomethod return
::class T
::method viaInternal
  call sub
  say 'not reached'
sub:
  raise user foo exit 'ex'
::method halts
  raise halt
::method signals
  raise user bar
::method haltReturns
  signal on syntax name s
  raise halt return 'x'
  return
s:
  say 'own trap' condition('O')~code sigl
