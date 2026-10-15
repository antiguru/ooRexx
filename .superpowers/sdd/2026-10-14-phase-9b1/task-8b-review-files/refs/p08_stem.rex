r = mk()
call filler
say 'p08a' r~value~class~id r~value[1]~length left(r~value[1],3) r~value~items
s. = copies('dd', 9)
s.2 = copies('s2', 9)
q = >s.
drop s.
call filler
say 'p08b' q~value~class~id q~value[2]
t.k = copies('tk', 9)
signal on syntax
interpret 'tr = >t.k'
say 'p08c' tr~value
exit
syntax: say 'p08c syntax' rc condition('o')~code
exit
mk: procedure
  st.1 = copies('st', 15)
  return >st.
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
