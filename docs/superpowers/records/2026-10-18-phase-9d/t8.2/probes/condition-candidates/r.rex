call on user u name mh
x = r1()
say 'x' x
call r2
say 'r2' result
call on user w name wh
call r4
signal on user v name sh
call r3
exit
r1: raise user u description 'd' return 'first'
r4: raise user w return
r3: raise user v return 'third'
mh: c = condition('O'); say 'mh' c~hasIndex('RESULT') c~result
return
wh: c = condition('O'); say 'wh' c~hasIndex('RESULT'); return
sh: c = condition('O'); say 'sh' c~hasIndex('RESULT') c~result
exit
::routine r2
  raise user u exit 'second'
