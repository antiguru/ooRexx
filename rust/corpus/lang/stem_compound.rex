s. = "default"
s.1 = "one"
s.2 = "two"
say s.1 s.2 s.3
i = 2
say s.i
a.b.c = "nested"
say a.b.c
t. = 0
do k = 1 to 3
  t.k = k * k
end
do k = 1 to 3
  say "t."k"="t.k
end
drop s.1
say s.1
say s.
