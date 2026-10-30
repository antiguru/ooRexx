parse arg f
signal on syntax name fallback
a = .Package~new(f)~source
call report a
exit 0

fallback:
  lines = .array~new
  do while lines(f) \== 0
    lines~append(linein(f))
  end
  call stream f, "c", "close"
  call report lines
  exit 0

report: procedure
  use arg a
  say "%SRCG%COUNT" a~items
  do i = 1 to a~items
    say "%SRCG%L" || a[i]
  end
  return
