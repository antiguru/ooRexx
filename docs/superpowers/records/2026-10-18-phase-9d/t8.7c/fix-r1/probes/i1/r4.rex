/* the r2 recursion without TRACE: DEFAULTNAME joining an array of itself (crate only) */
o = .rec~new
say .array~of(o)~makestring
say 'after'
exit
::class rec
::method defaultname
  return .array~of(self)~makestring
