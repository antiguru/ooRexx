/* The exit context's variable members and caller context, from a direct handler. */
call AddCmd 'xd', 'd'
x = 'ecks'; z = 'zed'
address xd 'VARS one'
say rc
say 'y='y 'z='z symbol('Z')
address xd 'CALLER'
say rc~class~id rc~line
address xd 'REF'
say rc x
call sub
exit
sub: procedure
  x = 'inner'
  address xd 'VARS two'
  say rc 'y='y
  return
::requires 'cmd' LIBRARY
