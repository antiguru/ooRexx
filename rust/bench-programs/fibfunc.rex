/* Recursive fib(22) by internal function call, n times. */
n = 30
do i = 1 to n
    r = fib(22)
end
say 'fib(22) =' r
exit
fib: procedure
    arg k
    if k < 2 then return k
    return fib(k - 1) + fib(k - 2)
