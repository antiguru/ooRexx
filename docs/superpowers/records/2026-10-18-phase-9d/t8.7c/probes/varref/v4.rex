s = 'abc'; n = 5
call try 't = >s; do i = t to 5; end'
call try 't = >n; numeric digits 1; n = "9.9E999999999"; say t > 1'
call try 't = >s; say t = 1'
exit
try:
  signal on syntax
  interpret arg(1)
  say arg(1)~left(30) 'ok'
  return
syntax:
  say arg(1)~left(30) condition('o')~code
  do l over condition('o')~traceback; say '   ' l; end
  return
