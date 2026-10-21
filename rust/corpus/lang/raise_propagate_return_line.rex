/* A RAISE PROPAGATE RETURN two internal calls below the trapping level is
 * re-raised in the first of them: the report names that call's line, and its
 * echo lines are the condition object's own. */
call outer
exit
outer:
  signal on syntax
  say 1/0
  return
syntax:
  call s
  return 9
s:
  call t
  return
t:
  raise propagate return 5
