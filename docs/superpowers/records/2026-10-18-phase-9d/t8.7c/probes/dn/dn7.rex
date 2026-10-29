o = .dn~new
say o~string
say o~objectName
say o
p = .on~new
say p~string
trace i
x = p
x = o
trace off
o~objectName = 'named'
say o~string .array~of(o)~makestring
q = .sn~new
trace i
x = q
trace off
say .dc
say .array~of(.dc)~makestring
signal on syntax
.dc~zork
exit
syntax:
say condition('o')~message
exit
::class dn
::method defaultname
  f = .context~stackFrames
  say '  frames' f~items f[2]~type f[2]~name
  return 'DN'
::class on
::method objectname
  f = .context~stackFrames
  say '  on frames' f~items f[2]~type f[2]~name
  return 'ON'
::class sn
::method string
  return 'SN'
::class dc
::method defaultname class
  return 'DC'
