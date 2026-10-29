/* stem as an operator receiver, under SIGNAL ON NOSTRING */
v. = .nstr~new
s. = 7
do stem over 'v.', 's.'
  call try 'say' stem '""'
  call try 'say' stem'""'
  call try 'say' stem '|| "x"'
  call try 'say' stem '+ 1'
  call try 'say' stem '= "x"'
  call try 'say \'stem
  call try 'say' stem '& 1'
end
o = .nstr~new
call try 'say o ""'
call try 'say o || "x"'
exit
try:
  signal on nostring name h2
  signal on syntax name h3
  interpret 'r =' arg(1)~substr(5)
  if r~isA(.string) then say arg(1)~left(20) '=>' r
  else say arg(1)~left(20) '=> obj' r~class~id
  return
h2:
  say arg(1)~left(20) 'NOSTRING |'condition('d')'|' condition('o')~additional~class~id
  return
h3:
  say arg(1)~left(20) 'SYNTAX' condition('o')~code
  return
::class nstr
::method string; return 'NSTR'
