signal on syntax
n = 0
call t "SysUtilVersion(1)"
call t "SysLoadFuncs(1)"
call t "SysDropFuncs('x')"
call t "SysWait(1)"
call t "SysFork(1)"
call t "SysCls(1)"
call t "SysQueryProcess('bogus')"
call t "SysQueryProcess('')"
call t "SysQueryProcess('pid', 2)"
call t "SysQueryProcess('PID'||'00'x||'x')"
call t "SysGetKey('bogus')"
call t "SysGetKey('')"
call t "SysGetKey(,)"
call t "SysCreatePipe('')"
call t "SysCreatePipe('x')"
call t "SysCreatePipe('b', 1)"
call t "SysSetPriority()"
call t "SysSetPriority(1)"
call t "SysSetPriority(,'a')"
call t "SysSetPriority(,1.5)"
call t "SysSetPriority('a', 0)"
call t "SysSetPriority(,99999999999)"
call t "SysSetPriority(,-30)"
call t "SysSetPriority(,1,2)"
call t "SysQueryProcess('pprio')"
exit
t:
  parse arg expr
  signal on syntax name trap
  interpret 'r = ' expr
  say expr '->' r
  return
trap:
  say expr '-> error' condition('C') rc '|' errortext(rc) '|' condition('O')~code '|' condition('O')~message
  return
syntax: say 'outer'; exit
