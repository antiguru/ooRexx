.k~define('FOO', 'return "v1" copies("a", 10)')
m1 = .k~method('FOO')
old = .k~new
h = old~instanceMethod('FOO')~identityHash
do i = 1 to 300
  .k~define('FOO', 'return "w' || i || '"')
end
call filler
say 't1a' old~foo
say 't1b' (old~instanceMethod('FOO') == m1) (old~instanceMethod('FOO')~identityHash = h)
say 't1c' m1~source[1]
say 't1d' .k~new~foo (.k~method('FOO') == m1)
say 't1e' .k~new~run(m1)
.k~define('BAR', m1)
call filler
say 't1f' .k~new~bar (.k~method('BAR') == m1) (.k~method('BAR')~source[1])
exit
::class k
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
