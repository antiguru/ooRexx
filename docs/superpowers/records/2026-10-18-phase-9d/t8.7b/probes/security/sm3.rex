.context~package~setSecurityManager(.sm~new)
'exit 5'
say 'main rc' rc
call r
call rt
interpret "'exit 7'; say 'interp rc' rc"
.p~new~go
exit
r:
'exit 6'
say 'internal rc' rc
return
::routine rt
'exit 3'
say 'routine rc' rc
::class sm
::method command
  use arg info
  say 'sm command' info~command
  info~rc = 42
  return 1
::method unknown
  use arg name, args
  say 'sm' name
  return 0
::class p
::method go
'exit 4'
say 'method rc' rc
