.context~package~setSecurityManager(.sm~new)
say 'set'
call rt
exit
::routine rt
  p = .package~new('pk', "say 'pk runs'; ::requires 'req2.cls'")
  say 'made'
  p~addPackage(.package~new('pk2', "::requires 'req1.cls'"))
  say 'done'
::requires 'req3.cls'
::class sm
::method requires
  use arg info
  say 'sm requires' info~name
  return 0
::method unknown
  use arg name, args
  say 'sm' name
  return 0
