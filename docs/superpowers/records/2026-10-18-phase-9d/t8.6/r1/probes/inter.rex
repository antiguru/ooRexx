say 's1'
call lineout 'STDERR', 'e1'
'echo c1; echo ce1 >&2'
say 's2'
address system 'echo c2; echo ce2 >&2' with output stem o.
say 's3' o.0
address system 'echo c3; echo ce3 >&2' with error stem e.
say 's4' e.0
.stderr~lineout('e2')
address system 'echo c4' with output stream 'o.txt'
say 's5'
trace r
x = 1
trace o
'echo c5'
do i = 1 to 3; say 'loop' i; 'echo cmd' i; end
