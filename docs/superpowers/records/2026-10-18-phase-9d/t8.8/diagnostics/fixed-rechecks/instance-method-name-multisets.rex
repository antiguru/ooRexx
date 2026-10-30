call emit 'NIL_ALL',.nil~instanceMethods
call emit 'NIL_NIL',.nil~instanceMethods(.nil)
call emit 'SUB_ALL',.sub~new~instanceMethods
call emit 'SUB_NIL',.sub~new~instanceMethods(.nil)
exit
emit: procedure
  use arg tag,s
  do while s~available
    say tag s~index
    s~next
  end
  return
::class base
::method init
::method foo
::method bar
::class sub subclass base
::method init
::method bar
