d = .directory~new
m = .k~method('BODY')~copy
m~setPrivate
d~setMethod('FOO', m)
drop m
d~setMethod("CALLFOO", "return self~foo")
say d~callfoo
say d~hasMethod('FOO')
e = .object~enhanced(.directory~of(('BAR', .k~method('BODY2')~copy~~setUnguarded)))
say e~bar
exit
::class k
::method body
  self~unsetMethod('FOO')
  call filler
  e = .context~executable
  return 'foo' e~class~id e~isPrivate e~isGuarded
::method body2
  self~unsetMethod('BAR')
  call filler
  e = .context~executable
  return 'bar' e~class~id e~isPrivate e~isGuarded
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
