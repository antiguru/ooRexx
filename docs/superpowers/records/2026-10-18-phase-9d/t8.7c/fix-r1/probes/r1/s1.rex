s. = .nstr~new
call try 'say 1 + s.'
call try 'say "x" = s.'
call try 'say s.'
call try 'say length(s.)'
call try 'say 1 + s.~string'
call try 'numeric digits s.'
exit
try:
  signal on nostring name h2
  interpret arg(1)
  return
h2:
  say 'h2' '|'condition('d')'|' '|'condition('o')~description'|' condition('a')~class~id condition('o')~additional~class~id
  return
::class nstr
::method string; return 'NSTR'
