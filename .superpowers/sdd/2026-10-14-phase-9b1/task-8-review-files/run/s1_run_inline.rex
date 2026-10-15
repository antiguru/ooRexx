o = .k~new
say o~go
say o~gop
exit
::class k
::method go
  return self~run(.k~method('RUNNER')~copy)
::method gop
  m = .k~method('RUNNER')~copy
  m~setPrivate
  m~setUnguarded
  .local['WP'] = .WeakReference~new(m)
  drop m
  return self~run(.local['WP']~value)
::method runner
  call filler
  if .local~hasEntry('WP') then say 'swept' (.local['WP']~value == .nil)
  e = .context~executable
  say 'exe' e~class~id e~isGuarded e~isPrivate e~source~items
  f = .context~stackFrames[1]
  say 'frame' f~name f~type f~executable~class~id (f~executable == e)
  return 'r'
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
