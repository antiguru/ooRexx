longs = 'a longer string than seven'
longs~objectName = 'long'
say 'a longer string than seven'~objectName
say ('a longer string' || ' than seven')~objectName
say ('a longer string than seven' || '')~objectName
say left('a longer string than seven', 26)~objectName
say longs~copy~objectName
say longs~string~objectName longs~makeString~objectName longs~request('string')~objectName
call r
say other()
interpret "say 'a longer string than seven'~objectName"
say value('longs')~objectName
d = .directory~new; d['K'] = longs; say d['K']~objectName
big = 12345678901234567890
big~objectName = 'big'
say big~objectName 12345678901234567890~objectName (12345678901234567889+1)~objectName
say '12345678901234567890'~objectName
sym = ABCDEFGHIJKL
sym~objectName = 'sym'
say ABCDEFGHIJKL~objectName 'ABCDEFGHIJKL'~objectName
exit
r: procedure
  say 'a longer string than seven'~objectName
  return
::routine other
  return 'a longer string than seven'~objectName
