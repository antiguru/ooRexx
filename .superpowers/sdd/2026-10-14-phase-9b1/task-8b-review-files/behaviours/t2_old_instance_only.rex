.k~define('FOO', 'return "v1" copies("a", 10)')
old = .k~new
h = old~instanceMethod('FOO')~identityHash
.k~define('FOO', 'return "v2"')
call filler
do i = 1 to 300
  .k~define('BAR', 'return' i)
end
call filler
say 't2a' old~foo .k~new~foo
say 't2b' (old~instanceMethod('FOO')~identityHash = h) old~instanceMethod('FOO')~source[1]
say 't2c' old~hasMethod('BAR') .k~new~bar
say 't2d' old~instanceMethods(.k)~allIndexes~sort~makestring('l', ',')
exit
::class k
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
