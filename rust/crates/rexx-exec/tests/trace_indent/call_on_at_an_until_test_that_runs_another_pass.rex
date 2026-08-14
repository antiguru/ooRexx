trace r
call on user zx name h
zn = 0
do until raiser() > 0
  zn = zn + 1
end
say 'after' zn
exit 0
raiser:
zn = zn + 0
if zn < 2 then raise user zx return 0
return 5
h:
say 'in h'
return
