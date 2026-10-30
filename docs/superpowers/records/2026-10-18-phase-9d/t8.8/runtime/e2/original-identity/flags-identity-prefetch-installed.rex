m=.k~method('BODY')~copy~~setPrivate~~setUnguarded
say 'template' m~isGuarded m~isPrivate m~isProtected
.k~define('FOO',m)
.local~installed=.k~method('FOO')
say 'installed' .installed~isGuarded .installed~isPrivate .installed~isProtected

.s~new~go
exit
::class k
::method go
  self~foo
::method body protected
  
  .k~define('FOO',.k~method('OTHER'))
  m=.context~executable
  say 'after' m~isGuarded m~isPrivate m~isProtected
  say 'is-installed' (m == .installed)

::method other
  return 17
::class s subclass k
