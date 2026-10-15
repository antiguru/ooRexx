.k~define('W', .k~method('WAITER')~copy)
o = .k~new
r = o~start('W')
do i = 1 to 20 until o~waiting
  call syssleep 0.05
end
say 'waiting' o~waiting
o~release
say r~result
exit
::class k
::method init
  expose flag waiting
  flag = 0; waiting = 0
::method waiting attribute
::method waiter
  expose flag waiting
  waiting = 1
  guard on when flag = 1
  call filler
  say 'exe' .context~executable~class~id .context~executable~isGuarded
  return 'woke'
::method release
  expose flag
  .k~define('W')
  call filler
  flag = 1
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
