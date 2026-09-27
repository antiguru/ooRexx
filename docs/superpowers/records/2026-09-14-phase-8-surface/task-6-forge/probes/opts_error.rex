/* each handler condition under ::OPTIONS ERROR SYNTAX alone */
call AddCmd 'xd', 'd'
signal on syntax name l1
address xd 'RAISE USER FOO'
say 'after user' rc .rs
l1: call show 'user'
signal on syntax name l2
address xd 'RAISENORES USER FOO'
say 'after user nores' rc .rs
l2: call show 'user nores'
signal on syntax name l3
address xd 'RAISE NOTREADY'
say 'after notready' rc .rs
l3: call show 'notready'
signal on syntax name l4
address xd 'RAISE HALT'
say 'after halt' rc .rs
l4: call show 'halt'
signal on syntax name l5
address xd 'RAISE LOSTDIGITS'
say 'after lostdigits' rc .rs
l5: call show 'lostdigits'
signal on syntax name l6
address xd 'RAISE NOVALUE'
say 'after novalue' rc .rs
l6: call show 'novalue'
signal on syntax name l7
address xd 'RAISE NOSTRING'
say 'after nostring' rc .rs
l7: call show 'nostring'
signal on syntax name l8
call on user foo name onuser
address xd 'RAISE USER FOO'
say 'after trapped user' rc .rs
l8: call show 'trapped user'
signal on syntax name l9
address xd
trace c
'RAISE USER FOO'
trace o
address
say 'after traced user' rc .rs
l9: trace o; call show 'traced user'
signal on syntax name l10
call on error name onerror
address xd 'RAISE USER FOO'
say 'after user, error armed' rc .rs
l10: call show 'user, error armed'
exit
show:
  c = condition('o')
  say arg(1)':' c~condition c~code c~message
  return
onuser:
  say 'onuser' condition('d')
  return
onerror:
  say 'onerror' condition('d')
  return
::requires 'cmd' LIBRARY
::options error syntax
