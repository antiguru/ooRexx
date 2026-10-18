signal on lostdigits name l1
a = 1 / 9999999999
say 'no lostdigits 1'
next1:
signal on lostdigits name l2
a = 1111111111 + 1
say 'no lostdigits 2' a
next2:
signal on lostdigits name l3
x = 1111111111
a = x + 1
say 'no lostdigits 3' a
exit
l1: say 'L1' condition('d'); signal next1
l2: say 'L2' condition('d'); signal next2
l3: say 'L3' condition('d'); exit
