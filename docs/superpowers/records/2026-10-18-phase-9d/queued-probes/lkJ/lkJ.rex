call pJ
call gc 'force'; say 'gc1'
call gc 'force'; say 'gc2'
x = 'abc' || 'def'; call q
call gc 'force'; say 'gc3'
do n = 1 to 3; y = n * 2; end
call gc 'force'; say 'gc4'
call pK
call gc 'force'; say 'gc5'
exit
q: procedure; return
pJ: procedure
signal on syntax name trapJ
do i = .c~new('err') to .t~new('J') for 5; nop; end
return
trapJ: return
pK: procedure
do i = .c~new('ok') to .t~new('K') for 2; nop; end
return
::class c
::method init; expose k; use arg k
::method '+'; return self
::method '>'
expose k
if k = 'err' then return 1 + 'x'
return 0
::class t
::method init; expose n; use arg n
::method '+'; return self
::method uninit; expose n; say 'uninit' n
