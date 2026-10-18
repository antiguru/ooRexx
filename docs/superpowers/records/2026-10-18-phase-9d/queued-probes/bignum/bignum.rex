numeric digits 2000
p = 2**255 - 19
x = 2**250 + 12345
y = 2**251 + 67890
call time 'r'
do i = 1 to 2000
  z = (x*y)//p; x = (z+i)//p; y = (y-z)//p
end
say length(x) (time('e') < 1)
