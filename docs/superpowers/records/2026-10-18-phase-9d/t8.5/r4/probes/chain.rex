/* a long acyclic chain of references */
parse arg n
do i = 1 to n
  interpret 'r'i' = >r'i + 1
end
interpret 'r'n + 1' = "leaf"'
say 'len' length(r1) 'leaf' r1
say 'arith' (r1 || 'x')
trace r
x = r1
trace o
say 'ok'
