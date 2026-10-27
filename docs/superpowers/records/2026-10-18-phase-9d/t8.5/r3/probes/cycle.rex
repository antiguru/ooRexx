a = 1; b = 2
x = >a; y = >b
a = y; b = x
say 'made'
signal on nostring
say 'say' x
exit
nostring: say 'NS' condition('D'); exit
