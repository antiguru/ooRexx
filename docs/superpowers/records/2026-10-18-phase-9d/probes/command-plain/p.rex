call time 'R'
o = .w~new
o~start('run')
call SysSleep 0.3
say 'main resumed late:' (time('E') >= 1.5)
call SysSleep 3
::class w
::method run
  address '' 'sleep 2'
  say 'cmd done'
