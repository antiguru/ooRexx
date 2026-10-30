call emit 'SUB_CLASS',.sub~instanceMethods
call emit 'SUB_CLASS_NIL',.sub~instanceMethods(.nil)
call emit 'ARRAY_CLASS',.array~instanceMethods
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
