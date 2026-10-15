/* INTERPRET n times over a fragment whose literal is longer than a handle
   carries inline, so each pass compiles a chunk that interns a heap string. */
n = 20000
t = 0
do i = 1 to n
    interpret 'x = "a literal longer than seven"; t = t + length(x)'
end
say t
