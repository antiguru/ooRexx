o = .k~new
r = o~mk
drop o
call filler
say 'q11a' length(r~value) left(r~value,3) .k~new~isref(r)
r2 = mk2()
call filler
say 'q11b' length(r2~value) left(r2~value,3)
exit
mk2: procedure
  y = 'short'
  .k~new~setref(>y, copies('sv', 14))
  return >y
::class k
::method mk
  expose v
  v = copies('nv', 18)
  return self~getref('V')
::method getref EXTERNAL "LIBRARY orxmethod TestGetObjectVariableReference"
::method isref EXTERNAL "LIBRARY orxmethod TestIsVariableReference"
::method setref EXTERNAL "LIBRARY orxmethod TestSetVariableReferenceValue"
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
