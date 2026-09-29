/* Recursive fib(22) by internal CALL, repeated n times: the cost of a CALL
   and its RETURN, with a PROCEDURE activation per call. */
n = 30
do i = 1 to n
    call fib 22
end
say 'fib(22) =' result
exit
fib: procedure
    arg k
    if k < 2 then return k
    call fib k - 1
    a = result
    call fib k - 2
    return a + result
