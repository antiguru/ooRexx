d = CondInfo(40001)
f = d~stackframes~makeArray[1]
say f~type '|' f~name '|' f~line '|' f~target '|' f~arguments~items f~arguments~class '|' f~invocation '|' f~context '|' f~traceline
say f~string
g = d~stackframes~makeArray[2]
say g~type '|' g~name '|' g~line '|' g~target '|' g~invocation~class
say .t~new~m(1, 2)
exit
::requires 'reach' LIBRARY
::class t
::method m
  use arg a, b
  d = CondInfo(40001)
  f = d~stackframes~makeArray[1]
  say f~type '|' f~name '|' f~line '|' f~target~class '|' f~arguments~items '|' f~invocation '|' f~context '|' f~traceline
  return d~traceback~items
