.k~define('SEP', .Method~loadExternalMethod('m', 'LIBRARY REXX file_separator'))
call filler
say 'define' .k~new~sep
m = .Method~loadExternalMethod('m', 'LIBRARY REXX file_separator')
m~setPrivate
.k~define('PSEP', m)
drop m
call filler
o = .k~new
say 'private' o~callp o~hasMethod('PSEP') .k~method('PSEP')~isPrivate
o~put('OSEP', .Method~loadExternalMethod('m', 'LIBRARY REXX file_separator')~copy)
call filler
say 'own' o~osep
do i = 1 to 3
  q = .Method~loadExternalMethod('q', 'LIBRARY REXX file_separator')
  q~setProtected
end
drop q
call filler
say 'again' .k~new~sep o~osep o~callp
exit
filler: procedure
  do i = 1 to 300
    z = copies('x', 100000)
  end
  return
::class k
::method callp
  return self~psep
::method put
  use arg name, m
  self~setMethod(name, m)
