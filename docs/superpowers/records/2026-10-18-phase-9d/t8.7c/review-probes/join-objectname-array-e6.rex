/* OBJECTNAME and DEFAULTNAME answering a non-string object, in each rendering */
signal on syntax name s1
say .array~of(.on~new)~makestring
say .on~new~string
say .dn~new~string
say .dn~new~objectname
o = .dn~new
trace i
x = o
trace off
o~nosuch
s1: say 'trapped' condition('o')~message
exit
::class on
::method objectname
  return .array~of('p','q')
::class dn
::method defaultname
  return .array~of('p','q')
