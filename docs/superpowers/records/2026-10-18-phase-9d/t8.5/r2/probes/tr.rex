m = .knum~new; k = .kmake~new; q = .array~of('p','q'); n = .nstr~new
trace value (>m)
say 'trace' trace()
trace o
r = (>k); trace i
y = r || 'z'
z = 'a' r
w = 'b' || (>n)
trace o
say d2c(>m + 60) left(>k, 2) copies(>k, 2) reverse(>k) word(>q, 1) words(>q)
say abs(>m) format(>m, 3) (>m)~length datatype(>m) datatype(>k)
::class kmake
::method makestring; return 'kmake'
::class knum
::method makestring; return 'N'
::class nstr
::method string; return 'nstr'
