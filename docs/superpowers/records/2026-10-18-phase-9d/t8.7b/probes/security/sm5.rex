r = .routine~newFile('agent.rex')
r~setSecurityManager(.sm~new)
say "calling"; r~call
say 'back'
::class sm
::method requires
  use arg info
  say 'sm requires' info~name
  return 0
::method call
  use arg info
  say 'sm call' info~name
  return 0
::method unknown
  use arg name, args
  say 'sm' name
  return 0
