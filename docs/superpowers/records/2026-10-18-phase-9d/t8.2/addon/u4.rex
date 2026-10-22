m = .stem~method('UNKNOWN'); m~setProtected
.context~package~setSecurityManager(.sm~new)
call rt
exit
::routine rt
s. = 'abc'
say 'len' s.~length
say 'direct' s.~unknown('LENGTH', .array~new)
o = .p~new
say 'prot' o~pm
::class sm
::method unknown
  use arg name, args
  say 'sm' name args[1]~name
  return 0
::class p
::method pm protected
  return 'pm-ran'
