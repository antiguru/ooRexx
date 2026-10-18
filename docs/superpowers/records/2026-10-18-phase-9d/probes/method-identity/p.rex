say (.object~method('copy') == .object~method('copy'))
s = .object~methods(.nil); a = s~item
s2 = .t~methods; do while s2~available; if s2~index == s~index then do; say (s2~item == a) s~index; leave; end; s2~next; end
say (.t~method('foo') == .t~new~instanceMethod('foo'))
::class t
::method foo
