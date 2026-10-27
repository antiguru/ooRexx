/* Any use of a compound variable creates its stem variable: a read, a drop,
   a VALUE or SYMBOL inquiry, a parse target, through PROCEDURE EXPOSE and
   on an object variable. SYMBOL of the bare stem name alone creates
   nothing. */
say 'a' symbol('W.')
drop w.1
say 'b' symbol('W.') symbol('W.1') symbol('W.2')
x.1 = 5
drop x.1
say 'c' symbol('X.') symbol('X.1')
say y.3
say 'd' symbol('Y.')
v = value('Z.1')
say 'e' symbol('Z.')
drop q.
say 'f' symbol('Q.')
k = 1
drop m.k
say 'g' symbol('M.') m.
say 'h' symbol('A.1') symbol('A.')
call value 'C.1'; say 'i' symbol('C.')
interpret 'say f.1'; say 'j' symbol('F.')
parse value 'q' with g.1 .; say 'k' symbol('G.') g.1
call t1; say 'l' symbol('J.') symbol('K.')
say 'm' (l.1 == 'L.1') symbol('L.')
vars = .context~variables
say 'n' vars~hasIndex('W.') vars~hasIndex('Y.') vars~hasIndex('M.') vars~hasIndex('K.')
o = .k~new
o~r
o~s
call t2
signal on novalue
say z9.1
exit
novalue: say 'novalue' condition('d') symbol('Z9.')
exit
t1: procedure expose j.
  say j.1
  return
t2: procedure
  drop u.5
  say 'u' symbol('U.') .context~variables~items
  return
::class k
::method r
  expose a2.
  say 'r' a2.1 symbol('A2.')
::method s
  expose a2.
  say 's' symbol('A2.') a2.~items .context~variables~hasIndex('A2.')
