/* an operator on a variable reference raises inside the referenced value's method */
s = 'abc'; n = 5; m. = 'q'; drop u
call try 't = >s; say t + 0'
call try 't = >s; say -t'
call try 't = >s; say +t'
call try 't = >n; say t / 0'
call try 't = >n; say t ** "x"'
call try 't = >n; say t + .array~new'
call try 't = >u; say t + 1'
call try 't = >m.; say t * 2'
call try 'tt = >s; t = >tt; say t - 1'
call try 't = >n; say t // 0'
call try 't = >s; say \t'
call try 't = >s; say t & 1'
exit
try:
  signal on syntax
  interpret arg(1)
  return
syntax:
  say arg(1)~left(30) condition('o')~code
  do l over condition('o')~traceback; say '   ' l; end
  return
