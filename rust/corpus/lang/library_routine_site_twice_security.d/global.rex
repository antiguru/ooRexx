do a over .array~of(4, 9)
  say 'global' RxCalcSqrt(a)
end
do a over .array~of('N', 'E')
  say 'internal' filespec(a, '/a/b.c')
end
