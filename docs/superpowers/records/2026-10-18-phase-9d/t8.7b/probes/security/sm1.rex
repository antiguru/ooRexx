m = .stem~method('UNKNOWN'); m~setProtected
.context~package~setSecurityManager(.sm~new)
call r
say 'main prot' .p~new~pm
call rt
exit
r:
s. = 'abc'
say 'len' s.~length
say 'direct' s.~unknown('LENGTH', .array~new)
o = .p~new
say 'prot' o~pm
return
::routine rt
say 'rt prot' .p~new~pm
::class sm
::method unknown
  use arg name, args
  say 'sm' name args[1]~name
  return 0
::class p
::method pm protected
  return 'pm-ran'
