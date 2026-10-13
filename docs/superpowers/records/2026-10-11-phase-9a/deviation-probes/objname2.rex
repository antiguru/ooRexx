q = 7
v = >q
v~objectName = 'my ref'
say v~objectName '|' v~string '|' v
w = >q
say w~objectName
say q~objectName
o = .object~new
o~objectName = 'my obj'
say o~objectName '|' o~string
longs = 'a longer string than seven'
longs~objectName = 'long'
say longs~objectName '|' longs~string
w = longs
say w~objectName
say 'a longer string than seven'~objectName
big = 123456789012
big~objectName = 'big'
say big~objectName big~string
say 123456789012~objectName
say 5~objectName
x = 100; x~objectName = 'hundred'
say (99+1)~objectName
y = 101; y~objectName = 'h1'
say (100+1)~objectName
z = -10; z~objectName = 'm10'
say (0-10)~objectName (-10)~objectName
say (1.0+4)~objectName
say (2+3)~objectName (10/2)~objectName (5*1)~objectName
say '5'~objectName ("5")~objectName
say length('abcde')~objectName
call r
say rr()~objectName
exit
r: procedure
  say 'abc'~objectName 5~objectName
  return
rr: return 5
