s. = 'default'
s.KM = 'found'
k = .key~new
say s.[k] s.[>k]
t. = k
say s.[t.]
s.[k,'x'] = 'written'
say s.KM.x s.~hasIndex(k,'x') s.~remove(k,'x') s.~hasIndex(k,'x')
s.[,'x'] = 'omitted'
say s.['','x'] s.[,'x']
s.['',''] = 'empty'
say s.['',''] s.~hasIndex('','')
exit
::class key
::method makeString
  say 'make'
  return 'KM'
