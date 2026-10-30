call emit 'ARRAY_ALL',.Array~methods
call emit 'ARRAY_NIL',.Array~methods(.nil)
exit
emit: procedure
  use arg tag,s
  do while s~available
    say tag s~index
    s~next
  end
  return
