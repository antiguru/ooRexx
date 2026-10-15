.local['WAITING'] = 0
o = .k~new
o~put('W', .k~method('WAITER')~copy)
o~put('R', .k~method('RELEASE')~copy)
r = o~start('W')
do i = 1 to 40 until .local['WAITING'] = 1
  call syssleep 0.05
end
o~r
say r~result
exit
::class k
::method put
  use arg n, m, s = "FLOAT"
  self~setMethod(n, m, s)
::method waiter
  expose flag
  flag = 0
  .local['WK'] = .WeakReference~new(.context~executable)
  h1 = .context~executable~identityHash
  .local['WAITING'] = 1
  guard on when flag = 1
  call filler
  say 'swept' (.local['WK']~value == .nil)
  say 'same' (h1 = .context~executable~identityHash)
  say 'guarded' .context~executable~isGuarded
  return 'woke'
::method release
  expose flag
  self~unsetMethod('W')
  call filler
  flag = 1
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
