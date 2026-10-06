c = .w~new
m0 = .message~new(c, 'val', 'I', 0)
m1 = .message~new(c, 'val', 'I', 1)
m2 = .message~new(c, 'val', 'I', 2)
c~start('s1', m0, m1, m2)
interpret "say 'main got' m0~result"
m2~send
say 'main sent m2'
say 'done'
::class w
::method val unguarded; use arg x; return x*10
::method s1 unguarded
  use arg m0, m1, m2
  .w~new~start('s3', m0, m1, m2)
  interpret 'say "S1 got" m1~result'
::method s3 unguarded
  use arg m0, m1, m2
  m0~send
  say 's3 sent m0'
  .w~new~start('s2', m1, m2)
::method s2 unguarded
  use arg m1, m2
  interpret 'say "S2 got" m2~result'
  m1~send
