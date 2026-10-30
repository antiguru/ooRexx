numeric digits 5
k = .kn~new; r = >k; s = .ks~new
signal on lostdigits name h; signal on syntax name sy
say k + 1
say 'no condition'; exit
h: say 'LOSTDIGITS' condition('d'); exit
sy: say 'SYNTAX' condition('o')~code condition('o')~message; exit
::class kn
::method makestring; return '1234567890'
::method defaultname; say '  (kn dn sent)'; return 'DNK'
::class ks
::method makestring; return ' 1234567890 '
