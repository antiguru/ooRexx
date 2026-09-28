/* A condition an extension raises with RaiseCondition and a CALL ON trap
   takes: the oracle builds its object at the raise, with the native frame
   leading STACKFRAMES and TRACEBACK, and a frame with a package and no line
   gives it no POSITION. From the main program, an internal routine and a
   method. No description is passed (oracle-crashes.txt entry 17). */
t = .T~new
call on user bar name handler
say 'main' t~rc('USER BAR', , .array~of('x'), 'r1')
call inner t
say 'meth' t~wrap
say 'done'
exit
inner: procedure
  use arg t
  call on user bar name handler
  say 'inner' t~rc('USER BAR', , , 'r2')
  return
handler:
  o = condition('o')
  do k over .array~of('CONDITION', 'DESCRIPTION', 'INSTRUCTION', 'POSITION', 'PROGRAM', 'PROPAGATED', 'RESULT', 'ADDITIONAL')
    if o~hasIndex(k) then say ' ' k '=' filespec('n', o[k]~string)
  end
  say '  frames' o['STACKFRAMES']~items
  do l over o['TRACEBACK']; say '  tb' l; end
  do f over o['STACKFRAMES']; say '  sf' f~type filespec('n', f~name) f~line; end
  return 'fromhandler'
::class T
::method rc external "LIBRARY orxmethod TestRaiseCondition"
::method wrap
  call on user bar name h2
  r = self~rc('USER BAR', , , 'r3')
  return r
h2:
  o = condition('o')
  say '  wrap position' o~hasIndex('POSITION') o['POSITION']~string
  do l over o['TRACEBACK']; say '  tb' l; end
  return
