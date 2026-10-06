-- object dropped in a started activity; no forced collection
m = .w~new~start('run')
m~wait
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
  o = .k~new('dropped')
  drop o
  say 'activity end'
