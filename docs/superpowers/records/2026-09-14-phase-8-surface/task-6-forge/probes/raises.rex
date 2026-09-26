/* A condition a direct handler raises through its thread context. */
call AddCmd 'xd', 'd'
address xd 'RAISE ERROR'
say 'error untrapped' rc .rs
address xd 'RAISE FAILURE'
say 'failure untrapped' rc .rs
address xd 'RAISENORES ERROR'
say 'error nores' rc .rs
address xd 'RAISE USER FOO'
say 'user untrapped' rc .rs
call on error name onerror
address xd 'RAISE ERROR'
say 'after call on error' rc .rs
address xd 'RAISE FAILURE'
say 'after failure to error' rc .rs
call off error
call on user foo name onuser
address xd 'RAISE USER FOO'
say 'after user' rc .rs
trace e
address xd 'RAISE ERROR'
trace o
signal on syntax
address xd 'SYNTAX'
say 'not here'
exit
syntax:
  c = condition('o')
  say 'syntax' c~code c~message rc
  exit
onerror:
  c = condition('o')
  say 'onerror' c~condition c~rc c~description c~additional c~result rc
  return
onuser:
  c = condition('o')
  say 'onuser' c~condition c~description c~additional c~result rc
  return
::requires 'cmd' LIBRARY
