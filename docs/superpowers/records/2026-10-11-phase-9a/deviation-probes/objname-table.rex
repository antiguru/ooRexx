longs = 'a longer string than seven'; longs~objectName = 'long'
say 'a longer string than seven'~objectName
say ('a longer string' || ' than seven')~objectName
say ('a longer string than seven' || '')~objectName
call r
x = 100; x~objectName = 'hundred'
say (99+1)~objectName
y = 101; y~objectName = 'hundredone'
say (100+1)~objectName
exit
r: procedure
  say 'a longer string than seven'~objectName
  return
