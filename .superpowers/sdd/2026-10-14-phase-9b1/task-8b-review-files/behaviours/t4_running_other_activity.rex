.local['READY'] = 0; .local['GO'] = 0
.k~define('FOO', .Method~new('foo', .k~method('BODY')~source)~~setPrivate~~setUnguarded)
o = .s~new
m = o~start('go')
do while .local['READY'] \== 1; call SysSleep 0.01; end
do i = 1 to 50
  .k~define('FOO', 'return "new' i '"')
end
call filler
.local['GO'] = 1
say 't4' m~result .s~new~foo
exit
::class k
::method body
  e0 = .context~executable~identityHash
  .local['READY'] = 1
  do while .local['GO'] \== 1; call SysSleep 0.01; end
  call filler
  e = .context~executable
  return 'exe' e~isPrivate e~isGuarded (e~identityHash = e0) e~source[1]
::method go unguarded
  return self~foo
::class s subclass k
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
