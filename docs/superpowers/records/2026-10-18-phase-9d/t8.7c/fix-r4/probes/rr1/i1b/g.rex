o = .dn~new
call show >o
say 'arr' .array~of(>o)~makestring
r = >o
say 'plain' r
say 'cat' r || 'x'
say 'len' length(r)
say 'str' r~string
say 'abut' 'p'r
trace i
z = r
trace off
exit
show: use arg >x
  say 'ref' x
  say 'ref cat' x'|'
  return
::class dn
::method defaultname
  say '  (dn sent)'
  return 'DN'
