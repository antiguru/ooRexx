.context~package~setSecurityManager(.sa~new)
call rt
exit
::routine rt
'exit 1'
.context~package~setSecurityManager(.sb~new)
'exit 2'
call r2
call rt2
return
r2:
'exit 3'
return
::routine rt2
'exit 4'
::class sa
::method command
  use arg info
  say 'A' info~command
  return 1
::method unknown
  return 0
::class sb
::method command
  use arg info
  say 'B' info~command
  return 1
::method unknown
  return 0
