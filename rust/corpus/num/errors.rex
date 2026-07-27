numeric digits 9
call try "1 / 0"
call try "1 % 0"
call try "1 // 0"
call try "1e999999999 * 10"
call try "'abc' + 1"
call try "2 ** 1e10"
say "done"
exit 0
try:
  parse arg expr
  signal on syntax name caught
  interpret "r =" expr
  say expr "->" r
  return
caught:
  say expr "-> SYNTAX" rc
  return
