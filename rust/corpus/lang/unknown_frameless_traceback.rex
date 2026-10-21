/* A message a Stem, a VariableReference, or a base-class Directory does not
 * understand reaches its primitive UNKNOWN with no traceback level of its own;
 * a Directory subclass and an explicit ~unknown send keep the level. */
d = .directory~new
d~setMethod('go', 'return 1/0')
call try d, 'GO'
call try .stem~new('S.'), 'NOSUCH'
call try .ss~new('T.'), 'NOSUCH'
v = 5
call try >v, 'NOSUCH'
s. = d
call try s., 'GO'
call try .dd~new~~setMethod('go', 'return 1/0'), 'GO'
call try d, 'UNKNOWN', 'GO', .array~new
d~setMethod('unknown', 'return 1/0')
call try d, 'OTHER'
say d~go
exit
try:
  signal on syntax
  use arg receiver, message
  if arg() > 2 then x = .message~new(receiver, message, 'i', arg(3), arg(4))~send
  else x = .message~new(receiver, message)~send
  return
syntax:
  say message condition('O')~code
  do line over condition('O')~traceback
    say '  ['line']'
  end
  return
::class ss subclass stem
::class dd subclass directory
