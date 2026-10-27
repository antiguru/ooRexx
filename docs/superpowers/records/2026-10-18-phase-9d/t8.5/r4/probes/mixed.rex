/* a long acyclic chain alternating stem defaults and references */
parse arg n
do i = 1 to n
  interpret 's'i'. = >s'i + 1'.'
end
interpret 's'n + 1'. = 7'
say 'len' length(s1.) 'leaf' s1.
r = >s1.
say 'ref' length(r) r (r + 1)
say 'stem' (s1. + 1)
trace r
x = s1.
x = r
trace o
say 'ok'
