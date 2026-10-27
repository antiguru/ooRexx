k = .kmake~new; s = 'kmake'
call t "'kmake'~equals(>k)"
call t "'kmake'~equals(>s)"
call t "'kmake'~equals(k)"
call t "'KMAKE'~caselessEquals(>k)"
call t "'KMAKE'~caselessEquals(k)"
call t "'kmake'~compareTo(>k)"
call t "'kmake'~compareTo(k)"
call t "'xkmakex'~pos(>k)"
call t "'xkmakex'~pos(k)"
call t "'xkmakex'~changestr(>k, '-')"
call t "'xkmakex'~changestr(k, '-')"
call t "'ab'~append(>k)"
call t "'ab'~append(k)"
call t "'kmake'~'=='(>k)"
call t "'kmake'~'='(>k)"
call t "'kmake'~'||'(>k)"
call t "'kmake'~startsWith(>k)"
call t "'kmake'~match(1, >k)"
call t "'kmake' == (>k)"
exit
t: parse arg code
  signal on syntax name e
  interpret 'r = ('code')'
  say code '->' r
  return
e: say code '-> SYNTAX' condition('o')~code; return
::class kmake
::method makestring; return 'kmake'
