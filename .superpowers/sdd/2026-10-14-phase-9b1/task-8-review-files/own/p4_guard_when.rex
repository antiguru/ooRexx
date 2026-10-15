.local['WAITING'] = 0
o = .k~new
o~put('W', .k~method('WAITER')~copy)
o~put('R', .k~method('RELEASE')~copy)
r = o~start('W')
do i = 1 to 40 until .local['WAITING'] = 1
  call syssleep 0.05
end
say 'waiting' .local['WAITING']
o~r
say r~result
say o~hasMethod('W') o~hasMethod('R')
exit
::class k
::method put
  use arg n, m, s = "FLOAT"
  self~setMethod(n, m, s)
::method waiter
  expose flag
  flag = 0
  .local['WAITING'] = 1
  guard on when flag = 1
  call filler
  say 'exe' .context~executable~class~id .context~executable~isGuarded
  f = .context~stackFrames[1]
  say 'frame' f~name f~executable~class~id
  return 'woke'
::method release
  expose flag
  self~unsetMethod('W')
  self~unsetMethod('R')
  call filler
  flag = 1
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
