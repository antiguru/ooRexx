/* A scanner error is reported when the parse reaches its clause, so a
   parse error in an earlier clause of the same INTERPRET string wins. */
call try 'say (;' || '0a'x
call try 'trace -;' || '0a'x
call try 'x = ;' || '0a'x
call try 'x = 1 + ;' || '01'x
call try 'say 1;' || '0a'x
call try 'say 1;' || '0a'x || 'say ('
call try 'if 1;' || '0a'x
call try 'do 2;' || '0a'x
call try 'nop; "abc'
call try 'say (; "abc'
call try 'say (; /* open'
call try 'nop; /* open'
exit

try:
  signal on syntax name caught
  interpret arg(1)
  say 'ran'
  return
caught:
  say condition('o')~code condition('o')~message
  return
