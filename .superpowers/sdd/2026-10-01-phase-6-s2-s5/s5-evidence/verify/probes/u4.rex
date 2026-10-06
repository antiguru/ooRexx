-- object dropped in a started activity; collection forced by allocation, then more returns
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
  do i = 1 to 200000
    s = .array~new(10)
  end
  say 'activity loop done'
  call f
  say 'activity end'
  return
f: return
