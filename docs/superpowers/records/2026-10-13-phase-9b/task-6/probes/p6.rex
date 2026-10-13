parse arg cat
call t "SysGetMessage(1, cat)"
call t "SysGetMessage(2, cat, 'a', 'b')"
call t "SysGetMessage(3, cat)"
call t "SysGetMessage(4, cat)"
call t "SysGetMessageX(2, 5, cat)"
call t "SysGetMessageX(2147483647, 1, cat)"
call t "SysGetMessageX(2147483646, 1, cat)"
call t "SysGetMessageX(4294967295, 1, cat)"
call t "SysGetMessageX(4294967296, 1, cat)"
call t "SysGetMessage(4294967295, cat)"
call t "SysGetMessage(4294967297, cat)"
call t "SysGetMessage(1, 'w.cat')"
call t "SysFormatMessage('x', .array~of(.r1~new, .r2~new))"
exit
t:
  parse arg expr
  signal on syntax name err
  interpret 'r =' expr
  say expr '->' '['r']'
  return
err:
  say expr '-> syntax' condition('o')~code ':' condition('o')~message
  return
::class r1
::method string; raise syntax 93.900 array('one')
::class r2
::method string; raise syntax 93.901 array('two')
