r = .Routine~new("r", "say 'r runs'; call ext3; .context~package~loadPackage('a8.cls'); say 'loaded'")
r~setSecurityManager(.sm~new)
r~call
::class sm
::method requires
  use arg info
  say 'sm requires' info~name~substr(info~name~lastpos('/') + 1)
  info~securityManager = self
  return 1
::method unknown
  use arg name, args
  say 'sm' name
  return 0
