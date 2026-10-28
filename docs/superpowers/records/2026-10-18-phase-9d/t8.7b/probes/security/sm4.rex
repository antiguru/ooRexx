.context~package~setSecurityManager(.sm~new)
call ext1
call rt
exit
::routine rt
call ext2
::class sm
::method requires
  use arg info
  say 'sm requires' info~name
  return 0
::method unknown
  use arg name, args
  say 'sm' name
  return 0
