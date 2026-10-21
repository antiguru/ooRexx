o = .o~new
o~start('run')
call syssleep 0.3
say 'B' linein()
call syssleep 1.2
say 'C' linein()
say 'D' linein()
::class o
::method run
  'sleep 1'
  say 'slept'
