call t "SysFormatMessage('&1', .array~of(.nonstr~new))"
call t "SysFormatMessage('&1', .array~of(.num~new))"
call t "SysFormatMessage('x', .array~of(.raiser~new))"
call t "SysFormatMessage('&1', .array~of(.say~new, .say2~new))"
call t "SysFormatMessage('&1', .array~of(.nostring~new))"
call t "SysFormatMessage('&1', .array~of(.nilstr~new))"
call t "SysFormatMessage(.say~new, .array~of(1))"
call t "SysFormatMessage(.nonstr~new, .array~of(1))"
call t "SysFormatMessage(.nilstr~new, .array~of(1))"
call t "SysGetMessage(5, .nonstr~new)"
call t "SysGetMessage(.num~new)"
call t "SysFormatMessage('&1', .mkarr~new)"
call t "SysFormatMessage('&1', .mkstr~new)"
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
::class nonstr
::method string; return .array~of(1)
::class num
::method string; return 7
::class raiser
::method string; raise syntax 93.900 array('boom')
::class say
::method string; say 'string of say'; return 'S1'
::class say2
::method string; say 'string of say2'; return 'S2'
::class nostring
::class nilstr
::method string; return .nil
::class mkarr
::method makearray; return .array~of('m1', 'm2')
::class mkstr
::method makearray; return 'notarray'
