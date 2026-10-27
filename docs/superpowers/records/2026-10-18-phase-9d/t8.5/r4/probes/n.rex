q = .array~of('p','q'); d = .directory~new; n = .n~new
r = >q; rr = >r; say rr
r = >n; say rr
signal on nostring
say (>q) || 'x'
say length(>q)
r = >d
say 'd' rr
exit
nostring: say 'NOSTRING' condition('d') condition('o')~additional~class~id
::class n
::method string
  return 'nstr'
