/* What a manager has to answer, and what happens when it does not.

   `callSecurityManager` requires a result and requires it to be a logical
   value, so a manager that returns nothing raises 91.999 and one that returns
   anything else raises 34.903. Both are raised at the checkpoint, inside the
   agent, and reach this program as ordinary SYNTAX conditions. */

call probe .Silent~new
call probe .Wrong~new
exit 0

probe: procedure
  use arg manager
  signal on syntax name reported
  agent = .Routine~newFile('agent.rex')
  agent~setSecurityManager(manager)
  agent~call
  say 'no condition'
  return
reported:
  say 'code   ' condition('O')['CODE']
  say 'message' condition('O')['MESSAGE']
  return

::class Silent
::method unknown
  use arg name, args
  return

::class Wrong
::method unknown
  use arg name, args
  return 'yes'
