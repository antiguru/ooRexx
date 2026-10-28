/* An activation runs under the security manager its package had when the
   activation began, and an internal call or INTERPRET under its caller's:
   a manager set or replaced mid-activation is seen by the next routine or
   method activation only. */
m = .stem~method('UNKNOWN'); m~setProtected
.context~package~setSecurityManager(.sa~new)
address system 'exit 5'
say 'main rc' rc
call r
interpret "address system 'exit 7'; say 'interp rc' rc"
say 'main prot' .p~new~pm
call rt
.p~new~go
call swap
exit
r:
  address system 'exit 6'
  say 'internal rc' rc
  s. = 'abc'
  say 'len' s.~length
  say 'direct' s.~unknown('LENGTH', .array~new)
  say 'prot' .p~new~pm
  return
::routine rt
  address system 'exit 3'
  say 'routine rc' rc
  s. = 'abc'
  say 'len' s.~length
  say 'prot' .p~new~pm
::routine swap
  address system 'exit 1'
  .context~package~setSecurityManager(.sb~new)
  address system 'exit 2'
  call s2
  call swap2
  return
s2:
  address system 'exit 3'
  return
::routine swap2
  address system 'exit 4'
::class sa
::method command
  use arg info
  say 'A command' info~command
  info~rc = 42
  return 1
::method unknown
  use arg name, args
  say 'A' name args[1]~name
  return 0
::class sb
::method command
  use arg info
  say 'B command' info~command
  return 1
::method unknown
  return 0
::class p
::method pm protected
  return 'pm-ran'
::method go
  address system 'exit 4'
  say 'method rc' rc
