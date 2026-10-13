/* SysDumpVariables: the caller's variables, a stem's own value and its tails,
   object values, dropped variables and tails, a method's variables, the file
   form appending, and the errors. The oracle lists the variables in hash
   order, so the file form's lines are sorted before they are shown. The
   stdout form is only used where its order is fixed: one line or none, or a
   stem's own line before its one tail, where a STRING method's SAY lands
   between the two. */
f = 'dumpvariables.out'
call SysFileDelete f
call one
call none
say 'none:' result
call several f
call show f
call several f
call several f
call show f
.k~new~m(f)
call show f
call talk
call t "call SysDumpVariables 1, 2"
call t "call SysDumpVariables .nil"
call t "call SysDumpVariables ''"
call t "call SysDumpVariables '/nonexistent-dir/x'"
call t "r = SysDumpVariables('" || f || "'); say 'returned' r"
call SysFileDelete f
call SysDumpVariables '/nonexistent-dir/x'
exit

one: procedure
  only = 'it''s'
  call SysDumpVariables
  return

none: procedure
  call SysDumpVariables
  return result

several: procedure
  parse arg f
  x = 'one'
  n = 12
  empty = ''
  s. = 'def'
  s.1 = 'a'
  s.k = .array~of('p', 'q')
  s.2 = 'gone'; drop s.2
  t.1 = 't1'
  t.two.three = 'deep'
  gone = 1; drop gone
  o = .object~new
  call SysDumpVariables f
  return

show: procedure
  parse arg f
  text = charin(f, 1, chars(f))
  call stream f, 'c', 'close'
  lines = .array~new
  do while text \== ''
    parse var text line '0a'x text
    lines~append(changestr('0d'x, line, '<CR>'))
  end
  say lines~items 'lines'
  do line over lines~sort
    say '  ' line
  end
  call SysFileDelete f
  return

talk: procedure
  s.1 = .talker~new
  call SysDumpVariables
  return

t: procedure
  parse arg code
  signal on syntax name bad
  interpret code
  say 'ok   ' code
  return
bad:
  c = condition('O')
  say 'error' code '|' c~code c~message
  return

::class k
::method m
  expose ivar
  parse arg f
  ivar = 'iv'
  local = 'lv'
  call SysDumpVariables f

::class talker
::method string
  say 'STRING ran'
  return 'K'
