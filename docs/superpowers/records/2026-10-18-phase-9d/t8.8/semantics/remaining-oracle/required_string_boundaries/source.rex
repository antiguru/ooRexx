k = .key~new
s.[k, k] = 'stored'
say s.[k, k] s.~hasIndex(k, k)
say s.~remove(k, k) s.~hasIndex(k, k)
parse value 'left pattern-long-enough-for-the-heap right' with before (k) after
say '['before']['after']'
parse caseless value 'LEFT PATTERN-LONG-ENOUGH-FOR-THE-HEAP RIGHT' with before (k) after
say '['before']['after']'
n = .numberKey~new
say 5~max(n) 9~min(n)
exit
::class key
::method makeString
  return 'pattern-long-enough-for-the-heap'
::class numberKey
::method makeString
  return ' 7 '
