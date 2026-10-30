numeric digits 5
k = .kn~new
signal on lostdigits name h
say 1 + k
say 'no lostdigits'
h: say 'LOSTDIGITS' condition('d')
signal on lostdigits name h3
say 1 = k
h3: say 'LOSTDIGITS3' condition('d')
signal on lostdigits name h4
say 1 * k
h4: say 'LOSTDIGITS4' condition('d')
signal on lostdigits name h5
say 1 + ' 1234567 '
h5: say 'LOSTDIGITS5' condition('d')
r = >k
signal on lostdigits name h6
say 1 + r
h6: say 'LOSTDIGITS6' condition('d')
numeric fuzz 1
signal on lostdigits name h7
say 1 = .kf~new
h7: say 'LOSTDIGITS7' condition('d')
signal on lostdigits name h8
say 1 < .kf~new
h8: say 'LOSTDIGITS8' condition('d')
exit
::class kn
::method makestring; say '  (mk)'; return '1234567890'
::method defaultname; say '  (kn dn sent)'; return 'DNK'
::class kf
::method makestring; say '  (mkf)'; return ' 12345'
::method defaultname; say '  (kf dn sent)'; return 'DNF'
