/* A write to a package's .ROUTINES table moves what a routine name
   resolves to at a call site that has not yet resolved it; a site that
   has keeps the routine it found. */
do i = 1 to 2
  call foo
  say i result foo()
  if i = 1 then .routines~put(.routines~bar, 'FOO')
end
call foo
say 'new site' result foo() (.context~package~findRoutine('FOO') == .routines~bar)
.routines~put(.routine~new('n', 'return "new" arg(1)'), 'NEWR')
call newr 'x'
say result newr('y') .context~package~findRoutine('newr')~class~id
e = .routine~new('e', 'return .context~executable')
.routines['EX'] = e
say (ex() == e)
.routines~remove('FOO')
signal on syntax
call foo
say 'after remove' result
exit
syntax:
say condition('O')~code condition('O')~message
say (.context~package~findRoutine('FOO') == .nil)
::routine foo
  return 'foo'
::routine bar
  return 'bar'
