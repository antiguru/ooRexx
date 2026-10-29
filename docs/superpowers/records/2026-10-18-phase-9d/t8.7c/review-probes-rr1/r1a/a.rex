s. = 7
t. = .array~of('one','two')
v. = .nstr~new
k. = .kmake~new
do stem over 's.', 't.', 'v.', 'k.'
  call try 'say' stem
  call try 'say substr('stem',1,2)'
  call try 'say reverse('stem')'
  call try 'say pos("n",'stem')'
  call try 'say "abc"~pos('stem')'
  call try 'say "abc"~left(1)'stem
  call try 'say ""' stem
  call try 'say' stem '== "7"'
  call try 'say "7" ==' stem
  call try 'say' stem '= 7'
  call try 'say' stem '+ 1'
  call try 'say -'stem
  call try 'say \'stem
  call try 'parse value' stem 'with pv; say pv'
  call try 'pvv =' stem'; parse var pvv pv2; say pv2'
  call try 'a.7 = "seven"; a.one = "uno"; say a.['stem']'
  call try 'interpret "say" ' stem
  call try 'push' stem'; parse pull pp; say pp'
  call try 'say d2x('stem')'
  call try 'say ('stem')~class~id'
  call try 'say abbrev("seven",' stem')'
  call try 'say translate('stem')'
  call try 'say' stem '\== "x"'
  call try 'say' stem '<< "zzz"'
end
exit
try:
  signal on nostring name h2
  signal on syntax name h3
  interpret arg(1)
  return
h2: say '  ' arg(1) '-> NOSTRING' condition('d') condition('o')~additional~class~id; return
h3: say '  ' arg(1) '-> SYNTAX' condition('o')~code; return
::class nstr
::method string; return 'NSTR'
::class kmake
::method makestring; return 'KM'
::method string; return 'KS'
