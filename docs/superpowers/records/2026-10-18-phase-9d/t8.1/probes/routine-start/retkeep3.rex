u5 = .WeakReference~new(.o~new~m)
u6 = .WeakReference~new(.o~new~m)
do i = 1 to 200000; z = .array~new(10); end
say (u5~value == .nil) (u6~value == .nil)
u7 = .WeakReference~new(fn())
do i = 1 to 200000; z = .array~new(10); end
say (u7~value == .nil)
u8 = .WeakReference~new(fn())
z = 1 + 2
do i = 1 to 200000; z = .array~new(10); end
say (u8~value == .nil)
exit
::routine fn
return .array~new
::class o
::method m
return .array~new
