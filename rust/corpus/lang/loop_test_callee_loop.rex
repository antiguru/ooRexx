/* A WHILE or UNTIL whose test calls a routine that runs a loop of its own,
   at a pass boundary of a loop with others open around it. */
do j = 1 to 2
  n = 0
  do while n < 2
    n = n + 1
    k = 0
    do while more(k)
      k = k + 1
    end
    m = 0
    do until stop(m)
      m = m + 1
    end
    say j n k m
  end
end
do r = 1 to 2
  call trapped
  say 'after' r
end
say 'end'
exit

more: procedure
  use arg k
  do i = 1 to 1
  end
  return k < 1

stop: procedure
  use arg m
  do i = 1 to 1
  end
  return m >= 2

trapped: procedure
  signal on syntax
  do q = 1 to 2
    t = 0
    do while bad(t)
      t = t + 1
    end
  end
  return
syntax:
  say 'trapped' condition('o')~code q t
  return

bad: procedure
  use arg t
  if t = 0 then return 1
  do i = 1 to 1
  end
  return 1 + 'a'
