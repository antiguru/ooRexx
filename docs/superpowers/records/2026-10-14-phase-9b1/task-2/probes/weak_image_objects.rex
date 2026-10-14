t = .true; a = .Array; e = .environment; one = 1
w1 = .WeakReference~new(t); w2 = .WeakReference~new(a); w3 = .WeakReference~new(e); w4 = .WeakReference~new(one)
say 'before' (w1~value == t) (w2~value == a) (w3~value == e) (w4~value == one)
do i = 1 to 3000; z = copies('y', 100000); q = .array~new(100); end
say 'after' (w1~value == t) (w2~value == a) (w3~value == e) (w4~value == one)
