o = .dn~new
r = >o
trace i
z = r
trace off
say '--'
trace r
z = r
trace off
say '--'
trace i
z = o
exit
::class dn
::method defaultname
  say '  (dn sent)'
  return 'DN'
