-- live object at end; a started activity still running when main ends
o = .k~new('live')
.w~new~start('run')
say 'main end'
::class k
::method init
  expose n
  use arg n
::method uninit
  expose n
  say 'uninit' n
::class w
::method run
  call SysSleep 0.3
  say 'activity end'
