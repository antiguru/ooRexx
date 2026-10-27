.local~novalue = .nv~new
say 's1' zzz
say 's2' a.7
i = 'I'; drop i
say 's3' b.i
say 's4' value('QQQ')
parse var ppp x; say 's5' x
say 's6' symbol('ZZZ') zzz
drop (zzz)
call p1
say 's7' c.
x = d.; say 's8' x
call p2
say 's9' arg(1)
do j over .array~of(1)
  say 's10' kk || j
end
interpret 'say s11 iii'
w = (>u1); say 's12' w~value
o = .k~new; o~m
say 's14' .context~variables~hasIndex('ZZZ')
.local~novalue = .nilnv~new
say 's15' yyy
signal on novalue
say 's16' yyy2
exit
novalue: say 'NOVALUE cond' condition('d'); exit
p1: procedure expose (yyy)
  return
p2: procedure
  say 's9a' pz
  return
::class nv
::method novalue; use arg n; say '  hook' n; return 'NV_'n
::class nilnv
::method novalue; use arg n; say '  nilhook' n; return .nil
::class k
::method m
  expose ov
  say 's13' ov
