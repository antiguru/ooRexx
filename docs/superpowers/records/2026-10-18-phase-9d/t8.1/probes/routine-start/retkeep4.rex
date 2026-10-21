u9 = .WeakReference~new(.array~new)
call churn
say 'native send' (u9~value == .nil)
u7 = .WeakReference~new(fn())
call churn
say 'routine' (u7~value == .nil)
u8 = .WeakReference~new(.o~new~m)
call churn
say 'method' (u8~value == .nil)
exit
churn: procedure
do i = 1 to 200000; z = .array~new(10); end
return
::routine fn
return .array~new
::class o
::method m
return .array~new
