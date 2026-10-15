o = .k~new
o~put('X', .k~method('BODY')~copy)
say o~x
o~put('Y', .k~method('BODY')~copy)
say o~y
exit
::class k
::method put
  use arg n, m, s = "FLOAT"
  self~setMethod(n, m, s)
::method body
  .local['W'] = .WeakReference~new(.context~executable)
  h1 = .context~executable~identityHash
  self~unsetMethod(.context~name)
  call filler
  say 'swept' (.local['W']~value == .nil)
  say 'same' (h1 = .context~executable~identityHash)
  say 'exe' .context~executable~class~id .context~executable~isGuarded .context~executable~isPrivate
  say 'srcitems' .context~executable~source~items
  return 'ok' .context~name
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
