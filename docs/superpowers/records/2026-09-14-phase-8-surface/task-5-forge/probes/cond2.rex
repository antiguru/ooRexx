d = CondInfo(40001)
do l over d~traceback; say '['l']'; end
do f over d~stackframes; say f~type '|' f~name '|' f~line '|' f~executable~class '|' f~traceline '|' f~target; end
call sub
say .t~new~m
exit
sub: procedure
  d = CondInfo(40001)
  do l over d~traceback; say '['l']'; end
  say d~position
  return
::requires 'reach' LIBRARY
::class t
::method m
  d = CondInfo(88917, 'x')
  do l over d~traceback; say '['l']'; end
  return d~position d~code
