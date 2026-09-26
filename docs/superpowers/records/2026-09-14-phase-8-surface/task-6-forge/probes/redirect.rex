/* A redirecting handler reading and writing each stream through every member. */
call AddCmd 'xr', 'r'
address xr 'ECHO' with input using (('l1', '', 'l3')) output stem o. error stem e.
say rc '|' o.0 o.1 o.2 o.3 '|' e.0 e.1 e.2 e.3
address xr 'ECHO' with input using (('l1', 'l2')) output stem s. error stem s.
say rc '|' s.0 s.1 s.2 s.3 s.4
address xr 'ECHO'
say rc
address xr 'ECHO' with output stem o.
say rc '|' o.0
i.0 = 0
address xr 'ECHO' with input stem i.
say rc
address xr 'BUFFIRST' with input using (('a', 'b')) output stem o.
say rc '|' o.0 o.1 o.2
address xr 'BUFFIRST' with output stem o.
say rc '|' o.0
address xr 'BUFFIRST' with input stem i. output stem o.
say rc '|' o.0
address xr 'BUFFERS' with output stem o. error stem e.
say rc '|' o.0 c2x(o.1) c2x(o.2) c2x(o.3) c2x(o.4) c2x(o.5) '|' e.0 c2x(e.1) c2x(e.2) c2x(e.3) c2x(e.4)
address xr 'BUFFERS' with output stem s. error stem s.
say rc '|' s.0
address xr 'BUFFERS'
say rc
address xr 'FLAGS' with input using 'x' error stem e.
say rc
a = .array~new
address xr 'ECHO' with input using 'q' output using (a) error normal
say rc a~items a[1]
::requires 'cmd' LIBRARY
