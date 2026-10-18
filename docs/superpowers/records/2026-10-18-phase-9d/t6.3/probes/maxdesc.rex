vals = " 4444444444 #04444444444#4.444444444E+3#44444444440E-1#1.0000000000#-0004444444444.0#4444444444e0#.00004444444444#4444444444E+20#12345678901234567890123#1.00000000000000000000000001E-7"
do while vals <> ''
  parse var vals v '#' vals
  call t v
end
exit
t: procedure
  parse arg v
  signal on lostdigits
  numeric digits 9
  x = max(1.5, v)
  say '['v'] none' x
  return
lostdigits:
  say '['v']' '['condition('D')']'
  return
