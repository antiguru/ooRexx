.k~define('FOO',.k~method('BODY')~copy~~setPrivate~~setUnguarded)
.s~new~go
exit
::class k
::method go
  self~foo
::method body protected
  .k~define('FOO',.k~method('OTHER'))
  m=.context~executable
  say 'exe' m~isGuarded m~isPrivate m~isProtected
::method other
  return 17
::class s subclass k
