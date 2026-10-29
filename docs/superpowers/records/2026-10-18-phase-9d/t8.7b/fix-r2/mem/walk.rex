parse arg lim
call on user ping name h
n = 0
call rp 'first'
say 'done' n
exit
rp: parse arg dd; raise user ping description (dd) return
h:
  n = n + 1
  if n < lim then call rp n
  else do
    c = arg(1)
    k = 0
    do while c \== .nil
      k = k + 1
      d = c~description
      prev = .nil
      do f over c~stackFrames
        if f~name == 'H' then do
          if f~arguments~items > 0 then prev = f~arguments[1]
          leave
        end
      end
      c = prev
    end
    say 'walked' k 'back to' d
  end
  return
