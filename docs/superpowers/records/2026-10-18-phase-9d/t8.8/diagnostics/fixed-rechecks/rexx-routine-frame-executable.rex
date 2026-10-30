/* The StackFrame of a REXX package native routine answers one source-less
   Routine per routine name for ~executable, kept across calls. */
signal on syntax name a1
call sysSleep 'x'
a1: e = condition('O')['STACKFRAMES']~firstItem~executable
signal on syntax name a2
call sysSleep 'y'
a2: f = condition('O')['STACKFRAMES']~firstItem~executable
signal on syntax name a3
x = sysSleep('z')
a3: g = condition('O')['STACKFRAMES']~firstItem~executable
signal on syntax name a4
call sysFileDelete
a4: h = condition('O')['STACKFRAMES']~firstItem~executable
say condition('O')['STACKFRAMES']~firstItem~name
say e~class~id e~source~items (e~package == .context~package) (e == f) (e == g) (e == h) h~class~id
p = e~package
say p~name (p == h~package) (p == .object~package) e~annotations~items
call sub
say (.sube == e) (.context~package~findRoutine('SYSSLEEP') == .nil)
exit
sub: procedure
  signal on syntax name b1
  call sysSleep 'w'
b1: .local~sube = condition('O')['STACKFRAMES']~firstItem~executable
  return
