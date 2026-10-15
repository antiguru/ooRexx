w = .w~new
m = w~start('hold')
w~ready  -- waits until hold has published its reference
r = .local['R']
call filler
say 'p04 read' length(r~value) left(r~value, 3)
drop r
.local['R'] = .nil
call filler
w~go = 1
say 'p04 result' m~result
exit
::class w
::attribute go
::method ready
  expose pub
  guard on when pub = 1
::method hold
  expose go pub
  y = copies('hh', 19)
  .local['R'] = >y
  go = 0
  pub = 1
  guard on when go = 1
  return length(y) left(y, 3)
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
