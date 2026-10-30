m=.k~method('BODY')~copy~~setPrivate~~setUnguarded
say 'template' m~isGuarded m~isPrivate m~isProtected
.k~define('FOO',m)

.s~new~go
exit
::class k
::method go
  self~foo
::method body protected
  m=.context~executable
  say 'before' m~isGuarded m~isPrivate m~isProtected

  .k~define('FOO',.k~method('OTHER'))
  m=.context~executable
  say 'after' m~isGuarded m~isPrivate m~isProtected
  
::method other
  return 17
::class s subclass k
