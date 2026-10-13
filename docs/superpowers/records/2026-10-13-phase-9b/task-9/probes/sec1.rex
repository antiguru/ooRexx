.context~package~setSecurityManager(.mgr~new)
call SysSleep 0
say 'sleep' result
say 'ver' SysVersion()~word(1)
call fs 'n', '/a/b.c'
say 'fs' result
say 'done'
::class mgr
::method unknown
  use arg name, args
  say 'event' name args[1]~name
  return 0
::routine fs external 'LIBRARY REXX Filespec'
