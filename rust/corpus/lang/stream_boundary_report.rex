/* A condition the Stream class's native boundary raises -- a write argument
 * with no string value, or an argument too many -- names the REXX package
 * with no line, inside a method of that package too. */
call try '.stdout~lineout(.nil)'
call try '.stdout~charout(.object~new)'
call try '.stdout~lineout("x", 1, 2)'
call try 'x = .stdout~description(1)'
call try '.stdout~say(.nil)'
call try '.stdout~lineout("x", "abc")'
.stdout~lineout(.nil)
exit
try:
  signal on syntax
  interpret arg(1)
  return
syntax:
  o = condition('O')
  where = o~program
  if where \== 'REXX' then where = filespec('name', where)
  say o~code where o~position
  return
