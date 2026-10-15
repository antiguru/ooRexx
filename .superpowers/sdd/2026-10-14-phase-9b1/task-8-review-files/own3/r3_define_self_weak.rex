.k~define('D', .k~method('BODY')~copy)
o = .k~new
say o~d
exit
::class k
::method body
  .local['W'] = .WeakReference~new(.context~executable)
  h1 = .context~executable~identityHash
  .k~define('D')
  call filler
  say 'swept' (.local['W']~value == .nil)
  say 'same' (h1 = .context~executable~identityHash)
  return 'ok'
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
