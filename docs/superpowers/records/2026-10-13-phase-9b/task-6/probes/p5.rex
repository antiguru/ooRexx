call t "SysFormatMessage('x', .array~of(.raiser~new, .say~new))"
call t "SysFormatMessage('&2', .array~of(.raiser~new, .say~new))"
call t "SysGetMessage(5, , .raiser~new)"
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
::class raiser
::method string; raise syntax 93.900 array('boom')
::class say
::method string; say 'string of say'; return 'S1'
