/* Directory and .local entry reads: the store-backed read path. */
d = .directory~new
d~put(1, 'A')
.local~zz = 2
n = 200000
do i = 1 to n
  x = d~a
  y = d['A']
  z = .zz
end
say x y z
