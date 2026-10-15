.k~define('FOO', 'return .context~executable~identityHash .context~executable~source[1]')
old = .k~new
h1 = old~foo
do i = 1 to 300
  .k~define('FOO', 'return "w' || i || '"')
end
call filler
h2 = old~foo
say 't5' (h1 = h2) word(h2, 2) .k~new~foo
exit
::class k
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
