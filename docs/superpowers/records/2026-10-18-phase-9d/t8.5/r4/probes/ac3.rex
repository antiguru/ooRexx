/* acyclic, no string value through a stem default */
signal on nostring
d = .directory~new
s. = d
r = >s.
say 'a' r
say 'end'
exit
nostring: say 'NS' condition('D') condition('O')~class~id condition('A')~class~id; exit
