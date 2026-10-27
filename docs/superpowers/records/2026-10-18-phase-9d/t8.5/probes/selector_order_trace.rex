o = .k~new
o~setup
o~m
a = 'p'; b = 'q'
call pe
drop (a) (b)
exit
pe: procedure expose (a) (b)
  return
::class k
::method setup
  expose a b
  a = 'x'; b = 'y'
::method m
  expose (a) (b)
::options trace i
