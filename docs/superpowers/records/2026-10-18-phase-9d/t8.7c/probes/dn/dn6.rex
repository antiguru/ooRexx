a = .array~of(.dn~new, 'x')
say a~makestring(, ',')
say a~toString(, ',')
say a
say a~string
t = .table~new; t[1] = .dn~new
say t~allItems~makestring
signal on syntax
.dn~new~nosuch
exit
syntax:
say condition('o')~message
exit
::class dn
::method defaultname
  f = .context~stackFrames
  say '  frames' f~items f[2]~type f[2]~name
  return 'DN'
