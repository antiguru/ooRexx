/* NOSTRING additional object, through a reference and through a stem */
signal on nostring
d = .directory~new
r = >d
say 'a' r
s. = d
say 'b' s.
q = >s.
say 'c' q
u = >q
say 'd' u
say 'end'
exit
nostring: say 'NS' condition('D') condition('O')~class~id condition('A')~class~id; signal on nostring; return
