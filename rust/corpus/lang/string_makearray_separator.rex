-- String~makeArray with and without a separator.
a = 'a b  c'~makearray(' ')
say a~items a[1] '['a[3]']'
b = 'x--y--'~makearray('--')
say b~items b~toString(, ',')
c = 'abc'~makearray('')
say c~items
d = ''~makearray(',')
say d~items d~dimension
e = ('l1' || '0a'x || 'l2')~makearray
say e~items e[2]
f = 'p;q'~makearray(';')
say f~items f[2]
signal on syntax name h
g = 'abc'~makearray(.array~new)
say 'no'
exit
h:
say condition('o')~code condition('o')~message
