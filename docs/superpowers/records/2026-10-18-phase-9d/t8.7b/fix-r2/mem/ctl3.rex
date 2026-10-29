parse arg lim
o = .nil
do i = 1 to lim
  o = g(o, i)
end
k = 0
c = o
do while c \== .nil
  k = k + 1
  prev = .nil
  do f over c~stackFrames
    if f~name == 'G' then do; prev = f~arguments[1]; leave; end
  end
  c = prev
end
say 'walked' k
exit
::routine g
  use arg prev, i
  call on user ping name hh
  call rp i
  return got
rp: parse arg dd; raise user ping description (dd) return
hh: got = arg(1); return
