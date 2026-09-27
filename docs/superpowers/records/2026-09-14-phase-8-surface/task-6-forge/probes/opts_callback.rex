/* the conditions Activity::raiseCondition escalates inside the callback */
call AddCmd 'xd', 'd'
address system 'exit 3'
signal on syntax name l1
address xd 'RAISE NOTREADY'
say 'after notready' rc .rs
l1: call show 'notready'
signal on syntax name l2
address xd 'RAISE LOSTDIGITS'
say 'after lostdigits' rc .rs
l2: call show 'lostdigits'
signal on syntax name l3
address xd 'RAISE NOSTRING'
say 'after nostring' rc .rs
l3: call show 'nostring'
signal on syntax name l4
address xd 'RAISE USER FOO'
say 'after user' rc .rs
l4: call show 'user'
signal on syntax name l5
address xd 'RAISE ERROR'
say 'after error' rc .rs
l5: call show 'error'
signal on syntax name l6
address xd 'RAISE NOVALUE'
say 'after novalue' rc .rs
l6: call show 'novalue'
signal on syntax name l7
call on notready name onnotready
address xd 'RAISE NOTREADY'
say 'after trapped notready' rc .rs
l7: call show 'trapped notready'
exit
show:
  c = condition('o')
  say arg(1)':' c~condition c~code c~message .rs rc
  return
onnotready:
  say 'onnotready' condition('d')
  return
::requires 'cmd' LIBRARY
::options notready syntax lostdigits syntax nostring syntax novalue syntax
