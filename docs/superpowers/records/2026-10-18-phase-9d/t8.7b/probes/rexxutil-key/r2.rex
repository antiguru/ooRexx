signal on syntax
call r
exit
syntax: say condition('o')~code condition('o')~message; exit
r: interpret "call x"; return
::requires 'REXXUTIL' library
