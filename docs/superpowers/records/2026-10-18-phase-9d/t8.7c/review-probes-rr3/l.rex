numeric digits 5
k = .kn~new
signal on lostdigits name h
say 1 + k
say 'no lostdigits'
exit
h: say 'LOSTDIGITS' condition('d')
o = .dn~new
s. = >o
say 'stem of ref' s.
u = .ustem~new
u[] = >o
say 'ustem of ref' u ''
say 'arr of stem of ref' .array~of(s.)~makestring
exit
::class kn
::method makestring; return '1234567890'
::method defaultname; say '  (kn dn sent)'; return 'DNK'
::class dn
::method defaultname
  say '  (dn sent)'
  return 'DN'
::class ustem subclass stem
