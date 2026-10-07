/* A control variable holding an object is sent + with BY at the increment:
   97.1 where it has none, an answer that is an object is sent the TO
   comparison, and a + that answers a number goes on with it. */
call t1; call t2; call t3; call t4; call t5
exit
t1: signal on syntax name x1; do i = 1 to 3; say 't1' i; i = .t~new; end; return
x1: say 't1' rc condition('O')~message; return
t2: signal on syntax name x2; do i = 1 to 3; say 't2' i; i = .array~of(1); end; return
x2: say 't2' rc condition('O')~message; return
t3: signal on syntax name x3; do i = 1 to 3; say 't3' i; i = .local; end; return
x3: say 't3' rc condition('O')~message; return
t4: do i = 1 to 3; say 't4' i; if i = 2 then i = .p~new(i); end; say 't4 done' i; return
t5: signal on syntax name x5; trace r; do i = 5 to 1 by -1; i = .array; end; return
x5: trace o; say 't5' rc condition('O')~code; return
::class t
::class p
::method init; expose v; use arg v
::method '+'; expose v; use arg o; if arg() = 0 then return v; return v + o
