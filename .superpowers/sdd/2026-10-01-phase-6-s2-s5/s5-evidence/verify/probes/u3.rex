-- object dropped in a started activity; GC('force') in that activity
m = .w~new~start('run')
m~wait
say 'main end'
::class k
::method init
  expose n
  use arg n
::method uninit
  expose n
  say 'uninit' n .context~thread
::class w
::method run
  o = .k~new('dropped')
  drop o
  call gc 'force'
  say 'activity end'
