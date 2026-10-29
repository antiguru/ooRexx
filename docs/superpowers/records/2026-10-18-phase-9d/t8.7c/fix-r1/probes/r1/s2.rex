/* stems in required-string positions, under SIGNAL ON NOSTRING */
s. = 7
t. = .array~of('one','two')
v. = .nstr~new
k. = .kmake~new
do stem over 's.', 't.', 'v.', 'k.'
  call try 'say' stem '""'
  call try 'say length('stem')'
  call try 'say "<"||'stem'||">"'
  call try 'say 1 +' stem
  call try 'say "x" =' stem
  call try 'say' stem'~string'
  call try 'say' stem'~request("STRING")'
end
exit
try:
  signal on nostring name h2
  signal on syntax name h3
  interpret 'r =' arg(1)~substr(5)
  if r~isA(.string) then say arg(1)~left(28) '=>' r~changestr('0a'x, '|')
  else say arg(1)~left(28) '=> obj' r~class~id
  return
h2:
  say arg(1)~left(28) 'NOSTRING |'condition('d')'|' condition('o')~additional~class~id
  return
h3:
  say arg(1)~left(28) 'SYNTAX' condition('o')~code
  return
::class nstr
::method string; return 'NSTR'
::class kmake
::method makestring; return 'KM'
::method string; return 'KS'
